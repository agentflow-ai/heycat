// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Enable coverage attribute on nightly for explicit exclusions
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod audio;
mod audio_constants;
mod commands;
mod dictionary;
mod events;
mod hotkey;
mod keyboard;
mod keyboard_capture;
mod listening;
mod model;
mod parakeet;
mod paths;
mod recording;
mod shutdown;
mod transcription;
mod voice_commands;
mod worktree;

use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_store::StoreExt;

// Re-export log macros for use throughout the crate
pub use tauri_plugin_log::log::{debug, error, info, trace, warn};

/// Concrete type for HotkeyService with dynamic backend (OS-selected)
type HotkeyServiceHandle = hotkey::HotkeyServiceDyn;

/// Application entry point - starts the Tauri event loop.
/// Note: This function cannot be unit tested as it starts a GUI.
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("heycat".to_string()),
                    }),
                ])
                .level(if cfg!(debug_assertions) {
                    tauri_plugin_log::log::LevelFilter::Debug
                } else {
                    tauri_plugin_log::log::LevelFilter::Info
                })
                // Suppress verbose DEBUG logs from tract ONNX inference library
                // These flood the console during model optimization and cause multi-second delays
                .level_for("tract_core", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("tract_onnx", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("tract_hir", tauri_plugin_log::log::LevelFilter::Warn)
                .level_for("tract_linalg", tauri_plugin_log::log::LevelFilter::Warn)
                .build(),
        )
        .setup(|app| {
            info!("Setting up heycat...");

            // Handle Ctrl+C to prevent paste during terminal termination
            // Must be configured early so SIGINT in `tauri dev` triggers a clean exit path.
            // We avoid `std::process::exit(0)` here because it skips destructors and can leave
            // CoreGraphics keyboard synthesis mid-flight (leading to stuck keys / multi-paste).
            shutdown::register_app_handle(app.handle().clone());
            if let Err(e) = ctrlc::set_handler(|| {
                eprintln!("[PASTE-TRACE] ctrlc handler fired - about to call signal_shutdown()");
                shutdown::signal_shutdown();
                eprintln!("[PASTE-TRACE] ctrlc handler - about to stop CGEventTap");
                // Stop CGEventTap run loop to prevent spurious events during exit
                shutdown::stop_cgeventtap();
                eprintln!("[PASTE-TRACE] ctrlc handler - requesting graceful app exit");
                shutdown::request_app_exit(0);
            }) {
                warn!("Failed to set Ctrl+C handler: {}", e);
            }

            // Detect worktree context for data isolation
            let worktree_context = worktree::detect_worktree();
            let worktree_state = worktree::WorktreeState { context: worktree_context.clone() };
            let settings_file = worktree_state.settings_file_name();
            if let Some(ref ctx) = worktree_context {
                info!("Running in worktree: {} (gitdir: {:?})", ctx.identifier, ctx.gitdir_path);
                info!("Using worktree-specific settings file: {}", settings_file);
            } else {
                info!("Running in main repository");
            }
            app.manage(worktree_state);

            // Set dynamic window title based on worktree context
            if let Some(window) = app.get_webview_window("main") {
                let title = match &worktree_context {
                    Some(ctx) => format!("heycat - {}", ctx.identifier),
                    None => "heycat".to_string(),
                };
                if let Err(e) = window.set_title(&title) {
                    warn!("Failed to set window title: {}", e);
                } else {
                    debug!("Window title set to: {}", title);
                }
            }

            // Check for collision with another running instance
            // This must happen before any state initialization that writes to data directories
            let collision_result = worktree::check_collision(worktree_context.as_ref());
            match &collision_result {
                Ok(worktree::CollisionResult::NoCollision) => {
                    debug!("No collision detected, proceeding with startup");
                }
                Ok(collision @ worktree::CollisionResult::InstanceRunning { .. }) => {
                    // Use format_collision_error for consistent error messaging
                    if let Some((title, message, steps)) = worktree::format_collision_error(collision) {
                        error!("{}: {}", title, message);
                        for step in &steps {
                            warn!("Resolution: {}", step);
                        }
                        // Return error to prevent app from starting with conflicting data
                        return Err(message.into());
                    }
                }
                Ok(collision @ worktree::CollisionResult::StaleLock { lock_file }) => {
                    if let Some((title, message, _)) = worktree::format_collision_error(collision) {
                        warn!("{}: {}", title, message);
                    }
                    info!("Cleaning up stale lock file from crashed instance...");
                    if let Err(e) = worktree::cleanup_stale_lock(lock_file) {
                        warn!("Failed to clean up stale lock file: {}", e);
                    } else {
                        info!("Stale lock file cleaned up successfully");
                    }
                }
                Err(e) => {
                    // Non-fatal: log warning but continue startup
                    warn!("Failed to check for collisions: {}", e);
                }
            }

            // Create lock file for this instance
            match worktree::create_lock(worktree_context.as_ref()) {
                Ok(lock_path) => {
                    debug!("Lock file created: {:?}", lock_path);
                }
                Err(e) => {
                    warn!("Failed to create lock file: {}", e);
                    // Non-fatal: continue without lock file
                }
            }

            // Create shared state for recording manager
            let recording_state = Arc::new(Mutex::new(recording::RecordingManager::new()));

            // Manage the state for Tauri commands
            app.manage(recording_state.clone());

            // Create and manage listening state, restoring persisted auto-start setting
            let listening_enabled = app
                .store(&settings_file)
                .ok()
                .and_then(|store| store.get("listening.autoStartOnLaunch"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            debug!("Restored listening.autoStartOnLaunch from store: {}", listening_enabled);
            let listening_state = Arc::new(Mutex::new(
                listening::ListeningManager::with_enabled(listening_enabled),
            ));
            app.manage(listening_state.clone());

            // Create and manage audio monitor state for device testing
            let audio_monitor = Arc::new(audio::AudioMonitorHandle::spawn());
            app.manage(audio_monitor);

            // Create shared transcription model (single ~3GB Parakeet model)
            // This model is shared between all transcription consumers and WakeWordDetector
            debug!("Creating SharedTranscriptionModel...");
            let shared_transcription_model = Arc::new(parakeet::SharedTranscriptionModel::new());

            // Create listening pipeline with shared model
            let mut pipeline = listening::ListeningPipeline::new();
            pipeline.set_shared_model((*shared_transcription_model).clone());
            let listening_pipeline = Arc::new(Mutex::new(pipeline));
            app.manage(listening_pipeline.clone());

            // Create and manage recording detectors (for silence/cancel detection during recording)
            // Use worktree-aware recordings directory for data isolation
            let recordings_dir = paths::get_recordings_dir(worktree_context.as_ref())
                .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat").join("recordings"));
            let recording_detectors = Arc::new(Mutex::new(listening::RecordingDetectors::with_recordings_dir(recordings_dir.clone())));
            app.manage(recording_detectors.clone());

            // Create event emitter, audio thread, and hotkey integration
            debug!("Creating audio thread...");
            let emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
            let audio_thread = Arc::new(audio::AudioThreadHandle::spawn());
            debug!("Audio thread spawned");

            // Initialize shared denoiser at startup (eliminates 2s delay on each recording)
            // Graceful degradation: if loading fails, recordings work without noise suppression
            debug!("Loading shared DTLN denoiser...");
            let shared_denoiser = match audio::SharedDenoiser::try_load() {
                Ok(denoiser) => {
                    info!("Shared DTLN denoiser loaded successfully (eliminates 2s recording delay)");
                    Some(Arc::new(denoiser))
                }
                Err(e) => {
                    warn!("Failed to load shared denoiser, recordings will work without noise suppression: {}", e);
                    None
                }
            };

            // Manage audio thread state for Tauri commands
            app.manage(audio_thread.clone());

            // Manage shared denoiser for Tauri commands
            app.manage(shared_denoiser.clone());

            // Manage shared transcription model for Tauri commands
            app.manage(shared_transcription_model.clone());

            // Create and manage VoiceCommandsState
            debug!("Creating VoiceCommandsState...");
            let (command_registry, command_matcher, action_dispatcher) = match voice_commands::VoiceCommandsState::new_with_context(worktree_context.as_ref()) {
                Ok(voice_state) => {
                    // Share the same registry between UI and matcher
                    let registry = voice_state.registry.clone();
                    let matcher = Arc::new(voice_commands::matcher::CommandMatcher::new());
                    let executor_state = voice_commands::executor::ExecutorState::new();
                    let dispatcher = executor_state.dispatcher.clone();

                    app.manage(voice_state);
                    app.manage(executor_state);
                    debug!("VoiceCommandsState initialized successfully");
                    (Some(registry), Some(matcher), Some(dispatcher))
                }
                Err(e) => {
                    warn!("Failed to initialize VoiceCommandsState: {}", e);
                    // Still create executor state even if voice commands failed
                    let executor_state = voice_commands::executor::ExecutorState::new();
                    app.manage(executor_state);
                    (None, None, None)
                }
            };
            debug!("ExecutorState initialized successfully");

            // Eager model loading at startup (if models exist)
            // Load TDT model into shared model if available
            // This single model instance will be shared between all transcription consumers and WakeWordDetector
            if let Ok(true) = model::check_model_exists_for_type(model::download::ModelType::ParakeetTDT) {
                if let Ok(model_dir) = model::download::get_model_dir(model::download::ModelType::ParakeetTDT) {
                    info!("Loading shared Parakeet TDT model from {:?}...", model_dir);
                    match shared_transcription_model.load(&model_dir) {
                        Ok(()) => info!("Shared Parakeet TDT model loaded successfully (saves ~3GB by sharing)"),
                        Err(e) => warn!("Failed to load Parakeet TDT model: {}", e),
                    }
                }
            } else {
                info!("TDT model not found, batch transcription and wake word detection will require download first");
            }

            // Create RecordingTranscriptionService for unified transcription flow
            // This service is used by stop_recording command, HotkeyIntegration, and wake word flow
            debug!("Creating RecordingTranscriptionService...");
            let transcription_service_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
            let mut transcription_service = transcription::RecordingTranscriptionService::new(
                shared_transcription_model.clone(),
                transcription_service_emitter,
                recording_state.clone(),
                app.handle().clone(),
            );

            // Create shared dictionary store (used by both transcription and CRUD commands)
            debug!("Loading dictionary entries...");
            let dictionary_store = {
                let mut store = match dictionary::DictionaryStore::with_default_path_context(worktree_context.as_ref()) {
                    Ok(store) => store,
                    Err(e) => {
                        warn!("Failed to initialize dictionary store: {}, using empty dictionary", e);
                        dictionary::DictionaryStore::new(std::path::PathBuf::new())
                    }
                };
                if let Err(e) = store.load() {
                    warn!("Failed to load dictionary entries: {}, using empty dictionary", e);
                }
                Mutex::new(store)
            };

            // Create expander for transcription service from shared store
            {
                let store = dictionary_store.lock().expect("dictionary store lock poisoned during setup");
                let entries: Vec<dictionary::DictionaryEntry> = store.list().into_iter().cloned().collect();
                if entries.is_empty() {
                    debug!("No dictionary entries loaded");
                } else {
                    info!("Loaded {} dictionary entries for expansion", entries.len());
                    let expander = dictionary::DictionaryExpander::new(&entries);
                    transcription_service = transcription_service.with_dictionary_expander(expander);
                    debug!("Dictionary expander wired to TranscriptionService");
                }
            }

            // Wire up voice command integration to transcription service if available
            if let (Some(ref registry), Some(ref matcher), Some(ref dispatcher)) = (&command_registry, &command_matcher, &action_dispatcher) {
                let service_command_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
                transcription_service = transcription_service
                    .with_command_registry(registry.clone())
                    .with_command_matcher(matcher.clone())
                    .with_action_dispatcher(dispatcher.clone())
                    .with_command_emitter(service_command_emitter);
                debug!("Voice commands wired to TranscriptionService");
            }

            let transcription_service = Arc::new(transcription_service);
            app.manage(transcription_service.clone());
            debug!("RecordingTranscriptionService created and managed");

            // Create a wrapper to pass to HotkeyIntegration (it needs owned value, not Arc)
            let recording_emitter = commands::TauriEventEmitter::new(app.handle().clone());

            // Create shortcut backend for Escape key registration (used by HotkeyIntegration)
            // Uses platform-specific backend: CGEventTap on macOS, Tauri on Windows/Linux
            let escape_backend = hotkey::create_shortcut_backend(app.handle().clone());

            // Create transcription callback that delegates to TranscriptionService
            let transcription_service_for_callback = transcription_service.clone();
            let transcription_callback: Arc<dyn Fn(String) + Send + Sync> =
                Arc::new(move |file_path: String| {
                    transcription_service_for_callback.process_recording(file_path);
                });

            // Create hotkey event emitter for key blocking notifications
            let hotkey_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));

            let mut integration_builder = hotkey::HotkeyIntegration::<
                commands::TauriEventEmitter,
                commands::TauriEventEmitter,
                commands::TauriEventEmitter,
            >::new(recording_emitter)
                .with_app_handle(app.handle().clone())
                .with_audio_thread(audio_thread)
                .with_shared_denoiser(shared_denoiser)
                .with_shared_transcription_model(shared_transcription_model)
                .with_transcription_emitter(emitter)
                .with_recording_state(recording_state.clone())
                .with_listening_state(listening_state)
                .with_listening_pipeline(listening_pipeline.clone())
                .with_recording_detectors(recording_detectors.clone())
                .with_recordings_dir(recordings_dir)
                .with_shortcut_backend(escape_backend)
                .with_transcription_callback(transcription_callback)
                .with_hotkey_emitter(hotkey_emitter)
                .with_silence_detection_enabled(false); // Disable for push-to-talk

            // Wire up voice command integration using grouped config if available
            // (still needed for HotkeyIntegration's silence detection callback)
            if let (Some(registry), Some(matcher), Some(dispatcher)) = (command_registry, command_matcher, action_dispatcher) {
                let command_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
                integration_builder = integration_builder
                    .with_voice_commands(hotkey::integration::VoiceCommandConfig {
                        registry,
                        matcher,
                        dispatcher,
                        emitter: Some(command_emitter),
                    });
                debug!("Voice command integration wired up using grouped config");
            }

            let integration = Arc::new(Mutex::new(integration_builder));

            // Set up escape callback after integration is created (so callback can capture reference)
            // Double-tap Escape cancels the recording without transcription
            {
                let integration_for_escape = integration.clone();
                let state_for_escape = recording_state.clone();
                let escape_callback: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                    debug!("Double-tap Escape detected, cancelling recording");
                    if let Ok(mut guard) = integration_for_escape.lock() {
                        guard.cancel_recording(&state_for_escape, "double-tap-escape");
                    } else {
                        error!("Failed to acquire integration lock for cancel");
                    }
                });

                if let Ok(mut guard) = integration.lock() {
                    guard.set_escape_callback(escape_callback);
                    debug!("Escape callback wired up for recording cancellation");
                }
            }

            // Manage integration state so it can be accessed from commands
            app.manage(integration.clone());

            // Clone for callback
            let integration_clone = integration.clone();
            let state_clone = recording_state.clone();
            let app_handle_clone = app.handle().clone();

            // Register hotkey using platform-specific backend
            // Uses CGEventTap on macOS (supports fn key, media keys), Tauri on Windows/Linux
            // Load saved shortcut from settings - user must set one during onboarding
            let saved_shortcut = app
                .store(&settings_file)
                .ok()
                .and_then(|store| store.get("hotkey.recordingShortcut"))
                .and_then(|v| v.as_str().map(|s| s.to_string()));

            let backend = hotkey::create_shortcut_backend(app.handle().clone());
            let service = hotkey::HotkeyServiceDyn::new(backend);

            if let Some(shortcut) = saved_shortcut {
                info!("Registering global hotkey: {}...", shortcut);
                if let Err(e) = service.backend.register(&shortcut, Box::new(move || {
                    debug!("Hotkey pressed!");
                    match integration_clone.lock() {
                        Ok(mut guard) => {
                            guard.handle_toggle(&state_clone);
                        }
                        Err(e) => {
                            error!("Failed to acquire integration lock: {}", e);
                            // Emit error event so frontend knows something went wrong
                            let _ = app_handle_clone.emit(
                                events::event_names::RECORDING_ERROR,
                                events::RecordingErrorPayload {
                                    message: "Internal error: please restart the application"
                                        .to_string(),
                                },
                            );
                        }
                    }
                })) {
                    warn!("Failed to register recording hotkey: {:?}", e);
                    warn!("Application will continue without global hotkey support");
                }
            } else {
                info!("No recording shortcut configured - user will set one during onboarding");
            }

            // Store service in state for cleanup on exit
            app.manage(service);

            // Create keyboard capture state for shortcut recording with fn key support
            let keyboard_capture = Arc::new(Mutex::new(keyboard_capture::KeyboardCapture::new()));
            app.manage(keyboard_capture);

            // Manage shared dictionary store for CRUD commands
            app.manage(dictionary_store);

            info!("Setup complete! Ready to record.");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // Signal shutdown FIRST - prevents async tasks from pasting during cleanup
                shutdown::signal_shutdown();

                debug!("Window destroyed, cleaning up...");

                // Get worktree context for cleanup
                let worktree_context = window.app_handle()
                    .try_state::<worktree::WorktreeState>()
                    .and_then(|s| s.context.clone());

                // Clean up lock file on graceful shutdown
                if let Err(e) = worktree::remove_lock(worktree_context.as_ref()) {
                    warn!("Failed to remove lock file: {}", e);
                } else {
                    debug!("Lock file removed successfully");
                }

                // Unregister hotkey on window close - use saved shortcut from settings
                if let Some(service) = window.app_handle().try_state::<HotkeyServiceHandle>() {
                    use tauri_plugin_store::StoreExt;
                    // Get worktree-aware settings file name
                    let settings_file = window.app_handle()
                        .try_state::<worktree::WorktreeState>()
                        .map(|s| s.settings_file_name())
                        .unwrap_or_else(|| worktree::DEFAULT_SETTINGS_FILE.to_string());
                    if let Some(shortcut) = window.app_handle()
                        .store(&settings_file)
                        .ok()
                        .and_then(|store| store.get("hotkey.recordingShortcut"))
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                    {
                        if let Err(e) = service.backend.unregister(&shortcut) {
                            warn!("Failed to unregister hotkey '{}': {}", shortcut, e);
                        } else {
                            debug!("Hotkey '{}' unregistered successfully", shortcut);
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::get_recording_state,
            commands::get_last_recording_buffer,
            commands::clear_last_recording_buffer,
            commands::list_recordings,
            commands::delete_recording,
            commands::transcribe_file,
            commands::enable_listening,
            commands::disable_listening,
            commands::get_listening_status,
            commands::list_audio_devices,
            commands::start_audio_monitor,
            commands::stop_audio_monitor,
            model::check_parakeet_model_status,
            model::download_model,
            voice_commands::get_commands,
            voice_commands::add_command,
            voice_commands::update_command,
            voice_commands::remove_command,
            voice_commands::executor::test_command,
            commands::suspend_recording_shortcut,
            commands::resume_recording_shortcut,
            commands::update_recording_shortcut,
            commands::get_recording_shortcut,
            commands::start_shortcut_recording,
            commands::stop_shortcut_recording,
            commands::open_accessibility_preferences,
            commands::get_settings_file_name,
            commands::dictionary::list_dictionary_entries,
            commands::dictionary::add_dictionary_entry,
            commands::dictionary::update_dictionary_entry,
            commands::dictionary::delete_dictionary_entry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Enable coverage attribute on nightly for explicit exclusions
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod activation;
mod audio;
mod audio_constants;
mod commands;
mod dictionary;
mod events;
mod hotkey;
mod keyboard;
mod keyboard_capture;
mod model;
mod parakeet;
mod paths;
mod recording;
mod shutdown;
mod storage;
mod swift;
mod turso;
mod transcription;
mod util;
mod voice_commands;
mod wake_handler;
mod window_context;
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
        .plugin(tauri_plugin_fs::init())
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
                // Suppress verbose INFO logs from ONNX Runtime during model loading
                .level_for("ort", tauri_plugin_log::log::LevelFilter::Warn)
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
                shutdown::signal_shutdown();
                // Stop CGEventTap run loop to prevent spurious events during exit
                shutdown::stop_cgeventtap();
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

            // Set dynamic window title based on worktree context (dev builds only)
            if let Some(window) = app.get_webview_window("main") {
                let title = match &worktree_context {
                    Some(ctx) if cfg!(debug_assertions) => {
                        // In dev builds with worktree, include recording hotkey if configured
                        let recording_shortcut = app
                            .store(&settings_file)
                            .ok()
                            .and_then(|store| store.get("hotkey.recordingShortcut"))
                            .and_then(|v| v.as_str().map(|s| s.to_string()));

                        match recording_shortcut {
                            Some(shortcut) => format!("heycat - {} ({})", ctx.identifier, shortcut),
                            None => format!("heycat - {}", ctx.identifier),
                        }
                    }
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

            // Initialize Turso/libsql embedded database client
            // Uses worktree-aware data directory for isolation
            let turso_data_dir = paths::get_data_dir(worktree_context.as_ref())
                .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat"));
            let turso_client: std::sync::Arc<turso::TursoClient> = match turso::TursoClient::new_blocking(turso_data_dir) {
                Ok(client) => {
                    info!("Turso database initialized at: {:?}", client.db_path());

                    // Initialize database schema (creates tables if needed, runs migrations)
                    // Must handle the case where no Tokio runtime is available
                    let schema_result = match tokio::runtime::Handle::try_current() {
                        Ok(handle) => tokio::task::block_in_place(|| {
                            handle.block_on(turso::initialize_schema(&client))
                        }),
                        Err(_) => {
                            // No runtime available, create a temporary one
                            let rt = tokio::runtime::Runtime::new()
                                .expect("Failed to create tokio runtime for schema init");
                            rt.block_on(turso::initialize_schema(&client))
                        }
                    };
                    match schema_result {
                        Ok(()) => {
                            debug!("Turso database schema initialized");
                        }
                        Err(e) => {
                            error!("Failed to initialize Turso schema: {}", e);
                            return Err(format!("Turso schema initialization failed: {}", e).into());
                        }
                    }

                    std::sync::Arc::new(client)
                }
                Err(e) => {
                    // Fatal: Turso database is required for data storage
                    error!("Failed to initialize Turso database: {}", e);
                    return Err(format!("Database initialization failed: {}", e).into());
                }
            };
            app.manage(turso_client.clone());

            // Create shared state for recording manager
            let recording_state = Arc::new(Mutex::new(recording::RecordingManager::new()));

            // Manage the state for Tauri commands
            app.manage(recording_state.clone());

            // Create and manage audio monitor state for device testing
            let audio_monitor = Arc::new(audio::AudioMonitorHandle::spawn());
            app.manage(audio_monitor.clone());

            // Pre-initialize the audio engine at startup so it's ready for recording
            // The engine stays running continuously for level monitoring
            // Use the saved device from settings so recording doesn't need to switch devices
            let saved_device = {
                let settings_file = worktree_context
                    .as_ref()
                    .map(|ctx| ctx.settings_file_name())
                    .unwrap_or_else(|| worktree::DEFAULT_SETTINGS_FILE.to_string());
                app.store(&settings_file)
                    .ok()
                    .and_then(|store| store.get("audio.selectedDevice"))
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
            };
            if let Some(ref device) = saved_device {
                debug!("Pre-initializing audio engine with saved device: {}", device);
            }
            if let Err(e) = audio_monitor.init(saved_device) {
                warn!("Failed to pre-initialize audio engine: {} (will initialize lazily)", e);
            } else {
                info!("Audio engine pre-initialized and running");
            }

            // Create shared transcription model (single ~3GB Parakeet model)
            // This model is shared between all transcription consumers
            debug!("Creating SharedTranscriptionModel...");
            let shared_transcription_model = Arc::new(parakeet::SharedTranscriptionModel::new());

            // Create and manage recording detectors (for silence detection during recording)
            // Use worktree-aware recordings directory for data isolation
            let recordings_dir = paths::get_recordings_dir(worktree_context.as_ref())
                .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat").join("recordings"));
            let recording_detectors = Arc::new(Mutex::new(recording::RecordingDetectors::with_recordings_dir(recordings_dir.clone())));
            app.manage(recording_detectors.clone());

            // Create event emitter, audio thread, and hotkey integration
            debug!("Creating audio thread...");
            let emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
            let audio_thread = Arc::new(audio::AudioThreadHandle::spawn());
            debug!("Audio thread spawned");

            // Manage audio thread state for Tauri commands
            app.manage(audio_thread.clone());

            // Manage shared transcription model for Tauri commands
            app.manage(shared_transcription_model.clone());

            // Create and manage voice command executor and registry for command matching
            debug!("Creating voice command infrastructure...");
            let executor_state = voice_commands::executor::ExecutorState::new();
            let dispatcher = executor_state.dispatcher.clone();
            app.manage(executor_state);

            // Initialize command matcher for transcription service command matching
            // Voice commands are fetched directly from TursoClient - no registry cache needed
            let command_matcher = Arc::new(voice_commands::matcher::CommandMatcher::new());
            let action_dispatcher = Some(dispatcher);
            debug!("Voice command infrastructure initialized");

            // Eager model loading at startup (if models exist)
            // Load TDT model into shared model if available
            // This single model instance will be shared between all transcription consumers and WakeWordDetector
            if let Ok(true) = model::check_model_exists_for_type(model::download::ModelType::ParakeetTDT) {
                if let Ok(model_dir) = model::download::get_model_dir(model::download::ModelType::ParakeetTDT) {
                    info!("Loading shared Parakeet TDT model from {:?}...", model_dir);
                    match shared_transcription_model.load(&model_dir) {
                        Ok(()) => {
                            info!("Shared Parakeet TDT model loaded successfully (saves ~3GB by sharing)");

                            // Initialize wake handler to reload model after system sleep/wake
                            // The ONNX model may become invalid after sleep, so we reload proactively
                            wake_handler::init_wake_handler(
                                app.handle().clone(),
                                (*shared_transcription_model).clone(),
                                model_dir,
                            );
                        }
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

            // Start window monitor for context-sensitive commands
            debug!("Starting window monitor...");
            let window_monitor = Arc::new({
                let mut monitor = window_context::WindowMonitor::new();
                if let Err(e) = monitor.start(app.handle().clone(), turso_client.clone()) {
                    warn!("Failed to start window monitor: {}", e);
                }
                Mutex::new(monitor)
            });

            // Create context resolver for window-aware command/dictionary resolution
            debug!("Creating context resolver...");
            let context_resolver = Arc::new(window_context::ContextResolver::new(
                window_monitor.clone(),
                turso_client.clone(),
            ));

            // Create expander for transcription service from dictionary entries
            {
                let entries = match tokio::runtime::Handle::try_current() {
                    Ok(handle) => tokio::task::block_in_place(|| {
                        handle.block_on(turso_client.list_dictionary_entries())
                    }),
                    Err(_) => {
                        let rt = tokio::runtime::Runtime::new()
                            .expect("Failed to create runtime for dictionary entries");
                        rt.block_on(turso_client.list_dictionary_entries())
                    }
                };
                match entries {
                    Ok(entries) => {
                        if entries.is_empty() {
                            debug!("No dictionary entries loaded");
                        } else {
                            info!("Loaded {} dictionary entries for expansion", entries.len());
                            let expander = dictionary::DictionaryExpander::new(&entries);
                            transcription_service = transcription_service.with_dictionary_expander(expander);
                            debug!("Dictionary expander wired to TranscriptionService");
                        }
                    }
                    Err(e) => {
                        warn!("Failed to load dictionary entries: {}", e);
                    }
                }
            }

            // Wire up voice command integration to transcription service
            if let Some(ref dispatcher) = action_dispatcher {
                let service_command_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
                transcription_service = transcription_service
                    .with_turso_client(turso_client.clone())
                    .with_command_matcher(command_matcher.clone())
                    .with_action_dispatcher(dispatcher.clone())
                    .with_command_emitter(service_command_emitter);
                debug!("Voice commands wired to TranscriptionService");
            }

            // Wire context resolver to transcription service for window-aware command resolution
            transcription_service = transcription_service
                .with_context_resolver(context_resolver);
            debug!("Context resolver wired to TranscriptionService");

            let transcription_service = Arc::new(transcription_service);
            app.manage(transcription_service.clone());
            debug!("RecordingTranscriptionService created and managed");

            // Create a wrapper to pass to HotkeyIntegration (it needs owned value, not Arc)
            let recording_emitter = commands::TauriEventEmitter::new(app.handle().clone());

            // Create SINGLE shared shortcut backend for all hotkeys
            // Uses platform-specific backend: CGEventTap on macOS, Tauri on Windows/Linux
            // IMPORTANT: Both escape key (HotkeyIntegration) and main hotkey (HotkeyServiceDyn) share this
            // to avoid multiple CGEventTaps which causes keyboard freezing when one stops
            let shared_backend = hotkey::create_shortcut_backend(app.handle().clone());

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
                .with_audio_monitor(audio_monitor)
                .with_shared_transcription_model(shared_transcription_model)
                .with_transcription_emitter(emitter)
                .with_recording_state(recording_state.clone())
                .with_recording_detectors(recording_detectors.clone())
                .with_recordings_dir(recordings_dir)
                .with_shortcut_backend(shared_backend.clone())
                .with_transcription_callback(transcription_callback)
                .with_hotkey_emitter(hotkey_emitter)
                .with_silence_detection_enabled(false); // Disable for push-to-talk

            // Wire up voice command integration using grouped config
            // (still needed for HotkeyIntegration's silence detection callback)
            if let Some(dispatcher) = action_dispatcher {
                let command_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
                integration_builder = integration_builder
                    .with_voice_commands(hotkey::integration::VoiceCommandConfig {
                        turso_client: turso_client.clone(),
                        matcher: command_matcher.clone(),
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

            // Load saved shortcut and recording mode from settings
            let saved_shortcut = app
                .store(&settings_file)
                .ok()
                .and_then(|store| store.get("hotkey.recordingShortcut"))
                .and_then(|v| v.as_str().map(|s| s.to_string()));

            let recording_mode: hotkey::RecordingMode = app
                .store(&settings_file)
                .ok()
                .and_then(|store| store.get("shortcuts.recordingMode"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            // Set recording mode on the integration
            if let Ok(mut guard) = integration.lock() {
                guard.set_recording_mode(recording_mode);
            }

            // Reuse shared_backend for main hotkey registration (same backend as escape key)
            let service = hotkey::HotkeyServiceDyn::new(shared_backend.clone());

            if let Some(shortcut) = saved_shortcut {
                info!("Registering global hotkey: {} (initial mode: {:?})...", shortcut, recording_mode);
                use hotkey::ShortcutBackendExt;

                // Always register with press + release callbacks for dynamic mode switching
                // The callbacks check the current mode at runtime
                let integration_press = integration.clone();
                let state_press = recording_state.clone();
                let app_handle_press = app.handle().clone();

                let integration_release = integration.clone();
                let state_release = recording_state.clone();
                let app_handle_release = app.handle().clone();

                let mut registered = false;

                // Try CGEventTap backend (macOS)
                #[cfg(target_os = "macos")]
                if let Some(ext_backend) = service.backend.as_any().downcast_ref::<hotkey::cgeventtap_backend::CGEventTapHotkeyBackend>() {
                    if let Err(e) = ext_backend.register_with_release(
                        &shortcut,
                        Box::new(move || {
                            // Clone for the async task - the callback must return immediately
                            // to avoid blocking the CGEventTap run loop (which would freeze ALL keyboard input)
                            let integration = integration_press.clone();
                            let state = state_press.clone();
                            let app_handle = app_handle_press.clone();

                            // Spawn the heavy work on Tauri's async runtime
                            tauri::async_runtime::spawn(async move {
                                match integration.lock() {
                                    Ok(mut guard) => {
                                        let mode = guard.recording_mode();
                                        debug!("Hotkey pressed (mode: {:?})", mode);
                                        match mode {
                                            hotkey::RecordingMode::Toggle => { guard.handle_toggle(&state); }
                                            hotkey::RecordingMode::PushToTalk => { guard.handle_hotkey_press(&state); }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to acquire integration lock: {}", e);
                                        let _ = app_handle.emit(
                                            events::event_names::RECORDING_ERROR,
                                            events::RecordingErrorPayload {
                                                message: "Internal error: please restart the application".to_string(),
                                            },
                                        );
                                    }
                                }
                            });
                        }),
                        Box::new(move || {
                            // Track timing to diagnose keyboard freezing issues
                            let cb_start = std::time::Instant::now();

                            // Clone for the async task - the callback must return immediately
                            // to avoid blocking the CGEventTap run loop (which would freeze ALL keyboard input)
                            let integration = integration_release.clone();
                            let state = state_release.clone();
                            let app_handle = app_handle_release.clone();

                            let clone_elapsed = cb_start.elapsed();

                            // Spawn the heavy work on Tauri's async runtime
                            tauri::async_runtime::spawn(async move {
                                match integration.lock() {
                                    Ok(mut guard) => {
                                        let mode = guard.recording_mode();
                                        debug!("Hotkey released (mode: {:?})", mode);
                                        // Only handle release in PTT mode
                                        if mode == hotkey::RecordingMode::PushToTalk {
                                            guard.handle_hotkey_release(&state);
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to acquire integration lock: {}", e);
                                        let _ = app_handle.emit(
                                            events::event_names::RECORDING_ERROR,
                                            events::RecordingErrorPayload {
                                                message: "Internal error: please restart the application".to_string(),
                                            },
                                        );
                                    }
                                }
                            });

                            let total_elapsed = cb_start.elapsed();
                            if total_elapsed.as_millis() > 5 {
                                warn!(
                                    "Release callback took {:?} (clone: {:?}) - SLOW!",
                                    total_elapsed,
                                    clone_elapsed
                                );
                            }
                        }),
                    ) {
                        warn!("Failed to register hotkey: {:?}", e);
                    } else {
                        registered = true;
                    }
                }

                // Try rdev backend (Windows/Linux)
                #[cfg(not(target_os = "macos"))]
                if !registered {
                    if let Some(ext_backend) = service.backend.as_any().downcast_ref::<hotkey::RdevShortcutBackend>() {
                        if let Err(e) = ext_backend.register_with_release(
                            &shortcut,
                            Box::new(move || {
                                // Clone for the async task - the callback must return immediately
                                // to avoid blocking the rdev event loop
                                let integration = integration_press.clone();
                                let state = state_press.clone();
                                let app_handle = app_handle_press.clone();

                                // Spawn the heavy work on Tauri's async runtime
                                tauri::async_runtime::spawn(async move {
                                    match integration.lock() {
                                        Ok(mut guard) => {
                                            let mode = guard.recording_mode();
                                            debug!("Hotkey pressed (mode: {:?})", mode);
                                            match mode {
                                                hotkey::RecordingMode::Toggle => { guard.handle_toggle(&state); }
                                                hotkey::RecordingMode::PushToTalk => { guard.handle_hotkey_press(&state); }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to acquire integration lock: {}", e);
                                            let _ = app_handle.emit(
                                                events::event_names::RECORDING_ERROR,
                                                events::RecordingErrorPayload {
                                                    message: "Internal error: please restart the application".to_string(),
                                                },
                                            );
                                        }
                                    }
                                });
                            }),
                            Box::new(move || {
                                // Clone for the async task - the callback must return immediately
                                // to avoid blocking the rdev event loop
                                let integration = integration_release.clone();
                                let state = state_release.clone();
                                let app_handle = app_handle_release.clone();

                                // Spawn the heavy work on Tauri's async runtime
                                tauri::async_runtime::spawn(async move {
                                    match integration.lock() {
                                        Ok(mut guard) => {
                                            let mode = guard.recording_mode();
                                            debug!("Hotkey released (mode: {:?})", mode);
                                            // Only handle release in PTT mode
                                            if mode == hotkey::RecordingMode::PushToTalk {
                                                guard.handle_hotkey_release(&state);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to acquire integration lock: {}", e);
                                            let _ = app_handle.emit(
                                                events::event_names::RECORDING_ERROR,
                                                events::RecordingErrorPayload {
                                                    message: "Internal error: please restart the application".to_string(),
                                                },
                                            );
                                        }
                                    }
                                });
                            }),
                        ) {
                            warn!("Failed to register hotkey: {:?}", e);
                        } else {
                            registered = true;
                        }
                    }
                }

                if !registered {
                    warn!("Backend doesn't support key release detection");
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

            // Manage window monitor for Tauri commands
            app.manage(window_monitor);

            // Ensure the app is activated on macOS so the UI receives events immediately.
            // Without this, the window may be visible but not receive left-clicks
            // until the user manually activates the app (e.g., via Cmd+Tab).
            activation::activate_app();

            info!("Setup complete! Ready to record.");

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                // Only trigger shutdown when the main window is destroyed
                // (not for splash or other transient windows)
                if window.label() != "main" {
                    debug!("Non-main window '{}' destroyed, skipping cleanup", window.label());
                    return;
                }

                // Signal shutdown FIRST - prevents async tasks from pasting during cleanup
                shutdown::signal_shutdown();

                debug!("Main window destroyed, cleaning up...");

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

                // Stop window monitor on window close
                if let Some(monitor) = window.app_handle().try_state::<Arc<Mutex<window_context::WindowMonitor>>>() {
                    if let Ok(mut monitor) = monitor.lock() {
                        if let Err(e) = monitor.stop() {
                            warn!("Failed to stop window monitor: {}", e);
                        } else {
                            debug!("Window monitor stopped successfully");
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
            commands::list_audio_devices,
            commands::start_audio_monitor,
            commands::stop_audio_monitor,
            commands::init_audio_monitor,
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
            commands::get_recording_mode,
            commands::set_recording_mode,
            commands::start_shortcut_recording,
            commands::stop_shortcut_recording,
            commands::open_accessibility_preferences,
            commands::get_settings_file_name,
            commands::dictionary::list_dictionary_entries,
            commands::dictionary::add_dictionary_entry,
            commands::dictionary::update_dictionary_entry,
            commands::dictionary::delete_dictionary_entry,
            commands::window_context::get_active_window_info,
            commands::window_context::list_running_applications,
            commands::window_context::list_window_contexts,
            commands::window_context::add_window_context,
            commands::window_context::update_window_context,
            commands::window_context::delete_window_context,
            commands::list_transcriptions,
            commands::get_transcriptions_by_recording,
            commands::show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


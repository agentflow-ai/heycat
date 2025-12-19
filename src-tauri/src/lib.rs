// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// Enable coverage attribute on nightly for explicit exclusions
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod audio;
mod audio_constants;
mod commands;
mod events;
mod hotkey;
mod keyboard_capture;
mod listening;
mod model;
mod parakeet;
mod recording;
mod transcription;
mod voice_commands;

use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_store::StoreExt;

// Re-export log macros for use throughout the crate
pub use tauri_plugin_log::log::{debug, error, info, trace, warn};

/// Concrete type for HotkeyService with TauriShortcutBackend
type HotkeyServiceHandle = hotkey::HotkeyService<hotkey::TauriShortcutBackend>;

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
                .build(),
        )
        .setup(|app| {
            info!("Setting up heycat...");

            // Create shared state for recording manager
            let recording_state = Arc::new(Mutex::new(recording::RecordingManager::new()));

            // Manage the state for Tauri commands
            app.manage(recording_state.clone());

            // Create and manage listening state, restoring persisted enabled setting
            let listening_enabled = app
                .store("settings.json")
                .ok()
                .and_then(|store| store.get("listening.enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            debug!("Restored listening.enabled from store: {}", listening_enabled);
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
            let recording_detectors = Arc::new(Mutex::new(listening::RecordingDetectors::new()));
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

            // Create and manage VoiceCommandsState
            debug!("Creating VoiceCommandsState...");
            let (command_registry, command_matcher, action_dispatcher) = match voice_commands::VoiceCommandsState::new() {
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
            let command_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));

            // Create shortcut backend for Escape key registration (used by HotkeyIntegration)
            let escape_backend: Arc<dyn hotkey::ShortcutBackend + Send + Sync> =
                Arc::new(hotkey::TauriShortcutBackend::new(app.handle().clone()));

            // Create transcription callback that delegates to TranscriptionService
            let transcription_service_for_callback = transcription_service.clone();
            let transcription_callback: Arc<dyn Fn(String) + Send + Sync> =
                Arc::new(move |file_path: String| {
                    transcription_service_for_callback.process_recording(file_path);
                });

            let mut integration_builder = hotkey::HotkeyIntegration::<
                commands::TauriEventEmitter,
                commands::TauriEventEmitter,
                commands::TauriEventEmitter,
            >::new(recording_emitter)
                .with_app_handle(app.handle().clone())
                .with_audio_thread(audio_thread)
                .with_shared_transcription_model(shared_transcription_model)
                .with_transcription_emitter(emitter)
                .with_recording_state(recording_state.clone())
                .with_listening_state(listening_state)
                .with_command_emitter(command_emitter)
                .with_listening_pipeline(listening_pipeline.clone())
                .with_recording_detectors(recording_detectors.clone())
                .with_shortcut_backend(escape_backend)
                .with_transcription_callback(transcription_callback);

            // Wire up voice command integration if available (still needed for HotkeyIntegration's silence detection callback)
            if let (Some(registry), Some(matcher), Some(dispatcher)) = (command_registry, command_matcher, action_dispatcher) {
                integration_builder = integration_builder
                    .with_command_registry(registry)
                    .with_command_matcher(matcher)
                    .with_action_dispatcher(dispatcher);
                debug!("Voice command integration wired up");
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

            // Register hotkey
            info!("Registering global hotkey (Cmd+Shift+R)...");
            let backend = hotkey::TauriShortcutBackend::new(app.handle().clone());
            let service = hotkey::HotkeyService::new(backend);

            if let Err(e) = service.register_recording_shortcut(Box::new(move || {
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

            // Store service in state for cleanup on exit
            app.manage(service);

            // Create keyboard capture state for shortcut recording with fn key support
            let keyboard_capture = Arc::new(Mutex::new(keyboard_capture::KeyboardCapture::new()));
            app.manage(keyboard_capture);

            info!("Setup complete! Ready to record.");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                debug!("Window destroyed, cleaning up...");
                // Unregister hotkey on window close
                if let Some(service) = window.app_handle().try_state::<HotkeyServiceHandle>() {
                    if let Err(e) = service.unregister_recording_shortcut() {
                        warn!("Failed to unregister hotkey: {:?}", e);
                    } else {
                        debug!("Hotkey unregistered successfully");
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
            commands::open_accessibility_preferences
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


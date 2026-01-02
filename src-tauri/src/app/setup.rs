//! Application setup and initialization.
//!
//! Contains the main setup logic extracted from lib.rs setup closure.

use std::sync::{Arc, Mutex};
use tauri::{App, Manager};
use tauri_plugin_store::StoreExt;

use crate::app::platform::register_hotkey_with_release;
use crate::app::state::HotkeyServiceHandle;
use crate::audio;
use crate::commands;
use crate::dictionary;
use crate::hotkey;
use crate::keyboard_capture;
use crate::model;
use crate::parakeet;
use crate::paths;
use crate::recording;
use crate::shutdown;
use crate::transcription;
use crate::turso;
use crate::voice_commands;
use crate::wake_handler;
use crate::window_context;
use crate::worktree;

/// Main application setup function.
///
/// Initializes all application state, services, and integrations.
/// This is called from the Tauri setup hook.
pub fn setup(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    crate::info!("Setting up heycat...");

    // Handle Ctrl+C to prevent paste during terminal termination
    setup_signal_handlers(app)?;

    // Detect worktree context for data isolation
    let worktree_context = setup_worktree(app)?;
    let settings_file = app
        .try_state::<worktree::WorktreeState>()
        .map(|s| s.settings_file_name())
        .unwrap_or_else(|| worktree::DEFAULT_SETTINGS_FILE.to_string());

    // Set dynamic window title based on worktree context
    setup_window_title(app, &worktree_context, &settings_file);

    // Check for collision with another running instance
    check_instance_collision(&worktree_context)?;

    // Create lock file for this instance
    create_instance_lock(&worktree_context);

    // Initialize Turso/libsql embedded database client
    let turso_client = setup_turso_database(&worktree_context)?;
    app.manage(turso_client.clone());

    // Create shared state for recording manager
    let recording_state = Arc::new(Mutex::new(recording::RecordingManager::new()));
    app.manage(recording_state.clone());

    // Create and manage audio monitor state for device testing
    let audio_monitor = Arc::new(audio::AudioMonitorHandle::spawn());
    app.manage(audio_monitor.clone());

    // Pre-initialize the audio engine at startup
    setup_audio_engine(app, &worktree_context, &audio_monitor);

    // Create shared transcription model (single ~3GB Parakeet model)
    crate::debug!("Creating SharedTranscriptionModel...");
    let shared_transcription_model = Arc::new(parakeet::SharedTranscriptionModel::new());

    // Create and manage recording detectors (for silence detection during recording)
    let recordings_dir = paths::get_recordings_dir(worktree_context.as_ref())
        .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat").join("recordings"));
    let recording_detectors = Arc::new(Mutex::new(
        recording::RecordingDetectors::with_recordings_dir(recordings_dir.clone()),
    ));
    app.manage(recording_detectors.clone());

    // Create audio thread
    crate::debug!("Creating audio thread...");
    let audio_thread = Arc::new(audio::AudioThreadHandle::spawn());
    crate::debug!("Audio thread spawned");
    app.manage(audio_thread.clone());

    // Manage shared transcription model for Tauri commands
    app.manage(shared_transcription_model.clone());

    // Create and manage voice command executor and registry
    let (command_matcher, action_dispatcher) = setup_voice_commands(app)?;

    // Eager model loading at startup (if models exist)
    load_transcription_model(app, &shared_transcription_model);

    // Create RecordingTranscriptionService for unified transcription flow
    let transcription_service = setup_transcription_service(
        app,
        &turso_client,
        &shared_transcription_model,
        &recording_state,
        &command_matcher,
        action_dispatcher.as_ref(),
    )?;
    app.manage(transcription_service.clone());
    crate::debug!("RecordingTranscriptionService created and managed");

    // Create SINGLE shared shortcut backend for all hotkeys
    let shared_backend = hotkey::create_shortcut_backend(app.handle().clone());

    // Create hotkey integration
    let integration = setup_hotkey_integration(
        app,
        &turso_client,
        &shared_transcription_model,
        &recording_state,
        &recording_detectors,
        &recordings_dir,
        &audio_thread,
        &audio_monitor,
        &transcription_service,
        &command_matcher,
        action_dispatcher,
        shared_backend.clone(),
    )?;
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

    // Reuse shared_backend for main hotkey registration
    let service: HotkeyServiceHandle = hotkey::HotkeyServiceDyn::new(shared_backend.clone());

    if let Some(shortcut) = saved_shortcut {
        let registered = register_hotkey_with_release(
            &service,
            &shortcut,
            integration.clone(),
            recording_state.clone(),
            app.handle().clone(),
            recording_mode,
        );

        if !registered {
            crate::warn!("Backend doesn't support key release detection");
            crate::warn!("Application will continue without global hotkey support");
        }
    } else {
        crate::info!("No recording shortcut configured - user will set one during onboarding");
    }

    // Store service in state for cleanup on exit
    app.manage(service);

    // Create keyboard capture state for shortcut recording with fn key support
    let keyboard_capture = Arc::new(Mutex::new(keyboard_capture::KeyboardCapture::new()));
    app.manage(keyboard_capture);

    // Manage window monitor for Tauri commands
    let window_monitor = setup_window_monitor(app, &turso_client)?;
    app.manage(window_monitor);

    // Ensure the app is activated on macOS so the UI receives events immediately
    crate::activation::activate_app();

    crate::info!("Setup complete! Ready to record.");
    Ok(())
}

/// Set up signal handlers for graceful shutdown.
fn setup_signal_handlers(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    shutdown::register_app_handle(app.handle().clone());
    if let Err(e) = ctrlc::set_handler(|| {
        shutdown::signal_shutdown();
        shutdown::stop_cgeventtap();
        shutdown::request_app_exit(0);
    }) {
        crate::warn!("Failed to set Ctrl+C handler: {}", e);
    }
    Ok(())
}

/// Set up worktree context for data isolation.
fn setup_worktree(app: &App) -> Result<Option<worktree::WorktreeContext>, Box<dyn std::error::Error>> {
    let worktree_context = worktree::detect_worktree();
    let worktree_state = worktree::WorktreeState {
        context: worktree_context.clone(),
    };
    let settings_file = worktree_state.settings_file_name();

    if let Some(ref ctx) = worktree_context {
        crate::info!(
            "Running in worktree: {} (gitdir: {:?})",
            ctx.identifier,
            ctx.gitdir_path
        );
        crate::info!("Using worktree-specific settings file: {}", settings_file);
    } else {
        crate::info!("Running in main repository");
    }

    app.manage(worktree_state);
    Ok(worktree_context)
}

/// Set dynamic window title based on worktree context.
fn setup_window_title(
    app: &App,
    worktree_context: &Option<worktree::WorktreeContext>,
    settings_file: &str,
) {
    if let Some(window) = app.get_webview_window("main") {
        let title = match worktree_context {
            Some(ctx) if cfg!(debug_assertions) => {
                let recording_shortcut = app
                    .store(settings_file)
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
            crate::warn!("Failed to set window title: {}", e);
        } else {
            crate::debug!("Window title set to: {}", title);
        }
    }
}

/// Check for collision with another running instance.
fn check_instance_collision(
    worktree_context: &Option<worktree::WorktreeContext>,
) -> Result<(), Box<dyn std::error::Error>> {
    let collision_result = worktree::check_collision(worktree_context.as_ref());
    match &collision_result {
        Ok(worktree::CollisionResult::NoCollision) => {
            crate::debug!("No collision detected, proceeding with startup");
        }
        Ok(collision @ worktree::CollisionResult::InstanceRunning { .. }) => {
            if let Some((title, message, steps)) = worktree::format_collision_error(collision) {
                crate::error!("{}: {}", title, message);
                for step in &steps {
                    crate::warn!("Resolution: {}", step);
                }
                return Err(message.into());
            }
        }
        Ok(collision @ worktree::CollisionResult::StaleLock { lock_file }) => {
            if let Some((title, message, _)) = worktree::format_collision_error(collision) {
                crate::warn!("{}: {}", title, message);
            }
            crate::info!("Cleaning up stale lock file from crashed instance...");
            if let Err(e) = worktree::cleanup_stale_lock(lock_file) {
                crate::warn!("Failed to clean up stale lock file: {}", e);
            } else {
                crate::info!("Stale lock file cleaned up successfully");
            }
        }
        Err(e) => {
            crate::warn!("Failed to check for collisions: {}", e);
        }
    }
    Ok(())
}

/// Create lock file for this instance.
fn create_instance_lock(worktree_context: &Option<worktree::WorktreeContext>) {
    match worktree::create_lock(worktree_context.as_ref()) {
        Ok(lock_path) => {
            crate::debug!("Lock file created: {:?}", lock_path);
        }
        Err(e) => {
            crate::warn!("Failed to create lock file: {}", e);
        }
    }
}

/// Initialize Turso/libsql embedded database client.
fn setup_turso_database(
    worktree_context: &Option<worktree::WorktreeContext>,
) -> Result<Arc<turso::TursoClient>, Box<dyn std::error::Error>> {
    let turso_data_dir = paths::get_data_dir(worktree_context.as_ref())
        .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat"));

    let client = turso::TursoClient::new_blocking(turso_data_dir)?;
    crate::info!("Turso database initialized at: {:?}", client.db_path());

    // Initialize database schema
    let schema_result = match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            tokio::task::block_in_place(|| handle.block_on(turso::initialize_schema(&client)))
        }
        Err(_) => {
            let rt =
                tokio::runtime::Runtime::new().expect("Failed to create tokio runtime for schema init");
            rt.block_on(turso::initialize_schema(&client))
        }
    };

    match schema_result {
        Ok(()) => {
            crate::debug!("Turso database schema initialized");
        }
        Err(e) => {
            crate::error!("Failed to initialize Turso schema: {}", e);
            return Err(format!("Turso schema initialization failed: {}", e).into());
        }
    }

    Ok(Arc::new(client))
}

/// Pre-initialize the audio engine at startup.
fn setup_audio_engine(
    app: &App,
    worktree_context: &Option<worktree::WorktreeContext>,
    audio_monitor: &Arc<audio::AudioMonitorHandle>,
) {
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
        crate::debug!("Pre-initializing audio engine with saved device: {}", device);
    }

    if let Err(e) = audio_monitor.init(saved_device) {
        crate::warn!(
            "Failed to pre-initialize audio engine: {} (will initialize lazily)",
            e
        );
    } else {
        crate::info!("Audio engine pre-initialized and running");
    }
}

/// Set up voice command executor and registry.
fn setup_voice_commands(
    app: &App,
) -> Result<
    (
        Arc<voice_commands::matcher::CommandMatcher>,
        Option<Arc<voice_commands::executor::ActionDispatcher>>,
    ),
    Box<dyn std::error::Error>,
> {
    crate::debug!("Creating voice command infrastructure...");
    let executor_state = voice_commands::executor::ExecutorState::new();
    let dispatcher = executor_state.dispatcher.clone();
    app.manage(executor_state);

    let command_matcher = Arc::new(voice_commands::matcher::CommandMatcher::new());
    crate::debug!("Voice command infrastructure initialized");

    Ok((command_matcher, Some(dispatcher)))
}

/// Load transcription model at startup if available.
fn load_transcription_model(app: &App, shared_model: &Arc<parakeet::SharedTranscriptionModel>) {
    if let Ok(true) = model::check_model_exists_for_type(model::download::ModelType::ParakeetTDT) {
        if let Ok(model_dir) = model::download::get_model_dir(model::download::ModelType::ParakeetTDT)
        {
            crate::info!("Loading shared Parakeet TDT model from {:?}...", model_dir);
            match shared_model.load(&model_dir) {
                Ok(()) => {
                    crate::info!(
                        "Shared Parakeet TDT model loaded successfully (saves ~3GB by sharing)"
                    );

                    wake_handler::init_wake_handler(
                        app.handle().clone(),
                        (**shared_model).clone(),
                        model_dir,
                    );
                }
                Err(e) => crate::warn!("Failed to load Parakeet TDT model: {}", e),
            }
        }
    } else {
        crate::info!(
            "TDT model not found, batch transcription and wake word detection will require download first"
        );
    }
}

/// Set up the RecordingTranscriptionService.
fn setup_transcription_service(
    app: &App,
    turso_client: &Arc<turso::TursoClient>,
    shared_model: &Arc<parakeet::SharedTranscriptionModel>,
    recording_state: &Arc<Mutex<recording::RecordingManager>>,
    command_matcher: &Arc<voice_commands::matcher::CommandMatcher>,
    action_dispatcher: Option<&Arc<voice_commands::executor::ActionDispatcher>>,
) -> Result<Arc<transcription::RecordingTranscriptionService<commands::TauriEventEmitter, commands::TauriEventEmitter>>, Box<dyn std::error::Error>> {
    crate::debug!("Creating RecordingTranscriptionService...");
    let transcription_service_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
    let mut transcription_service = transcription::RecordingTranscriptionService::new(
        shared_model.clone(),
        transcription_service_emitter,
        recording_state.clone(),
        app.handle().clone(),
    );

    // Start window monitor for context-sensitive commands
    crate::debug!("Starting window monitor...");
    let window_monitor = {
        let mut monitor = window_context::WindowMonitor::new();
        if let Err(e) = monitor.start(app.handle().clone(), turso_client.clone()) {
            crate::warn!("Failed to start window monitor: {}", e);
        }
        Arc::new(Mutex::new(monitor))
    };

    // Create context resolver for window-aware command/dictionary resolution
    crate::debug!("Creating context resolver...");
    let context_resolver = Arc::new(window_context::ContextResolver::new(
        window_monitor.clone(),
        turso_client.clone(),
    ));

    // Create expander for transcription service from dictionary entries
    {
        let entries = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                tokio::task::block_in_place(|| handle.block_on(turso_client.list_dictionary_entries()))
            }
            Err(_) => {
                let rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create runtime for dictionary entries");
                rt.block_on(turso_client.list_dictionary_entries())
            }
        };

        match entries {
            Ok(entries) => {
                if entries.is_empty() {
                    crate::debug!("No dictionary entries loaded");
                } else {
                    crate::info!("Loaded {} dictionary entries for expansion", entries.len());
                    let expander = dictionary::DictionaryExpander::new(&entries);
                    transcription_service = transcription_service.with_dictionary_expander(expander);
                    crate::debug!("Dictionary expander wired to TranscriptionService");
                }
            }
            Err(e) => {
                crate::warn!("Failed to load dictionary entries: {}", e);
            }
        }
    }

    // Wire up voice command integration
    if let Some(dispatcher) = action_dispatcher {
        let service_command_emitter =
            Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
        transcription_service = transcription_service
            .with_turso_client(turso_client.clone())
            .with_command_matcher(command_matcher.clone())
            .with_action_dispatcher(dispatcher.clone())
            .with_command_emitter(service_command_emitter);
        crate::debug!("Voice commands wired to TranscriptionService");
    }

    // Wire context resolver
    transcription_service = transcription_service.with_context_resolver(context_resolver);
    crate::debug!("Context resolver wired to TranscriptionService");

    Ok(Arc::new(transcription_service))
}

/// Set up the HotkeyIntegration.
#[allow(clippy::too_many_arguments)]
fn setup_hotkey_integration(
    app: &App,
    turso_client: &Arc<turso::TursoClient>,
    shared_model: &Arc<parakeet::SharedTranscriptionModel>,
    recording_state: &Arc<Mutex<recording::RecordingManager>>,
    recording_detectors: &Arc<Mutex<recording::RecordingDetectors>>,
    recordings_dir: &std::path::PathBuf,
    audio_thread: &Arc<audio::AudioThreadHandle>,
    audio_monitor: &Arc<audio::AudioMonitorHandle>,
    transcription_service: &Arc<transcription::RecordingTranscriptionService<commands::TauriEventEmitter, commands::TauriEventEmitter>>,
    command_matcher: &Arc<voice_commands::matcher::CommandMatcher>,
    action_dispatcher: Option<Arc<voice_commands::executor::ActionDispatcher>>,
    shared_backend: Arc<dyn hotkey::ShortcutBackend + Send + Sync>,
) -> Result<Arc<Mutex<hotkey::HotkeyIntegration<commands::TauriEventEmitter, commands::TauriEventEmitter, commands::TauriEventEmitter>>>, Box<dyn std::error::Error>> {
    let emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
    let recording_emitter = commands::TauriEventEmitter::new(app.handle().clone());

    // Create transcription callback
    let transcription_service_for_callback = transcription_service.clone();
    let transcription_callback: Arc<dyn Fn(String) + Send + Sync> =
        Arc::new(move |file_path: String| {
            transcription_service_for_callback.process_recording(file_path);
        });

    // Create hotkey event emitter
    let hotkey_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));

    let mut integration_builder = hotkey::HotkeyIntegration::<
        commands::TauriEventEmitter,
        commands::TauriEventEmitter,
        commands::TauriEventEmitter,
    >::new(recording_emitter)
    .with_app_handle(app.handle().clone())
    .with_audio_thread(audio_thread.clone())
    .with_audio_monitor(audio_monitor.clone())
    .with_shared_transcription_model(shared_model.clone())
    .with_transcription_emitter(emitter)
    .with_recording_state(recording_state.clone())
    .with_recording_detectors(recording_detectors.clone())
    .with_recordings_dir(recordings_dir.clone())
    .with_shortcut_backend(shared_backend)
    .with_transcription_callback(transcription_callback)
    .with_hotkey_emitter(hotkey_emitter)
    .with_silence_detection_enabled(false);

    // Wire up voice command integration
    if let Some(dispatcher) = action_dispatcher {
        let command_emitter = Arc::new(commands::TauriEventEmitter::new(app.handle().clone()));
        integration_builder = integration_builder.with_voice_commands(hotkey::integration::VoiceCommandConfig {
            turso_client: turso_client.clone(),
            matcher: command_matcher.clone(),
            dispatcher,
            emitter: Some(command_emitter),
        });
        crate::debug!("Voice command integration wired up using grouped config");
    }

    let integration = Arc::new(Mutex::new(integration_builder));

    // Set up escape callback
    {
        let integration_for_escape = integration.clone();
        let state_for_escape = recording_state.clone();
        let escape_callback: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
            crate::debug!("Double-tap Escape detected, cancelling recording");
            if let Ok(mut guard) = integration_for_escape.lock() {
                guard.cancel_recording(&state_for_escape, "double-tap-escape");
            } else {
                crate::error!("Failed to acquire integration lock for cancel");
            }
        });

        if let Ok(mut guard) = integration.lock() {
            guard.set_escape_callback(escape_callback);
            crate::debug!("Escape callback wired up for recording cancellation");
        }
    }

    Ok(integration)
}

/// Set up the window monitor for context-sensitive commands.
fn setup_window_monitor(
    app: &App,
    turso_client: &Arc<turso::TursoClient>,
) -> Result<Arc<Mutex<window_context::WindowMonitor>>, Box<dyn std::error::Error>> {
    crate::debug!("Starting window monitor...");
    let window_monitor = {
        let mut monitor = window_context::WindowMonitor::new();
        if let Err(e) = monitor.start(app.handle().clone(), turso_client.clone()) {
            crate::warn!("Failed to start window monitor: {}", e);
        }
        Arc::new(Mutex::new(monitor))
    };
    Ok(window_monitor)
}

/// Handle window destroyed event for cleanup.
pub fn on_window_destroyed(window: &tauri::Window) {
    // Only trigger shutdown when the main window is destroyed
    // (not for splash or other transient windows)
    if window.label() != "main" {
        crate::debug!(
            "Non-main window '{}' destroyed, skipping cleanup",
            window.label()
        );
        return;
    }

    // Signal shutdown FIRST - prevents async tasks from pasting during cleanup
    shutdown::signal_shutdown();

    crate::debug!("Main window destroyed, cleaning up...");

    // Get worktree context for cleanup
    let worktree_context = window
        .app_handle()
        .try_state::<worktree::WorktreeState>()
        .and_then(|s| s.context.clone());

    // Clean up lock file on graceful shutdown
    if let Err(e) = worktree::remove_lock(worktree_context.as_ref()) {
        crate::warn!("Failed to remove lock file: {}", e);
    } else {
        crate::debug!("Lock file removed successfully");
    }

    // Unregister hotkey on window close
    if let Some(service) = window.app_handle().try_state::<HotkeyServiceHandle>() {
        let settings_file = window
            .app_handle()
            .try_state::<worktree::WorktreeState>()
            .map(|s| s.settings_file_name())
            .unwrap_or_else(|| worktree::DEFAULT_SETTINGS_FILE.to_string());

        if let Some(shortcut) = window
            .app_handle()
            .store(&settings_file)
            .ok()
            .and_then(|store| store.get("hotkey.recordingShortcut"))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
        {
            if let Err(e) = service.backend.unregister(&shortcut) {
                crate::warn!("Failed to unregister hotkey '{}': {}", shortcut, e);
            } else {
                crate::debug!("Hotkey '{}' unregistered successfully", shortcut);
            }
        }
    }

    // Stop window monitor on window close
    if let Some(monitor) = window
        .app_handle()
        .try_state::<Arc<Mutex<window_context::WindowMonitor>>>()
    {
        if let Ok(mut monitor) = monitor.lock() {
            if let Err(e) = monitor.stop() {
                crate::warn!("Failed to stop window monitor: {}", e);
            } else {
                crate::debug!("Window monitor stopped successfully");
            }
        }
    }
}

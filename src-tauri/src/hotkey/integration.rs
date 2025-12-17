// Hotkey-to-recording integration module
// Connects global hotkey to recording state with debouncing
// Uses unified command implementations for start/stop logic

use crate::audio::AudioThreadHandle;
use crate::commands::logic::{start_recording_impl, stop_recording_impl};
use crate::events::{
    current_timestamp, CommandAmbiguousPayload, CommandCandidate, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload, ListeningEventEmitter,
    RecordingCancelledPayload, RecordingErrorPayload, RecordingEventEmitter,
    RecordingStartedPayload, RecordingStoppedPayload, TranscriptionCompletedPayload,
    TranscriptionErrorPayload, TranscriptionEventEmitter, TranscriptionStartedPayload,
};
use crate::hotkey::double_tap::{DoubleTapDetector, DEFAULT_DOUBLE_TAP_WINDOW_MS};
use crate::hotkey::ShortcutBackend;
use crate::listening::{ListeningManager, ListeningPipeline, RecordingDetectors, SilenceConfig};
use crate::model::{check_model_exists_for_type, ModelType};
use crate::recording::{RecordingManager, RecordingState};
use crate::voice_commands::executor::ActionDispatcher;
use crate::voice_commands::matcher::{CommandMatcher, MatchResult};
use crate::voice_commands::registry::CommandRegistry;
use crate::parakeet::{SharedTranscriptionModel, TranscriptionService};
use crate::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::sync::Semaphore;

/// Maximum concurrent transcriptions allowed
const MAX_CONCURRENT_TRANSCRIPTIONS: usize = 2;

/// Default transcription timeout in seconds
/// If transcription takes longer than this, it will be cancelled and an error emitted
pub const DEFAULT_TRANSCRIPTION_TIMEOUT_SECS: u64 = 60;

/// Debounce duration for hotkey presses (200ms)
pub const DEBOUNCE_DURATION_MS: u64 = 200;

/// Simulate Cmd+V paste keystroke on macOS using CoreGraphics
#[cfg(target_os = "macos")]
fn simulate_paste() -> Result<(), String> {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| "Failed to create event source")?;

    // V key = keycode 9
    let key_v: CGKeyCode = 9;

    // Key down with Command modifier
    let event_down = CGEvent::new_keyboard_event(source.clone(), key_v, true)
        .map_err(|_| "Failed to create key down event")?;
    event_down.set_flags(CGEventFlags::CGEventFlagCommand);
    event_down.post(CGEventTapLocation::HID);

    // Small delay for event processing
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Key up
    let event_up = CGEvent::new_keyboard_event(source, key_v, false)
        .map_err(|_| "Failed to create key up event")?;
    event_up.set_flags(CGEventFlags::CGEventFlagCommand);
    event_up.post(CGEventTapLocation::HID);

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn simulate_paste() -> Result<(), String> {
    Err("Paste simulation only supported on macOS".to_string())
}

/// Handles hotkey toggle with debouncing and event emission
pub struct HotkeyIntegration<R: RecordingEventEmitter, T: TranscriptionEventEmitter + ListeningEventEmitter, C: CommandEventEmitter> {
    last_toggle_time: Option<Instant>,
    debounce_duration: Duration,
    recording_emitter: R,
    /// Optional audio thread handle - when present, starts/stops capture on toggle
    audio_thread: Option<Arc<AudioThreadHandle>>,
    /// Optional SharedTranscriptionModel for auto-transcription after recording stops
    shared_transcription_model: Option<Arc<SharedTranscriptionModel>>,
    /// Transcription event emitter for emitting events from spawned thread
    transcription_emitter: Option<Arc<T>>,
    /// Reference to recording state for getting audio buffer in transcription thread
    recording_state: Option<Arc<Mutex<RecordingManager>>>,
    /// Optional listening state for determining return state after recording
    listening_state: Option<Arc<Mutex<ListeningManager>>>,
    /// Optional command registry for voice command matching
    command_registry: Option<Arc<Mutex<CommandRegistry>>>,
    /// Optional command matcher for voice command matching
    command_matcher: Option<Arc<CommandMatcher>>,
    /// Optional action dispatcher for executing matched commands
    action_dispatcher: Option<Arc<ActionDispatcher>>,
    /// Optional command event emitter for voice command events
    command_emitter: Option<Arc<C>>,
    /// Semaphore to limit concurrent transcriptions
    transcription_semaphore: Arc<Semaphore>,
    /// Optional app handle for clipboard access
    app_handle: Option<AppHandle>,
    /// Optional listening pipeline for restarting wake word detection after recording
    listening_pipeline: Option<Arc<Mutex<ListeningPipeline>>>,
    /// Transcription timeout duration (defaults to 60 seconds)
    transcription_timeout: Duration,
    /// Recording detectors for silence-based auto-stop (shared with wake word flow)
    recording_detectors: Option<Arc<Mutex<RecordingDetectors>>>,
    /// Whether to enable silence detection for hotkey recordings (defaults to true)
    pub(crate) silence_detection_enabled: bool,
    /// Custom silence configuration for hotkey recordings (optional)
    pub(crate) silence_config: Option<SilenceConfig>,
    /// Optional shortcut backend for Escape key registration (boxed to avoid generic parameter)
    shortcut_backend: Option<Arc<dyn ShortcutBackend + Send + Sync>>,
    /// Callback invoked when double-tap Escape is detected during recording
    escape_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Whether Escape key is currently registered (to track cleanup)
    escape_registered: bool,
    /// Time window for double-tap detection (default 300ms)
    double_tap_window_ms: u64,
    /// Double-tap detector for Escape key (created on recording start)
    double_tap_detector: Option<Arc<Mutex<DoubleTapDetector<Box<dyn Fn() + Send + Sync>>>>>,
}

impl<R: RecordingEventEmitter, T: TranscriptionEventEmitter + ListeningEventEmitter + 'static, C: CommandEventEmitter + 'static> HotkeyIntegration<R, T, C> {
    /// Create a new HotkeyIntegration with default debounce duration
    pub fn new(recording_emitter: R) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(DEBOUNCE_DURATION_MS),
            recording_emitter,
            audio_thread: None,
            shared_transcription_model: None,
            transcription_emitter: None,
            recording_state: None,
            listening_state: None,
            command_registry: None,
            command_matcher: None,
            action_dispatcher: None,
            command_emitter: None,
            transcription_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
            app_handle: None,
            listening_pipeline: None,
            transcription_timeout: Duration::from_secs(DEFAULT_TRANSCRIPTION_TIMEOUT_SECS),
            recording_detectors: None,
            silence_detection_enabled: true,
            silence_config: None,
            shortcut_backend: None,
            escape_callback: None,
            escape_registered: false,
            double_tap_window_ms: DEFAULT_DOUBLE_TAP_WINDOW_MS,
            double_tap_detector: None,
        }
    }

    /// Add app handle for clipboard access (builder pattern)
    pub fn with_app_handle(mut self, handle: AppHandle) -> Self {
        self.app_handle = Some(handle);
        self
    }

    /// Get the selected audio device from persistent settings store
    fn get_selected_audio_device(&self) -> Option<String> {
        use tauri_plugin_store::StoreExt;
        self.app_handle.as_ref().and_then(|app| {
            app.store("settings.json")
                .ok()
                .and_then(|store| store.get("audio.selectedDevice"))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        })
    }

    /// Add an audio thread handle (builder pattern)
    pub fn with_audio_thread(mut self, handle: Arc<AudioThreadHandle>) -> Self {
        self.audio_thread = Some(handle);
        self
    }

    /// Add SharedTranscriptionModel for auto-transcription (builder pattern)
    pub fn with_shared_transcription_model(mut self, model: Arc<SharedTranscriptionModel>) -> Self {
        self.shared_transcription_model = Some(model);
        self
    }

    /// Add transcription event emitter for emitting events from spawned thread (builder pattern)
    pub fn with_transcription_emitter(mut self, emitter: Arc<T>) -> Self {
        self.transcription_emitter = Some(emitter);
        self
    }

    /// Add recording state reference for transcription thread (builder pattern)
    pub fn with_recording_state(mut self, state: Arc<Mutex<RecordingManager>>) -> Self {
        self.recording_state = Some(state);
        self
    }

    /// Add listening state reference for determining return state after recording (builder pattern)
    pub fn with_listening_state(mut self, state: Arc<Mutex<ListeningManager>>) -> Self {
        self.listening_state = Some(state);
        self
    }

    /// Add voice command registry for command matching (builder pattern)
    pub fn with_command_registry(mut self, registry: Arc<Mutex<CommandRegistry>>) -> Self {
        self.command_registry = Some(registry);
        self
    }

    /// Add command matcher for voice command matching (builder pattern)
    pub fn with_command_matcher(mut self, matcher: Arc<CommandMatcher>) -> Self {
        self.command_matcher = Some(matcher);
        self
    }

    /// Add action dispatcher for executing matched commands (builder pattern)
    pub fn with_action_dispatcher(mut self, dispatcher: Arc<ActionDispatcher>) -> Self {
        self.action_dispatcher = Some(dispatcher);
        self
    }

    /// Add command event emitter for voice command events (builder pattern)
    pub fn with_command_emitter(mut self, emitter: Arc<C>) -> Self {
        self.command_emitter = Some(emitter);
        self
    }

    /// Add listening pipeline for restarting after hotkey recording (builder pattern)
    pub fn with_listening_pipeline(mut self, pipeline: Arc<Mutex<ListeningPipeline>>) -> Self {
        self.listening_pipeline = Some(pipeline);
        self
    }

    /// Set custom transcription timeout duration (builder pattern)
    ///
    /// Default is 60 seconds. If transcription takes longer than this, it will be
    /// cancelled and a timeout error will be emitted to the frontend.
    #[allow(dead_code)]
    pub fn with_transcription_timeout(mut self, timeout: Duration) -> Self {
        self.transcription_timeout = timeout;
        self
    }

    /// Add recording detectors for silence-based auto-stop (builder pattern)
    ///
    /// When configured, hotkey recordings will automatically stop when silence is
    /// detected (after speech ends). This shares the same RecordingDetectors state
    /// used by the wake word flow for consistency.
    pub fn with_recording_detectors(mut self, detectors: Arc<Mutex<RecordingDetectors>>) -> Self {
        self.recording_detectors = Some(detectors);
        self
    }

    /// Enable or disable silence detection for hotkey recordings (builder pattern)
    ///
    /// Default is true (enabled). When disabled, hotkey recordings will only stop
    /// when the user manually triggers the hotkey again.
    #[allow(dead_code)]
    pub fn with_silence_detection_enabled(mut self, enabled: bool) -> Self {
        self.silence_detection_enabled = enabled;
        self
    }

    /// Set custom silence configuration for hotkey recordings (builder pattern)
    ///
    /// If not set, uses the default SilenceConfig. This allows customizing
    /// silence duration thresholds, no-speech timeouts, etc.
    #[allow(dead_code)]
    pub fn with_silence_config(mut self, config: SilenceConfig) -> Self {
        self.silence_config = Some(config);
        self
    }

    /// Add shortcut backend for Escape key registration (builder pattern)
    ///
    /// When configured with an escape callback, the Escape key listener will be
    /// automatically registered when recording starts and unregistered when
    /// recording stops. This enables cancel functionality via Escape key.
    pub fn with_shortcut_backend(mut self, backend: Arc<dyn ShortcutBackend + Send + Sync>) -> Self {
        self.shortcut_backend = Some(backend);
        self
    }

    /// Set the callback to invoke when double-tap Escape is detected during recording (builder pattern)
    ///
    /// The callback will only fire when recording is in progress AND a double-tap
    /// of the Escape key is detected within the configured time window (default 300ms).
    /// Single Escape key presses are ignored. The Escape key listener is automatically
    /// registered when recording starts and unregistered when recording stops.
    ///
    /// Note: Production code uses `set_escape_callback` instead because the callback
    /// needs to capture a reference to the integration after it's wrapped in Arc<Mutex<>>.
    /// This builder method is kept for test ergonomics.
    #[allow(dead_code)]
    pub fn with_escape_callback(mut self, callback: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.escape_callback = Some(callback);
        self
    }

    /// Set the time window for double-tap detection (builder pattern)
    ///
    /// Default is 300ms. Two Escape key presses within this window will trigger
    /// the cancel callback. Single presses are ignored.
    #[allow(dead_code)]
    pub fn with_double_tap_window(mut self, window_ms: u64) -> Self {
        self.double_tap_window_ms = window_ms;
        self
    }

    /// Set the escape callback after construction
    ///
    /// This allows setting the callback after the integration is wrapped in Arc<Mutex<>>,
    /// which is necessary when the callback needs to capture a reference to the integration
    /// itself (for calling cancel_recording).
    pub fn set_escape_callback(&mut self, callback: Arc<dyn Fn() + Send + Sync>) {
        self.escape_callback = Some(callback);
    }

    /// Create with custom debounce duration (for testing)
    #[cfg(test)]
    pub fn with_debounce(recording_emitter: R, debounce_ms: u64) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(debounce_ms),
            recording_emitter,
            audio_thread: None,
            shared_transcription_model: None,
            transcription_emitter: None,
            recording_state: None,
            listening_state: None,
            command_registry: None,
            command_matcher: None,
            action_dispatcher: None,
            command_emitter: None,
            transcription_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
            app_handle: None,
            listening_pipeline: None,
            transcription_timeout: Duration::from_secs(DEFAULT_TRANSCRIPTION_TIMEOUT_SECS),
            recording_detectors: None,
            silence_detection_enabled: true,
            silence_config: None,
            shortcut_backend: None,
            escape_callback: None,
            escape_registered: false,
            double_tap_window_ms: DEFAULT_DOUBLE_TAP_WINDOW_MS,
            double_tap_detector: None,
        }
    }

    /// Handle hotkey toggle - debounces rapid presses
    ///
    /// Toggles recording state (Idle → Recording → Idle) and emits events.
    /// Delegates to unified command implementations for start/stop logic.
    ///
    /// Returns true if the toggle was accepted, false if debounced or busy
    ///
    /// Coverage exclusion: Error paths (lock poisoning, command failures) cannot
    /// be triggered without mocking std::sync primitives. The happy path is tested
    /// via integration_test.rs with mock emitters.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn handle_toggle(&mut self, state: &Mutex<RecordingManager>) -> bool {
        let now = Instant::now();

        // Check debounce
        if let Some(last) = self.last_toggle_time {
            if now.duration_since(last) < self.debounce_duration {
                trace!("Toggle debounced");
                return false;
            }
        }

        self.last_toggle_time = Some(now);

        // Check current state to decide action
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                error!("Failed to acquire lock: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        debug!("Toggle received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Idle | RecordingState::Listening => {
                info!("Starting recording from {:?} state...", current_state);

                // If coming from Listening state, stop the pipeline first to prevent zombie
                // (analysis thread running with orphaned audio buffer)
                if current_state == RecordingState::Listening {
                    self.stop_listening_pipeline_for_hotkey();
                }

                // Check model availability (TDT for batch transcription)
                let model_available = check_model_exists_for_type(ModelType::ParakeetTDT).unwrap_or(false);

                // Use unified command implementation
                // Read selected device from persistent settings store
                let device_name = self.get_selected_audio_device();
                match start_recording_impl(state, self.audio_thread.as_deref(), model_available, device_name) {
                    Ok(()) => {
                        self.recording_emitter
                            .emit_recording_started(RecordingStartedPayload {
                                timestamp: current_timestamp(),
                            });
                        info!("Recording started, emitted recording_started event");

                        // Register Escape key listener for cancel functionality
                        self.register_escape_listener();

                        // Start silence detection if enabled and configured
                        self.start_silence_detection(state);

                        true
                    }
                    Err(e) => {
                        error!("Failed to start recording: {}", e);
                        self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Recording => {
                info!("Stopping recording (manual stop via hotkey)...");

                // Unregister Escape key listener first
                self.unregister_escape_listener();

                // Stop silence detection first to prevent it from interfering
                // Manual stop takes precedence over auto-stop
                self.stop_silence_detection();

                // Check if listening mode is enabled to determine return state
                let return_to_listening = self
                    .listening_state
                    .as_ref()
                    .and_then(|ls| ls.lock().ok())
                    .map(|lm| lm.is_enabled())
                    .unwrap_or(false);

                // Use unified command implementation
                match stop_recording_impl(state, self.audio_thread.as_deref(), return_to_listening) {
                    Ok(metadata) => {
                        info!(
                            "Recording stopped: {} samples, {:.2}s duration",
                            metadata.sample_count, metadata.duration_secs
                        );
                        // Clone file_path before metadata is moved
                        let file_path_for_transcription = metadata.file_path.clone();
                        self.recording_emitter
                            .emit_recording_stopped(RecordingStoppedPayload { metadata });
                        debug!("Emitted recording_stopped event");

                        // Auto-transcribe if transcription manager is configured
                        self.spawn_transcription(file_path_for_transcription);

                        // Restart listening pipeline if returning to listening mode
                        if return_to_listening {
                            self.restart_listening_pipeline();
                        }

                        true
                    }
                    Err(e) => {
                        error!("Failed to stop recording: {}", e);
                        self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Processing => {
                // In Processing state - ignore toggle (busy)
                debug!("Toggle ignored - already processing");
                false
            }
        }
    }

    /// Spawn transcription as an async task
    ///
    /// Transcribes the WAV file, tries command matching, then fallback to clipboard.
    /// Uses Tauri's async runtime for bounded async execution.
    /// No-op if transcription manager or transcription emitter is not configured.
    ///
    /// This method is public so it can be called from the wake word recording flow
    /// (via the coordinator) in addition to the hotkey recording flow.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn spawn_transcription(&self, file_path: String) {
        // Check all required components are present
        let shared_model = match &self.shared_transcription_model {
            Some(m) => m.clone(),
            None => {
                debug!("Transcription skipped: no shared transcription model configured");
                return;
            }
        };

        let transcription_emitter = match &self.transcription_emitter {
            Some(te) => te.clone(),
            None => {
                debug!("Transcription skipped: no transcription emitter configured");
                return;
            }
        };

        // Optional voice command components
        let command_registry = self.command_registry.clone();
        let command_matcher = self.command_matcher.clone();
        let action_dispatcher = self.action_dispatcher.clone();
        let command_emitter = self.command_emitter.clone();

        // Clone app_handle for clipboard access
        let app_handle = self.app_handle.clone();

        // Clone recording_state for buffer cleanup after transcription
        let recording_state = self.recording_state.clone();

        // Check if model is loaded
        if !shared_model.is_loaded() {
            info!("Transcription skipped: transcription model not loaded");
            return;
        }

        // Clone semaphore for the async task
        let semaphore = self.transcription_semaphore.clone();

        // Clone timeout for the async task
        let timeout_duration = self.transcription_timeout;

        info!("Spawning transcription task...");

        // Spawn async task using Tauri's async runtime
        tauri::async_runtime::spawn(async move {
            // Helper to clear recording buffer - call this in all exit paths to prevent memory leaks
            let clear_recording_buffer = || {
                if let Some(ref state) = recording_state {
                    if let Ok(mut manager) = state.lock() {
                        manager.clear_last_recording();
                        debug!("Cleared recording buffer");
                    }
                }
            };

            // Acquire semaphore permit to limit concurrent transcriptions
            let _permit = match semaphore.try_acquire() {
                Ok(permit) => permit,
                Err(_) => {
                    warn!("Too many concurrent transcriptions, skipping this one");
                    // Emit error event so the user knows their recording was not processed
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: "Too many transcriptions in progress. Please wait and try again.".to_string(),
                    });
                    clear_recording_buffer();
                    return;
                }
            };

            // Emit transcription_started event
            let start_time = Instant::now();
            transcription_emitter.emit_transcription_started(TranscriptionStartedPayload {
                timestamp: current_timestamp(),
            });

            debug!("Transcribing file: {}", file_path);

            // Perform transcription on blocking thread pool (CPU-intensive) with timeout
            let transcriber = shared_model.clone();
            let transcription_future = tokio::task::spawn_blocking(move || {
                transcriber.transcribe(&file_path)
            });

            let transcription_result = tokio::time::timeout(timeout_duration, transcription_future).await;

            let text = match transcription_result {
                Ok(Ok(Ok(text))) => text,
                Ok(Ok(Err(e))) => {
                    error!("Transcription failed: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: e.to_string(),
                    });
                    if let Err(reset_err) = shared_model.reset_to_idle() {
                        warn!("Failed to reset transcription state: {}", reset_err);
                    }
                    clear_recording_buffer();
                    return;
                }
                Ok(Err(e)) => {
                    error!("Transcription task panicked: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: "Internal transcription error.".to_string(),
                    });
                    if let Err(reset_err) = shared_model.reset_to_idle() {
                        warn!("Failed to reset transcription state: {}", reset_err);
                    }
                    clear_recording_buffer();
                    return;
                }
                Err(_) => {
                    // Timeout error
                    error!("Transcription timed out after {:?}", timeout_duration);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: format!("Transcription timed out after {} seconds. The audio may be too long or the model may be stuck.", timeout_duration.as_secs()),
                    });
                    if let Err(reset_err) = shared_model.reset_to_idle() {
                        warn!("Failed to reset transcription state: {}", reset_err);
                    }
                    clear_recording_buffer();
                    return;
                }
            };

            let duration_ms = start_time.elapsed().as_millis() as u64;
            info!(
                "Transcription completed in {}ms: {} chars",
                duration_ms,
                text.len()
            );
            info!("=== spawn_transcription received text ===");
            info!("text content: {:?}", text);
            info!("=== end spawn_transcription text ===");

            // Try voice command matching if configured
            // Note: We extract all data from the lock before any await to ensure Send safety
            // Local enum to capture match results before releasing the registry lock.
            // IMPORTANT: registry_guard must be dropped before any await to ensure
            // this async block remains Send. Holding a MutexGuard across an await
            // point would make it !Send.
            enum MatchOutcome {
                Matched { cmd: crate::voice_commands::registry::CommandDefinition, trigger: String, confidence: f64 },
                Ambiguous { candidates: Vec<CommandCandidate> },
                NoMatch,
            }

            let command_handled = if let (Some(registry), Some(matcher), Some(dispatcher), Some(emitter)) =
                (&command_registry, &command_matcher, &action_dispatcher, &command_emitter)
            {
                // Lock registry, match, extract all needed data, then release lock
                let outcome = {
                    let registry_guard = match registry.lock() {
                        Ok(g) => g,
                        Err(_) => {
                            error!("Failed to lock command registry - lock poisoned");
                            // Emit error event so UI doesn't hang waiting
                            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                                error: "Internal error: command registry unavailable. Please restart the application.".to_string(),
                            });
                            clear_recording_buffer();
                            return;
                        }
                    };

                    let match_result = matcher.match_input(&text, &registry_guard);

                    match match_result {
                        MatchResult::Exact { command: matched_cmd, .. } => {
                            match registry_guard.get(matched_cmd.id).cloned() {
                                Some(cmd) => MatchOutcome::Matched {
                                    cmd,
                                    trigger: matched_cmd.trigger.clone(),
                                    confidence: 1.0
                                },
                                None => MatchOutcome::NoMatch,
                            }
                        }
                        MatchResult::Fuzzy { command: matched_cmd, score, .. } => {
                            match registry_guard.get(matched_cmd.id).cloned() {
                                Some(cmd) => MatchOutcome::Matched {
                                    cmd,
                                    trigger: matched_cmd.trigger.clone(),
                                    confidence: score
                                },
                                None => MatchOutcome::NoMatch,
                            }
                        }
                        MatchResult::Ambiguous { candidates } => {
                            let candidate_data: Vec<_> = candidates
                                .iter()
                                .map(|c| CommandCandidate {
                                    id: c.command.id.to_string(),
                                    trigger: c.command.trigger.clone(),
                                    confidence: c.score,
                                })
                                .collect();
                            MatchOutcome::Ambiguous { candidates: candidate_data }
                        }
                        MatchResult::NoMatch => MatchOutcome::NoMatch,
                    }
                    // registry_guard is dropped here - before any await
                };

                match outcome {
                    MatchOutcome::Matched { cmd, trigger, confidence } => {
                        info!("Command matched: {} (confidence: {:.2})", trigger, confidence);

                        // Emit command_matched event
                        emitter.emit_command_matched(CommandMatchedPayload {
                            transcription: text.clone(),
                            command_id: cmd.id.to_string(),
                            trigger: trigger.clone(),
                            confidence,
                        });

                        // Execute command directly using await (no new runtime needed!)
                        match dispatcher.execute(&cmd).await {
                            Ok(action_result) => {
                                info!("Command executed: {}", action_result.message);
                                emitter.emit_command_executed(CommandExecutedPayload {
                                    command_id: cmd.id.to_string(),
                                    trigger: trigger.clone(),
                                    message: action_result.message,
                                });
                            }
                            Err(action_error) => {
                                error!("Command execution failed: {}", action_error);
                                emitter.emit_command_failed(CommandFailedPayload {
                                    command_id: cmd.id.to_string(),
                                    trigger: trigger.clone(),
                                    error_code: action_error.code.to_string(),
                                    error_message: action_error.message,
                                });
                            }
                        }
                        true // Command was handled
                    }
                    MatchOutcome::Ambiguous { candidates } => {
                        info!("Ambiguous match: {} candidates", candidates.len());

                        // Emit command_ambiguous event for disambiguation UI
                        emitter.emit_command_ambiguous(CommandAmbiguousPayload {
                            transcription: text.clone(),
                            candidates,
                        });
                        true // Command matching was handled (ambiguous)
                    }
                    MatchOutcome::NoMatch => {
                        debug!("No command match for: {}", text);
                        false // Fall through to clipboard
                    }
                }
            } else {
                debug!("Voice commands not configured, skipping command matching");
                false
            };

            // Fallback to clipboard if no command was handled
            if !command_handled {
                // Copy to clipboard using the clipboard plugin
                if let Some(ref handle) = app_handle {
                    if let Err(e) = handle.clipboard().write_text(&text) {
                        warn!("Failed to copy to clipboard: {}", e);
                    } else {
                        debug!("Transcribed text copied to clipboard");
                        // Auto-paste the clipboard content
                        if let Err(e) = simulate_paste() {
                            warn!("Failed to auto-paste: {}", e);
                        } else {
                            debug!("Auto-pasted transcribed text");
                        }
                    }
                } else {
                    warn!("Clipboard unavailable: no app handle configured");
                }
            }

            // Always emit transcription_completed (whether command handled or not)
            // This ensures the frontend clears the "Transcribing..." state
            info!("=== Emitting transcription_completed ===");
            info!("text to emit: {:?}", text);
            info!("=== end emit ===");
            transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                text,
                duration_ms,
            });

            // Reset transcription state to idle
            if let Err(e) = shared_model.reset_to_idle() {
                warn!("Failed to reset transcription state: {}", e);
            }

            // Clear recording buffer to free memory
            clear_recording_buffer();
        });
    }

    /// Stop the listening pipeline before hotkey recording starts
    /// Called when transitioning from Listening -> Recording via hotkey
    /// This prevents the "zombie pipeline" issue where the analysis thread
    /// continues running but the audio buffer is orphaned
    fn stop_listening_pipeline_for_hotkey(&self) {
        let pipeline = match &self.listening_pipeline {
            Some(p) => p,
            None => return,
        };

        let audio_thread = match &self.audio_thread {
            Some(at) => at,
            None => return,
        };

        if let Ok(mut p) = pipeline.lock() {
            if p.is_running() {
                info!("Stopping listening pipeline before hotkey recording...");
                // Note: We don't need the buffer since hotkey recording
                // creates its own fresh buffer (unlike wake word which hands off)
                if let Err(e) = p.stop(audio_thread.as_ref()) {
                    warn!("Failed to stop listening pipeline: {:?}", e);
                    // Continue anyway - recording should still work
                }
            }
        }
    }

    /// Restart the listening pipeline after recording stops
    /// Only restarts if pipeline and audio thread are configured and pipeline isn't running
    fn restart_listening_pipeline(&self) {
        let pipeline = match &self.listening_pipeline {
            Some(p) => p,
            None => {
                debug!("No listening pipeline configured, skipping restart");
                return;
            }
        };

        let audio_thread = match &self.audio_thread {
            Some(at) => at,
            None => {
                debug!("No audio thread configured, cannot restart pipeline");
                return;
            }
        };

        let emitter = match &self.transcription_emitter {
            Some(e) => e.clone(),
            None => {
                debug!("No transcription emitter configured, cannot restart pipeline");
                return;
            }
        };

        let mut p = match pipeline.lock() {
            Ok(guard) => guard,
            Err(_) => {
                warn!("Failed to lock listening pipeline for restart");
                return;
            }
        };

        // Stop if still running (shouldn't happen since we now stop before recording,
        // but keep for safety in case of race conditions or other code paths)
        if p.is_running() {
            info!("Pipeline still running unexpectedly, stopping before restart...");
            match p.stop(audio_thread.as_ref()) {
                Ok(()) => debug!("Pipeline stopped successfully"),
                Err(e) => {
                    // NotRunning error is fine - thread may have exited naturally
                    warn!("Error stopping pipeline: {:?}", e);
                }
            }
        }

        // Small delay to ensure any thread cleanup completes
        // The stop() method joins the thread, but this gives a moment for cleanup
        drop(p); // Release lock during sleep
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Re-acquire lock for restart
        let mut p = match pipeline.lock() {
            Ok(guard) => guard,
            Err(_) => {
                warn!("Failed to re-lock listening pipeline for restart");
                return;
            }
        };

        info!("Restarting listening pipeline after hotkey recording");
        match p.start(audio_thread.as_ref(), emitter) {
            Ok(_) => info!("Listening pipeline restarted successfully"),
            Err(crate::listening::PipelineError::AlreadyRunning) => {
                // Should not happen after stop, but handle gracefully
                warn!("Pipeline reported AlreadyRunning after stop - unexpected state");
            }
            Err(e) => {
                warn!("Failed to restart listening pipeline: {:?}", e);
            }
        }
    }

    /// Start silence detection for hotkey recording
    ///
    /// When silence detection is enabled and all required components are configured,
    /// this starts monitoring the recording audio for silence. The recording will
    /// automatically stop when silence is detected after speech ends.
    ///
    /// This method is called after recording starts successfully. The detection runs
    /// in a separate thread and will handle saving/transcription when silence triggers.
    fn start_silence_detection(&self, recording_state: &Mutex<RecordingManager>) {
        // Check if silence detection is enabled
        if !self.silence_detection_enabled {
            debug!("Silence detection disabled for hotkey recordings");
            return;
        }

        // Check for required components
        let detectors = match &self.recording_detectors {
            Some(d) => d.clone(),
            None => {
                debug!("Recording detectors not configured, skipping silence detection");
                return;
            }
        };

        let audio_thread = match &self.audio_thread {
            Some(at) => at.clone(),
            None => {
                debug!("No audio thread configured, cannot start silence detection");
                return;
            }
        };

        // Verify transcription emitter is configured (needed for the callback)
        if self.transcription_emitter.is_none() {
            debug!("No transcription emitter configured, cannot start silence detection");
            return;
        }

        // Get the audio buffer from recording state
        let buffer = {
            let manager = match recording_state.lock() {
                Ok(m) => m,
                Err(_) => {
                    warn!("Failed to lock recording state for silence detection");
                    return;
                }
            };

            match manager.get_audio_buffer() {
                Ok(buf) => buf.clone(),
                Err(_) => {
                    warn!("No audio buffer available for silence detection");
                    return;
                }
            }
        };

        // Check if listening mode is enabled to determine return state after silence
        let return_to_listening = self
            .listening_state
            .as_ref()
            .and_then(|ls| ls.lock().ok())
            .map(|lm| lm.is_enabled())
            .unwrap_or(false);

        // Create transcription callback that calls spawn_transcription
        // This is the same pattern used in wake word flow
        let shared_model = self.shared_transcription_model.clone();
        let transcription_emitter_for_callback = self.transcription_emitter.clone();
        let app_handle_for_callback = self.app_handle.clone();
        let recording_state_for_callback = self.recording_state.clone();
        let transcription_semaphore_for_callback = self.transcription_semaphore.clone();
        let transcription_timeout_for_callback = self.transcription_timeout;

        // Build transcription callback
        let transcription_callback: Option<Box<dyn Fn(String) + Send + 'static>> =
            if shared_model.is_some() && transcription_emitter_for_callback.is_some() {
                Some(Box::new(move |file_path: String| {
                    // Spawn transcription using the same async pattern as spawn_transcription
                    let shared_model = match &shared_model {
                        Some(m) => m.clone(),
                        None => return,
                    };
                    let transcription_emitter = match &transcription_emitter_for_callback {
                        Some(te) => te.clone(),
                        None => return,
                    };

                    if !shared_model.is_loaded() {
                        info!("Transcription skipped: model not loaded");
                        return;
                    }

                    let semaphore = transcription_semaphore_for_callback.clone();
                    let timeout_duration = transcription_timeout_for_callback;
                    let app_handle = app_handle_for_callback.clone();
                    let recording_state = recording_state_for_callback.clone();

                    info!("[silence_detection] Spawning transcription task for: {}", file_path);

                    tauri::async_runtime::spawn(async move {
                        // Helper to clear recording buffer
                        let clear_recording_buffer = || {
                            if let Some(ref state) = recording_state {
                                if let Ok(mut manager) = state.lock() {
                                    manager.clear_last_recording();
                                    debug!("Cleared recording buffer");
                                }
                            }
                        };

                        // Acquire semaphore
                        let _permit = match semaphore.try_acquire() {
                            Ok(permit) => permit,
                            Err(_) => {
                                warn!("Too many concurrent transcriptions, skipping");
                                transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                                    error: "Too many transcriptions in progress.".to_string(),
                                });
                                clear_recording_buffer();
                                return;
                            }
                        };

                        // Emit started
                        let start_time = Instant::now();
                        transcription_emitter.emit_transcription_started(TranscriptionStartedPayload {
                            timestamp: current_timestamp(),
                        });

                        debug!("Transcribing file: {}", file_path);

                        // Perform transcription with timeout
                        let transcriber = shared_model.clone();
                        let transcription_future = tokio::task::spawn_blocking(move || {
                            transcriber.transcribe(&file_path)
                        });

                        let transcription_result = tokio::time::timeout(timeout_duration, transcription_future).await;

                        let text = match transcription_result {
                            Ok(Ok(Ok(text))) => text,
                            Ok(Ok(Err(e))) => {
                                error!("Transcription failed: {}", e);
                                transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                                    error: e.to_string(),
                                });
                                let _ = shared_model.reset_to_idle();
                                clear_recording_buffer();
                                return;
                            }
                            Ok(Err(e)) => {
                                error!("Transcription task panicked: {}", e);
                                transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                                    error: "Internal transcription error.".to_string(),
                                });
                                let _ = shared_model.reset_to_idle();
                                clear_recording_buffer();
                                return;
                            }
                            Err(_) => {
                                error!("Transcription timed out");
                                transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                                    error: format!("Transcription timed out after {} seconds.", timeout_duration.as_secs()),
                                });
                                let _ = shared_model.reset_to_idle();
                                clear_recording_buffer();
                                return;
                            }
                        };

                        let duration_ms = start_time.elapsed().as_millis() as u64;
                        info!("Transcription completed in {}ms: {} chars", duration_ms, text.len());

                        // Silence detection auto-stop always goes to clipboard
                        // Voice command matching is only supported for manual hotkey recordings
                        // (via spawn_transcription). This is by design - auto-stop recordings
                        // are intended for quick dictation, not command execution.
                        if let Some(ref handle) = app_handle {
                            if let Err(e) = handle.clipboard().write_text(&text) {
                                warn!("Failed to copy to clipboard: {}", e);
                            } else {
                                debug!("Transcribed text copied to clipboard");
                                if let Err(e) = simulate_paste() {
                                    warn!("Failed to auto-paste: {}", e);
                                }
                            }
                        }

                        // Emit completed
                        transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                            text,
                            duration_ms,
                        });

                        let _ = shared_model.reset_to_idle();
                        clear_recording_buffer();
                    });
                }))
            } else {
                None
            };

        // Lock detectors and start monitoring
        let mut det = match detectors.lock() {
            Ok(d) => d,
            Err(_) => {
                warn!("Failed to lock recording detectors");
                return;
            }
        };

        // Use the recording state Arc that was configured via builder
        let recording_state_arc = match &self.recording_state {
            Some(rs) => rs.clone(),
            None => {
                warn!("No recording state configured, cannot start silence detection");
                return;
            }
        };

        // Create a recording emitter for the detection coordinator
        // We need to use R which implements RecordingEventEmitter
        // But we can't clone self.recording_emitter since it's moved into self
        // Instead, create a new emitter from the app handle
        let recording_emitter_for_detectors = match &self.app_handle {
            Some(handle) => Arc::new(crate::commands::TauriEventEmitter::new(handle.clone())),
            None => {
                warn!("No app handle configured, cannot create emitter for silence detection");
                return;
            }
        };

        info!("[silence_detection] Starting monitoring for hotkey recording");
        if let Err(e) = det.start_monitoring(
            buffer,
            recording_state_arc,
            audio_thread,
            recording_emitter_for_detectors,
            return_to_listening,
            self.listening_pipeline.clone(),
            transcription_callback,
        ) {
            warn!("Failed to start silence detection: {}", e);
        } else {
            info!("[silence_detection] Monitoring started successfully");
        }
    }

    /// Stop silence detection for hotkey recording
    ///
    /// Called when the user manually stops recording via hotkey. This ensures
    /// the silence detection thread is stopped before processing the recording,
    /// allowing manual stop to take precedence over auto-stop.
    fn stop_silence_detection(&self) {
        let detectors = match &self.recording_detectors {
            Some(d) => d,
            None => return,
        };

        if let Ok(mut det) = detectors.lock() {
            if det.is_running() {
                info!("[silence_detection] Stopping monitoring (manual stop)");
                det.stop_monitoring();
            }
        }
    }

    /// Register the Escape key listener for cancel functionality
    ///
    /// Called when recording starts. Only registers if both shortcut_backend
    /// and escape_callback are configured. The listener is automatically
    /// unregistered when recording stops.
    ///
    /// Uses double-tap detection: single Escape presses are ignored, only
    /// double-taps within the configured time window trigger the cancel callback.
    ///
    /// IMPORTANT: The actual registration is deferred to a spawned thread to avoid
    /// re-entrancy deadlock. When this function is called from within a global shortcut
    /// callback (e.g., the recording hotkey), calling backend.register() synchronously
    /// would deadlock because the shortcut manager's lock is already held.
    fn register_escape_listener(&mut self) {
        // Skip if already registered or not configured
        if self.escape_registered {
            debug!("Escape listener already registered, skipping");
            return;
        }

        let backend = match &self.shortcut_backend {
            Some(b) => b.clone(),
            None => {
                debug!("No shortcut backend configured, skipping Escape registration");
                return;
            }
        };

        let callback = match &self.escape_callback {
            Some(c) => c.clone(),
            None => {
                debug!("No escape callback configured, skipping Escape registration");
                return;
            }
        };

        // Create double-tap detector with the configured window
        // The detector wraps the callback and only invokes it on double-tap
        let boxed_callback: Box<dyn Fn() + Send + Sync> = Box::new(move || callback());
        let detector = Arc::new(Mutex::new(DoubleTapDetector::with_window(
            boxed_callback,
            self.double_tap_window_ms,
        )));
        self.double_tap_detector = Some(detector.clone());

        // In tests, use synchronous registration (mock backends don't have deadlock issues)
        // In production, spawn registration on a separate thread to avoid re-entrancy deadlock
        #[cfg(test)]
        {
            match backend.register(super::ESCAPE_SHORTCUT, Box::new(move || {
                if let Ok(mut det) = detector.lock() {
                    if det.on_tap() {
                        debug!("Double-tap Escape detected, cancel triggered");
                    } else {
                        trace!("Single Escape tap recorded, waiting for double-tap");
                    }
                }
            })) {
                Ok(()) => {
                    self.escape_registered = true;
                    info!("Escape key listener registered for recording cancel (double-tap required)");
                }
                Err(e) => {
                    warn!("Failed to register Escape key listener: {}", e);
                    self.double_tap_detector = None;
                }
            }
        }

        #[cfg(not(test))]
        {
            // Mark as registered optimistically before spawning
            // If registration fails, cancel via Escape won't work but the app continues
            self.escape_registered = true;

            // Spawn registration on a separate thread to avoid re-entrancy deadlock
            // This is necessary because we're called from within a global shortcut callback,
            // and the shortcut manager holds a lock during callback execution.
            std::thread::spawn(move || {
                // Small delay to ensure the calling shortcut callback has completed
                std::thread::sleep(std::time::Duration::from_millis(10));

                match backend.register(super::ESCAPE_SHORTCUT, Box::new(move || {
                    if let Ok(mut det) = detector.lock() {
                        if det.on_tap() {
                            debug!("Double-tap Escape detected, cancel triggered");
                        } else {
                            trace!("Single Escape tap recorded, waiting for double-tap");
                        }
                    }
                })) {
                    Ok(()) => {
                        info!("Escape key listener registered for recording cancel (double-tap required)");
                    }
                    Err(e) => {
                        warn!("Failed to register Escape key listener: {}", e);
                        // Note: escape_registered remains true, but unregister will handle this gracefully
                    }
                }
            });
        }
    }

    /// Unregister the Escape key listener
    ///
    /// Called when recording stops (either normally or via cancellation).
    /// Safe to call even if listener was never registered. Also resets the
    /// double-tap detector state.
    ///
    /// IMPORTANT: Like register_escape_listener, the actual unregistration is deferred
    /// to a spawned thread to avoid re-entrancy deadlock when called from within a
    /// global shortcut callback (e.g., the recording hotkey or Escape key itself).
    fn unregister_escape_listener(&mut self) {
        // Reset double-tap detector state
        if let Some(ref detector) = self.double_tap_detector {
            if let Ok(mut det) = detector.lock() {
                det.reset();
            }
        }
        self.double_tap_detector = None;

        if !self.escape_registered {
            return;
        }

        let backend = match &self.shortcut_backend {
            Some(b) => b.clone(),
            None => return,
        };

        // Mark as unregistered immediately
        self.escape_registered = false;

        // In tests, use synchronous unregistration (mock backends don't have deadlock issues)
        // In production, spawn unregistration on a separate thread to avoid re-entrancy deadlock
        #[cfg(test)]
        {
            match backend.unregister(super::ESCAPE_SHORTCUT) {
                Ok(()) => {
                    debug!("Escape key listener unregistered");
                }
                Err(e) => {
                    warn!("Failed to unregister Escape key listener: {}", e);
                }
            }
        }

        #[cfg(not(test))]
        {
            // Spawn unregistration on a separate thread to avoid re-entrancy deadlock
            // This is necessary because we may be called from within a global shortcut callback
            // (e.g., when stopping via recording hotkey or cancelling via Escape double-tap).
            std::thread::spawn(move || {
                // Small delay to ensure the calling shortcut callback has completed
                std::thread::sleep(std::time::Duration::from_millis(10));

                match backend.unregister(super::ESCAPE_SHORTCUT) {
                    Ok(()) => {
                        debug!("Escape key listener unregistered");
                    }
                    Err(e) => {
                        // This can happen if registration failed or was never completed
                        warn!("Failed to unregister Escape key listener: {}", e);
                    }
                }
            });
        }
    }

    /// Cancel recording without transcription
    ///
    /// This method is called when the user double-taps Escape during recording.
    /// It stops the recording immediately, discards the audio buffer (no WAV file
    /// created, no transcription triggered), and transitions directly to Idle state.
    ///
    /// # Arguments
    /// * `state` - The recording state mutex
    /// * `reason` - The reason for cancellation (e.g., "double-tap-escape")
    ///
    /// # Returns
    /// * `true` if cancellation was successful
    /// * `false` if not in recording state or an error occurred
    pub fn cancel_recording(&mut self, state: &Mutex<RecordingManager>, reason: &str) -> bool {
        // Check current state - can only cancel from Recording state
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                error!("Failed to acquire lock for cancel: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        if current_state != RecordingState::Recording {
            debug!("Cancel ignored - not in recording state (current: {:?})", current_state);
            return false;
        }

        info!("Cancelling recording (reason: {})", reason);

        // 1. Unregister Escape key listener first
        self.unregister_escape_listener();

        // 2. Stop silence detection if active
        self.stop_silence_detection();

        // 3. Stop audio capture (discard result - we don't want the audio)
        if let Some(ref audio_thread) = self.audio_thread {
            // Stop the audio thread to halt capture
            if let Err(e) = audio_thread.stop() {
                warn!("Failed to stop audio thread during cancel: {:?}", e);
                // Continue anyway - the buffer will be discarded
            }
        }

        // 4. Abort recording - this clears the buffer and transitions directly to Idle
        //    (bypassing Processing state, so no transcription will be triggered)
        let abort_result = match state.lock() {
            Ok(mut guard) => guard.abort_recording(RecordingState::Idle),
            Err(e) => {
                error!("Failed to acquire lock for abort: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        match abort_result {
            Ok(()) => {
                // 5. Emit recording_cancelled event
                self.recording_emitter.emit_recording_cancelled(RecordingCancelledPayload {
                    reason: reason.to_string(),
                    timestamp: current_timestamp(),
                });

                info!("Recording cancelled successfully");
                true
            }
            Err(e) => {
                error!("Failed to abort recording: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: format!("Failed to cancel recording: {}", e),
                });
                false
            }
        }
    }

    /// Check if currently in debounce window (for testing)
    #[cfg(test)]
    pub fn is_debouncing(&self) -> bool {
        if let Some(last) = self.last_toggle_time {
            Instant::now().duration_since(last) < self.debounce_duration
        } else {
            false
        }
    }
}

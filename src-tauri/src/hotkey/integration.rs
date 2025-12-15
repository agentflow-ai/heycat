// Hotkey-to-recording integration module
// Connects global hotkey to recording state with debouncing
// Uses unified command implementations for start/stop logic

use crate::audio::AudioThreadHandle;
use crate::commands::logic::{start_recording_impl, stop_recording_impl};
use crate::events::{
    current_timestamp, CommandAmbiguousPayload, CommandCandidate, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload, ListeningEventEmitter,
    RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::listening::{ListeningManager, ListeningPipeline};
use crate::model::{check_model_exists_for_type, ModelType};
use crate::recording::{RecordingManager, RecordingState};
use crate::voice_commands::executor::ActionDispatcher;
use crate::voice_commands::matcher::{CommandMatcher, MatchResult};
use crate::voice_commands::registry::CommandRegistry;
use crate::parakeet::{TranscriptionManager, TranscriptionService};
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
    /// Optional TranscriptionManager for auto-transcription after recording stops
    transcription_manager: Option<Arc<TranscriptionManager>>,
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
}

impl<R: RecordingEventEmitter, T: TranscriptionEventEmitter + ListeningEventEmitter + 'static, C: CommandEventEmitter + 'static> HotkeyIntegration<R, T, C> {
    /// Create a new HotkeyIntegration with default debounce duration
    pub fn new(recording_emitter: R) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(DEBOUNCE_DURATION_MS),
            recording_emitter,
            audio_thread: None,
            transcription_manager: None,
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
        }
    }

    /// Add app handle for clipboard access (builder pattern)
    pub fn with_app_handle(mut self, handle: AppHandle) -> Self {
        self.app_handle = Some(handle);
        self
    }

    /// Add an audio thread handle (builder pattern)
    pub fn with_audio_thread(mut self, handle: Arc<AudioThreadHandle>) -> Self {
        self.audio_thread = Some(handle);
        self
    }

    /// Add TranscriptionManager for auto-transcription (builder pattern)
    pub fn with_transcription_manager(mut self, manager: Arc<TranscriptionManager>) -> Self {
        self.transcription_manager = Some(manager);
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
    pub fn with_transcription_timeout(mut self, timeout: Duration) -> Self {
        self.transcription_timeout = timeout;
        self
    }

    /// Create with custom debounce duration (for testing)
    #[cfg(test)]
    pub fn with_debounce(recording_emitter: R, debounce_ms: u64) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(debounce_ms),
            recording_emitter,
            audio_thread: None,
            transcription_manager: None,
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
                match start_recording_impl(state, self.audio_thread.as_deref(), model_available) {
                    Ok(()) => {
                        self.recording_emitter
                            .emit_recording_started(RecordingStartedPayload {
                                timestamp: current_timestamp(),
                            });
                        info!("Recording started, emitted recording_started event");
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
                info!("Stopping recording...");

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
        let transcription_manager = match &self.transcription_manager {
            Some(tm) => tm.clone(),
            None => {
                debug!("Transcription skipped: no transcription manager configured");
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
        if !transcription_manager.is_loaded() {
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
            let transcriber = transcription_manager.clone();
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
                    if let Err(reset_err) = transcription_manager.reset_to_idle() {
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
                    if let Err(reset_err) = transcription_manager.reset_to_idle() {
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
                    if let Err(reset_err) = transcription_manager.reset_to_idle() {
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
            if let Err(e) = transcription_manager.reset_to_idle() {
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

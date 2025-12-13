// Hotkey-to-recording integration module
// Connects global hotkey to recording state with debouncing
// Uses unified command implementations for start/stop logic

use crate::audio::AudioThreadHandle;
use crate::commands::logic::{get_last_recording_buffer_impl, start_recording_impl, stop_recording_impl};
use crate::events::{
    command_events, current_timestamp, CommandAmbiguousPayload, CommandCandidate,
    CommandEventEmitter, CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload,
    RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::model::check_model_exists;
use crate::recording::{RecordingManager, RecordingState};
use crate::voice_commands::executor::{ActionDispatcher, ExecutorState};
use crate::voice_commands::matcher::{CommandMatcher, MatchResult};
use crate::voice_commands::registry::CommandRegistry;
use crate::whisper::{TranscriptionService, WhisperManager};
use crate::{debug, error, info, trace, warn};
use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Debounce duration for hotkey presses (200ms)
pub const DEBOUNCE_DURATION_MS: u64 = 200;

/// Handles hotkey toggle with debouncing and event emission
pub struct HotkeyIntegration<R: RecordingEventEmitter, T: TranscriptionEventEmitter, C: CommandEventEmitter> {
    last_toggle_time: Option<Instant>,
    debounce_duration: Duration,
    recording_emitter: R,
    /// Optional audio thread handle - when present, starts/stops capture on toggle
    audio_thread: Option<Arc<AudioThreadHandle>>,
    /// Optional WhisperManager for auto-transcription after recording stops
    whisper_manager: Option<Arc<WhisperManager>>,
    /// Transcription event emitter for emitting events from spawned thread
    transcription_emitter: Option<Arc<T>>,
    /// Reference to recording state for getting audio buffer in transcription thread
    recording_state: Option<Arc<Mutex<RecordingManager>>>,
    /// Optional command registry for voice command matching
    command_registry: Option<Arc<Mutex<CommandRegistry>>>,
    /// Optional command matcher for voice command matching
    command_matcher: Option<Arc<CommandMatcher>>,
    /// Optional action dispatcher for executing matched commands
    action_dispatcher: Option<Arc<ActionDispatcher>>,
    /// Optional command event emitter for voice command events
    command_emitter: Option<Arc<C>>,
}

impl<R: RecordingEventEmitter, T: TranscriptionEventEmitter + 'static, C: CommandEventEmitter + 'static> HotkeyIntegration<R, T, C> {
    /// Create a new HotkeyIntegration with default debounce duration
    pub fn new(recording_emitter: R) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(DEBOUNCE_DURATION_MS),
            recording_emitter,
            audio_thread: None,
            whisper_manager: None,
            transcription_emitter: None,
            recording_state: None,
            command_registry: None,
            command_matcher: None,
            action_dispatcher: None,
            command_emitter: None,
        }
    }

    /// Add an audio thread handle (builder pattern)
    pub fn with_audio_thread(mut self, handle: Arc<AudioThreadHandle>) -> Self {
        self.audio_thread = Some(handle);
        self
    }

    /// Add WhisperManager for auto-transcription (builder pattern)
    pub fn with_whisper_manager(mut self, manager: Arc<WhisperManager>) -> Self {
        self.whisper_manager = Some(manager);
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

    /// Create with custom debounce duration (for testing)
    #[cfg(test)]
    pub fn with_debounce(recording_emitter: R, debounce_ms: u64) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(debounce_ms),
            recording_emitter,
            audio_thread: None,
            whisper_manager: None,
            transcription_emitter: None,
            recording_state: None,
            command_registry: None,
            command_matcher: None,
            action_dispatcher: None,
            command_emitter: None,
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
            RecordingState::Idle => {
                info!("Starting recording...");
                // Check model availability before starting
                let model_available = check_model_exists().unwrap_or(false);
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
                // Use unified command implementation
                match stop_recording_impl(state, self.audio_thread.as_deref()) {
                    Ok(metadata) => {
                        info!(
                            "Recording stopped: {} samples, {:.2}s duration",
                            metadata.sample_count, metadata.duration_secs
                        );
                        self.recording_emitter
                            .emit_recording_stopped(RecordingStoppedPayload { metadata });
                        debug!("Emitted recording_stopped event");

                        // Auto-transcribe if whisper manager is configured
                        self.spawn_transcription();

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

    /// Spawn transcription in a separate thread
    ///
    /// Gets audio buffer, transcribes, tries command matching, then fallback to clipboard.
    /// No-op if whisper manager, transcription emitter, or recording state is not configured.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn spawn_transcription(&self) {
        // Check all required components are present
        let whisper_manager = match &self.whisper_manager {
            Some(wm) => wm.clone(),
            None => {
                debug!("Transcription skipped: no whisper manager configured");
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

        let recording_state = match &self.recording_state {
            Some(rs) => rs.clone(),
            None => {
                debug!("Transcription skipped: no recording state configured");
                return;
            }
        };

        // Optional voice command components
        let command_registry = self.command_registry.clone();
        let command_matcher = self.command_matcher.clone();
        let action_dispatcher = self.action_dispatcher.clone();
        let command_emitter = self.command_emitter.clone();

        // Check if model is loaded
        if !whisper_manager.is_loaded() {
            info!("Transcription skipped: whisper model not loaded");
            return;
        }

        info!("Spawning transcription thread...");

        std::thread::spawn(move || {
            // Emit transcription_started event
            let start_time = Instant::now();
            transcription_emitter.emit_transcription_started(TranscriptionStartedPayload {
                timestamp: current_timestamp(),
            });

            // Get audio buffer
            let samples = match get_last_recording_buffer_impl(&recording_state) {
                Ok(audio_data) => audio_data.samples,
                Err(e) => {
                    error!("Failed to get recording buffer: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: format!("Failed to get recording buffer: {}", e),
                    });
                    return;
                }
            };

            debug!("Transcribing {} samples...", samples.len());

            // Perform transcription
            match whisper_manager.transcribe(&samples) {
                Ok(text) => {
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    info!(
                        "Transcription completed in {}ms: {} chars",
                        duration_ms,
                        text.len()
                    );

                    // Try voice command matching if configured
                    let command_handled = if let (Some(registry), Some(matcher), Some(dispatcher), Some(emitter)) =
                        (&command_registry, &command_matcher, &action_dispatcher, &command_emitter)
                    {
                        // Lock registry and try to match
                        if let Ok(registry_guard) = registry.lock() {
                            let match_result = matcher.match_input(&text, &registry_guard);

                            match &match_result {
                                MatchResult::Exact { command: matched_cmd, .. } | MatchResult::Fuzzy { command: matched_cmd, score: _, .. } => {
                                    let confidence = match &match_result {
                                        MatchResult::Exact { .. } => 1.0,
                                        MatchResult::Fuzzy { score, .. } => *score,
                                        _ => 0.0,
                                    };
                                    info!("Command matched: {} (confidence: {:.2})", matched_cmd.trigger, confidence);

                                    // Get full command from registry
                                    let full_command = registry_guard.get(matched_cmd.id).cloned();
                                    drop(registry_guard); // Release lock before execution

                                    if let Some(cmd) = full_command {
                                        // Emit command_matched event
                                        emitter.emit_command_matched(CommandMatchedPayload {
                                            transcription: text.clone(),
                                            command_id: cmd.id.to_string(),
                                            trigger: matched_cmd.trigger.clone(),
                                            confidence,
                                        });

                                        // Execute command using tokio runtime
                                        let rt = tokio::runtime::Runtime::new();
                                        match rt {
                                            Ok(runtime) => {
                                                let exec_result = runtime.block_on(dispatcher.execute(&cmd));
                                                match exec_result {
                                                    Ok(action_result) => {
                                                        info!("Command executed: {}", action_result.message);
                                                        emitter.emit_command_executed(CommandExecutedPayload {
                                                            command_id: cmd.id.to_string(),
                                                            trigger: matched_cmd.trigger.clone(),
                                                            message: action_result.message,
                                                        });
                                                    }
                                                    Err(action_error) => {
                                                        error!("Command execution failed: {}", action_error);
                                                        emitter.emit_command_failed(CommandFailedPayload {
                                                            command_id: cmd.id.to_string(),
                                                            trigger: matched_cmd.trigger.clone(),
                                                            error_code: action_error.code,
                                                            error_message: action_error.message,
                                                        });
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                error!("Failed to create runtime for command execution: {}", e);
                                                emitter.emit_command_failed(CommandFailedPayload {
                                                    command_id: cmd.id.to_string(),
                                                    trigger: matched_cmd.trigger.clone(),
                                                    error_code: "RUNTIME_ERROR".to_string(),
                                                    error_message: format!("Failed to create async runtime: {}", e),
                                                });
                                            }
                                        }
                                        true // Command was handled
                                    } else {
                                        warn!("Command not found in registry after match");
                                        false
                                    }
                                }
                                MatchResult::Ambiguous { ref candidates } => {
                                    info!("Ambiguous match: {} candidates", candidates.len());

                                    // Emit command_ambiguous event for disambiguation UI
                                    emitter.emit_command_ambiguous(CommandAmbiguousPayload {
                                        transcription: text.clone(),
                                        candidates: candidates
                                            .iter()
                                            .map(|c| CommandCandidate {
                                                id: c.command.id.to_string(),
                                                trigger: c.command.trigger.clone(),
                                                confidence: c.score,
                                            })
                                            .collect(),
                                    });
                                    true // Command matching was handled (ambiguous)
                                }
                                MatchResult::NoMatch => {
                                    debug!("No command match for: {}", text);
                                    false // Fall through to clipboard
                                }
                            }
                        } else {
                            warn!("Failed to lock command registry");
                            false
                        }
                    } else {
                        debug!("Voice commands not configured, skipping command matching");
                        false
                    };

                    // Fallback to clipboard if no command was handled
                    if !command_handled {
                        // Copy to clipboard
                        match Clipboard::new() {
                            Ok(mut clipboard) => {
                                if let Err(e) = clipboard.set_text(&text) {
                                    warn!("Failed to copy to clipboard: {}", e);
                                } else {
                                    debug!("Transcribed text copied to clipboard");
                                }
                            }
                            Err(e) => {
                                warn!("Failed to access clipboard: {}", e);
                            }
                        }

                        // Emit transcription_completed event
                        transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                            text,
                            duration_ms,
                        });
                    }

                    // Reset whisper state to idle
                    if let Err(e) = whisper_manager.reset_to_idle() {
                        warn!("Failed to reset whisper state: {}", e);
                    }
                }
                Err(e) => {
                    error!("Transcription failed: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: e.to_string(),
                    });

                    // Reset whisper state to idle on error
                    if let Err(reset_err) = whisper_manager.reset_to_idle() {
                        warn!("Failed to reset whisper state: {}", reset_err);
                    }
                }
            }
        });
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

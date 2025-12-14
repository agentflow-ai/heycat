// Hotkey-to-recording integration module
// Connects global hotkey to recording state with debouncing
// Uses unified command implementations for start/stop logic

use crate::audio::{AudioThreadHandle, StreamingAudioReceiver, StreamingAudioSender};
use crate::commands::logic::{start_recording_impl, stop_recording_impl};
use crate::events::{
    current_timestamp, CommandAmbiguousPayload, CommandCandidate, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload, RecordingErrorPayload,
    RecordingEventEmitter, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::model::{check_model_exists_for_type, ModelType};
use crate::recording::{RecordingManager, RecordingState};
use crate::voice_commands::executor::ActionDispatcher;
use crate::voice_commands::matcher::{CommandMatcher, MatchResult};
use crate::voice_commands::registry::CommandRegistry;
use crate::parakeet::{StreamingTranscriber, TranscriptionManager, TranscriptionMode, TranscriptionService};
use crate::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::sync::Semaphore;

/// Maximum concurrent transcriptions allowed
const MAX_CONCURRENT_TRANSCRIPTIONS: usize = 2;

/// Debounce duration for hotkey presses (200ms)
pub const DEBOUNCE_DURATION_MS: u64 = 200;

/// Handles hotkey toggle with debouncing and event emission
pub struct HotkeyIntegration<R: RecordingEventEmitter, T: TranscriptionEventEmitter, C: CommandEventEmitter> {
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
    /// Optional streaming transcriber for real-time EOU transcription
    streaming_transcriber: Option<Arc<Mutex<StreamingTranscriber<T>>>>,
    /// Holds the streaming receiver during recording (wrapped in Option for take semantics)
    streaming_receiver: Arc<Mutex<Option<StreamingAudioReceiver>>>,
}

impl<R: RecordingEventEmitter, T: TranscriptionEventEmitter + 'static, C: CommandEventEmitter + 'static> HotkeyIntegration<R, T, C> {
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
            command_registry: None,
            command_matcher: None,
            action_dispatcher: None,
            command_emitter: None,
            transcription_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
            app_handle: None,
            streaming_transcriber: None,
            streaming_receiver: Arc::new(Mutex::new(None)),
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

    /// Add streaming transcriber for real-time EOU transcription (builder pattern)
    pub fn with_streaming_transcriber(mut self, transcriber: Arc<Mutex<StreamingTranscriber<T>>>) -> Self {
        self.streaming_transcriber = Some(transcriber);
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
            command_registry: None,
            command_matcher: None,
            action_dispatcher: None,
            command_emitter: None,
            transcription_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
            app_handle: None,
            streaming_transcriber: None,
            streaming_receiver: Arc::new(Mutex::new(None)),
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
                // Check model availability before starting (check Parakeet TDT model)
                let model_available = check_model_exists_for_type(ModelType::ParakeetTDT).unwrap_or(false);

                // Check transcription mode at recording start (not toggle time) for deterministic behavior
                let mode = self.transcription_manager.as_ref()
                    .map(|tm| tm.current_mode())
                    .unwrap_or(TranscriptionMode::Batch);

                // Create streaming channel if in streaming mode
                let streaming_sender = match mode {
                    TranscriptionMode::Streaming => {
                        let (sender, receiver) = std::sync::mpsc::sync_channel::<Vec<f32>>(10);
                        // Store receiver for consumer task
                        if let Ok(mut rx_holder) = self.streaming_receiver.lock() {
                            *rx_holder = Some(receiver);
                        }
                        // Spawn consumer task to process audio chunks
                        self.spawn_streaming_consumer();
                        Some(sender)
                    }
                    TranscriptionMode::Batch => None,
                };

                debug!("Recording mode: {:?}, streaming_sender: {}", mode, streaming_sender.is_some());

                // Use unified command implementation
                match start_recording_impl(state, self.audio_thread.as_deref(), model_available, streaming_sender) {
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

                // Check transcription mode to decide how to handle transcription
                let mode = self.transcription_manager.as_ref()
                    .map(|tm| tm.current_mode())
                    .unwrap_or(TranscriptionMode::Batch);

                // Use unified command implementation
                match stop_recording_impl(state, self.audio_thread.as_deref()) {
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

                        // Handle transcription based on mode
                        match mode {
                            TranscriptionMode::Batch => {
                                // Auto-transcribe if transcription manager is configured
                                self.spawn_transcription(file_path_for_transcription);
                            }
                            TranscriptionMode::Streaming => {
                                // Finalize streaming transcription
                                self.finalize_streaming();
                            }
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
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn spawn_transcription(&self, file_path: String) {
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

        // Check if model is loaded
        if !transcription_manager.is_loaded() {
            info!("Transcription skipped: transcription model not loaded");
            return;
        }

        // Clone semaphore for the async task
        let semaphore = self.transcription_semaphore.clone();

        info!("Spawning transcription task...");

        // Spawn async task using Tauri's async runtime
        tauri::async_runtime::spawn(async move {
            // Acquire semaphore permit to limit concurrent transcriptions
            let _permit = match semaphore.try_acquire() {
                Ok(permit) => permit,
                Err(_) => {
                    warn!("Too many concurrent transcriptions, skipping this one");
                    // Emit error event so the user knows their recording was not processed
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: "Too many transcriptions in progress. Please wait and try again.".to_string(),
                    });
                    return;
                }
            };

            // Emit transcription_started event
            let start_time = Instant::now();
            transcription_emitter.emit_transcription_started(TranscriptionStartedPayload {
                timestamp: current_timestamp(),
            });

            debug!("Transcribing file: {}", file_path);

            // Perform transcription on blocking thread pool (CPU-intensive)
            let transcriber = transcription_manager.clone();
            let transcription_result = tokio::task::spawn_blocking(move || {
                transcriber.transcribe(&file_path)
            }).await;

            let text = match transcription_result {
                Ok(Ok(text)) => text,
                Ok(Err(e)) => {
                    error!("Transcription failed: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: e.to_string(),
                    });
                    if let Err(reset_err) = transcription_manager.reset_to_idle() {
                        warn!("Failed to reset transcription state: {}", reset_err);
                    }
                    return;
                }
                Err(e) => {
                    error!("Transcription task panicked: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: "Internal transcription error.".to_string(),
                    });
                    if let Err(reset_err) = transcription_manager.reset_to_idle() {
                        warn!("Failed to reset transcription state: {}", reset_err);
                    }
                    return;
                }
            };

            let duration_ms = start_time.elapsed().as_millis() as u64;
            info!(
                "Transcription completed in {}ms: {} chars",
                duration_ms,
                text.len()
            );

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
                    }
                } else {
                    warn!("Clipboard unavailable: no app handle configured");
                }

                // Emit transcription_completed event
                transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                    text,
                    duration_ms,
                });
            }

            // Reset transcription state to idle
            if let Err(e) = transcription_manager.reset_to_idle() {
                warn!("Failed to reset transcription state: {}", e);
            }
        });
    }

    /// Spawn a consumer thread that reads audio chunks from the streaming channel
    /// and processes them through the streaming transcriber
    ///
    /// This is called at recording start in streaming mode. The thread will exit
    /// when the channel is closed (receiver dropped on recording stop).
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn spawn_streaming_consumer(&self) {
        let receiver = self.streaming_receiver.clone();
        let transcriber = match &self.streaming_transcriber {
            Some(t) => t.clone(),
            None => {
                debug!("Streaming consumer not spawned: no streaming transcriber configured");
                return;
            }
        };

        debug!("Spawning streaming consumer thread...");

        std::thread::spawn(move || {
            debug!("Streaming consumer thread started");
            // Take the receiver out of the holder
            let rx = {
                let mut rx_guard = match receiver.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        error!("Streaming consumer: failed to lock receiver holder: {}", e);
                        return;
                    }
                };
                rx_guard.take()
            };

            let rx = match rx {
                Some(rx) => rx,
                None => {
                    warn!("Streaming consumer: no receiver available");
                    return;
                }
            };

            // Process chunks until channel is closed
            while let Ok(chunk) = rx.recv() {
                if let Ok(mut t) = transcriber.lock() {
                    if let Err(e) = t.process_samples(&chunk) {
                        warn!("Streaming transcription error: {}", e);
                    }
                }
            }

            debug!("Streaming consumer thread exiting (channel closed)");
        });
    }

    /// Finalize streaming transcription and handle the result
    ///
    /// Called when recording stops in streaming mode. Closes the channel to stop
    /// the consumer thread, then finalizes the transcriber to get the complete text.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn finalize_streaming(&self) {
        let transcriber = match &self.streaming_transcriber {
            Some(t) => t.clone(),
            None => {
                debug!("Streaming finalization skipped: no streaming transcriber configured");
                return;
            }
        };

        // Drop the receiver to signal consumer thread to exit
        // This closes the channel, causing recv() to return Err
        {
            if let Ok(mut rx_holder) = self.streaming_receiver.lock() {
                *rx_holder = None;
            }
        }

        // Give the consumer thread a moment to finish processing
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Finalize transcription and get the complete text
        let result = {
            let mut t = match transcriber.lock() {
                Ok(guard) => guard,
                Err(e) => {
                    error!("Streaming finalization failed: lock poisoned: {}", e);
                    return;
                }
            };

            let result = t.finalize();
            t.reset();
            result
        };

        match result {
            Ok(text) => {
                info!("Streaming transcription finalized: {} chars", text.len());
                // Handle command matching / clipboard same as batch mode
                self.handle_transcription_result(&text);
            }
            Err(e) => {
                error!("Streaming finalization failed: {}", e);
            }
        }
    }

    /// Handle transcription result: try command matching, fallback to clipboard
    ///
    /// This is the common result handling for both batch and streaming modes.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn handle_transcription_result(&self, text: &str) {
        // Try voice command matching if configured
        // Note: Streaming mode doesn't need to emit transcription_started because
        // partial events are already being emitted during recording

        // For now, just copy to clipboard as fallback
        // Full command matching would require async execution which complicates
        // the synchronous finalize_streaming flow
        if let Some(ref handle) = self.app_handle {
            if let Err(e) = handle.clipboard().write_text(text) {
                warn!("Failed to copy to clipboard: {}", e);
            } else {
                debug!("Transcribed text copied to clipboard");
            }
        } else {
            warn!("Clipboard unavailable: no app handle configured");
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

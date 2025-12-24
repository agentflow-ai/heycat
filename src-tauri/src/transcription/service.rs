// RecordingTranscriptionService - unified transcription flow
// Handles: WAV transcription → command matching → clipboard fallback
//
// This service decouples transcription from HotkeyIntegration, enabling
// button-initiated recordings and wake word flows to share the same logic.

use crate::dictionary::{DictionaryEntry, DictionaryExpander, ExpansionResult};
use crate::events::{
    current_timestamp, CommandAmbiguousPayload, CommandCandidate, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::parakeet::{SharedTranscriptionModel, TranscriptionService as TranscriptionServiceTrait};
use crate::recording::RecordingManager;
use crate::voice_commands::executor::ActionDispatcher;
use crate::voice_commands::matcher::{CommandMatcher, MatchResult};
use crate::voice_commands::registry::CommandRegistry;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::sync::Semaphore;

/// Maximum concurrent transcriptions allowed
const MAX_CONCURRENT_TRANSCRIPTIONS: usize = 2;

/// Default transcription timeout in seconds
pub const DEFAULT_TRANSCRIPTION_TIMEOUT_SECS: u64 = 60;

/// Simulate Cmd+V paste keystroke on macOS using CoreGraphics
#[cfg(target_os = "macos")]
fn simulate_paste() -> Result<(), String> {
    eprintln!("[PASTE-TRACE] service.rs::simulate_paste() ENTRY, shutting_down={}", crate::shutdown::is_shutting_down());

    // Safety check: don't paste during shutdown
    if crate::shutdown::is_shutting_down() {
        eprintln!("[PASTE-TRACE] service.rs::simulate_paste() - BLOCKED by shutdown check");
        crate::debug!("Skipping paste - app is shutting down");
        return Ok(());
    }

    // Centralized synthesis ensures key-up always follows key-down and sequences don't interleave.
    crate::keyboard::synth::simulate_cmd_v_paste()?;

    eprintln!("[PASTE-TRACE] service.rs::simulate_paste() EXIT - paste completed");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn simulate_paste() -> Result<(), String> {
    Err("Paste simulation only supported on macOS".to_string())
}

/// Service for handling recording transcription and command matching
///
/// This service provides a unified transcription flow that can be used by:
/// - Hotkey-triggered recordings
/// - Button-initiated recordings (via stop_recording command)
/// - Wake word recordings
///
/// The flow is: WAV transcription → command matching → clipboard fallback
pub struct RecordingTranscriptionService<T, C>
where
    T: TranscriptionEventEmitter + 'static,
    C: CommandEventEmitter + 'static,
{
    /// Shared transcription model for transcribing audio
    shared_transcription_model: Arc<SharedTranscriptionModel>,
    /// Event emitter for transcription events
    transcription_emitter: Arc<T>,
    /// Recording state for buffer cleanup
    recording_state: Arc<Mutex<RecordingManager>>,
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
    /// App handle for clipboard access
    app_handle: AppHandle,
    /// Transcription timeout duration
    transcription_timeout: Duration,
    /// Dictionary expander for text expansion (interior mutable for runtime updates)
    dictionary_expander: Arc<RwLock<Option<DictionaryExpander>>>,
}

impl<T, C> RecordingTranscriptionService<T, C>
where
    T: TranscriptionEventEmitter + Send + Sync + 'static,
    C: CommandEventEmitter + Send + Sync + 'static,
{
    /// Create a new RecordingTranscriptionService with required dependencies
    pub fn new(
        shared_transcription_model: Arc<SharedTranscriptionModel>,
        transcription_emitter: Arc<T>,
        recording_state: Arc<Mutex<RecordingManager>>,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            shared_transcription_model,
            transcription_emitter,
            recording_state,
            command_registry: None,
            command_matcher: None,
            action_dispatcher: None,
            command_emitter: None,
            transcription_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
            app_handle,
            transcription_timeout: Duration::from_secs(DEFAULT_TRANSCRIPTION_TIMEOUT_SECS),
            dictionary_expander: Arc::new(RwLock::new(None)),
        }
    }

    /// Add voice command registry (builder pattern)
    pub fn with_command_registry(mut self, registry: Arc<Mutex<CommandRegistry>>) -> Self {
        self.command_registry = Some(registry);
        self
    }

    /// Add command matcher (builder pattern)
    pub fn with_command_matcher(mut self, matcher: Arc<CommandMatcher>) -> Self {
        self.command_matcher = Some(matcher);
        self
    }

    /// Add action dispatcher (builder pattern)
    pub fn with_action_dispatcher(mut self, dispatcher: Arc<ActionDispatcher>) -> Self {
        self.action_dispatcher = Some(dispatcher);
        self
    }

    /// Add command event emitter (builder pattern)
    pub fn with_command_emitter(mut self, emitter: Arc<C>) -> Self {
        self.command_emitter = Some(emitter);
        self
    }

    /// Set custom transcription timeout (builder pattern)
    #[allow(dead_code)]
    pub fn with_transcription_timeout(mut self, timeout: Duration) -> Self {
        self.transcription_timeout = timeout;
        self
    }

    /// Add dictionary expander for text expansion (builder pattern)
    pub fn with_dictionary_expander(mut self, expander: DictionaryExpander) -> Self {
        self.dictionary_expander = Arc::new(RwLock::new(Some(expander)));
        self
    }

    /// Update the dictionary expander with new entries at runtime
    ///
    /// This method is called when dictionary entries are added, updated, or deleted
    /// to ensure the transcription pipeline uses the latest dictionary.
    pub fn update_dictionary(&self, entries: &[DictionaryEntry]) {
        let expander = if entries.is_empty() {
            crate::debug!("Clearing dictionary expander (no entries)");
            None
        } else {
            crate::info!(
                "Updating dictionary expander with {} entries",
                entries.len()
            );
            Some(DictionaryExpander::new(entries))
        };

        match self.dictionary_expander.write() {
            Ok(mut guard) => {
                *guard = expander;
                crate::debug!("Dictionary expander updated successfully");
            }
            Err(e) => {
                crate::error!("Failed to update dictionary expander: {}", e);
            }
        }
    }

    /// Process a recording file: transcribe → match commands → clipboard fallback
    ///
    /// This is the main entry point for transcription. It:
    /// 1. Checks if the model is loaded
    /// 2. Spawns an async task to transcribe the file
    /// 3. Tries voice command matching if configured
    /// 4. Falls back to clipboard + auto-paste if no command matched
    /// 5. Emits transcription events for frontend state updates
    ///
    /// This method is non-blocking - it spawns the transcription as an async task.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn process_recording(&self, file_path: String) {
        // Check if model is loaded
        if !self.shared_transcription_model.is_loaded() {
            crate::info!("Transcription skipped: transcription model not loaded");
            return;
        }

        // Clone all required components for the async task
        let shared_model = self.shared_transcription_model.clone();
        let transcription_emitter = self.transcription_emitter.clone();
        let recording_state = self.recording_state.clone();
        let command_registry = self.command_registry.clone();
        let command_matcher = self.command_matcher.clone();
        let action_dispatcher = self.action_dispatcher.clone();
        let command_emitter = self.command_emitter.clone();
        let app_handle = self.app_handle.clone();
        let semaphore = self.transcription_semaphore.clone();
        let timeout_duration = self.transcription_timeout;
        let dictionary_expander = self.dictionary_expander.clone();

        crate::info!("Spawning transcription task for: {}", file_path);

        // Spawn async task using Tauri's async runtime
        tauri::async_runtime::spawn(async move {
            // Helper to clear recording buffer - call this in all exit paths to prevent memory leaks
            let clear_recording_buffer = || {
                if let Ok(mut manager) = recording_state.lock() {
                    manager.clear_last_recording();
                    crate::debug!("Cleared recording buffer");
                }
            };

            // Acquire semaphore permit to limit concurrent transcriptions
            let _permit = match semaphore.try_acquire() {
                Ok(permit) => permit,
                Err(_) => {
                    crate::warn!("Too many concurrent transcriptions, skipping this one");
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: "Too many transcriptions in progress. Please wait and try again."
                            .to_string(),
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

            crate::debug!("Transcribing file: {}", file_path);

            // Perform transcription on blocking thread pool (CPU-intensive) with timeout
            let transcriber = shared_model.clone();
            let transcription_future =
                tokio::task::spawn_blocking(move || transcriber.transcribe(&file_path));

            let transcription_result =
                tokio::time::timeout(timeout_duration, transcription_future).await;

            let text = match transcription_result {
                Ok(Ok(Ok(text))) => text,
                Ok(Ok(Err(e))) => {
                    crate::error!("Transcription failed: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: e.to_string(),
                    });
                    if let Err(reset_err) = shared_model.reset_to_idle() {
                        crate::warn!("Failed to reset transcription state: {}", reset_err);
                    }
                    clear_recording_buffer();
                    return;
                }
                Ok(Err(e)) => {
                    crate::error!("Transcription task panicked: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: "Internal transcription error.".to_string(),
                    });
                    if let Err(reset_err) = shared_model.reset_to_idle() {
                        crate::warn!("Failed to reset transcription state: {}", reset_err);
                    }
                    clear_recording_buffer();
                    return;
                }
                Err(_) => {
                    // Timeout error
                    crate::error!("Transcription timed out after {:?}", timeout_duration);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: format!(
                            "Transcription timed out after {} seconds. The audio may be too long or the model may be stuck.",
                            timeout_duration.as_secs()
                        ),
                    });
                    if let Err(reset_err) = shared_model.reset_to_idle() {
                        crate::warn!("Failed to reset transcription state: {}", reset_err);
                    }
                    clear_recording_buffer();
                    return;
                }
            };

            let duration_ms = start_time.elapsed().as_millis() as u64;
            crate::info!(
                "Transcription completed in {}ms: {} chars",
                duration_ms,
                text.len()
            );

            // Apply dictionary expansion if configured
            let expansion_result = match dictionary_expander.read() {
                Ok(guard) => {
                    if let Some(ref expander) = *guard {
                        let result = expander.expand(&text);
                        if result.expanded_text != text {
                            crate::debug!(
                                "Dictionary expansion applied: '{}' -> '{}'",
                                text,
                                result.expanded_text
                            );
                        }
                        result
                    } else {
                        ExpansionResult {
                            expanded_text: text,
                            should_press_enter: false,
                        }
                    }
                }
                Err(e) => {
                    crate::warn!("Failed to acquire dictionary expander lock: {}", e);
                    ExpansionResult {
                        expanded_text: text,
                        should_press_enter: false,
                    }
                }
            };
            let expanded_text = expansion_result.expanded_text;

            // Try voice command matching if configured (using expanded text)
            let command_handled =
                Self::try_command_matching(&expanded_text, &command_registry, &command_matcher, &action_dispatcher, &command_emitter, &transcription_emitter)
                    .await;

            // Fallback to clipboard if no command was handled (using expanded text)
            // Safety check: don't paste during shutdown
            eprintln!("[PASTE-TRACE] process_recording: command_handled={}, shutting_down={}", command_handled, crate::shutdown::is_shutting_down());
            if !command_handled && !crate::shutdown::is_shutting_down() {
                eprintln!("[PASTE-TRACE] process_recording: about to write to clipboard and paste");
                if let Err(e) = app_handle.clipboard().write_text(&expanded_text) {
                    crate::warn!("Failed to copy to clipboard: {}", e);
                } else {
                    crate::debug!("Transcribed text copied to clipboard");
                    eprintln!("[PASTE-TRACE] process_recording: about to call simulate_paste()");
                    if let Err(e) = simulate_paste() {
                        crate::warn!("Failed to auto-paste: {}", e);
                    } else {
                        crate::debug!("Auto-pasted transcribed text");

                        // Simulate Enter keypress if auto_enter was triggered
                        if expansion_result.should_press_enter {
                            crate::debug!("Auto-enter triggered, simulating Enter keypress");
                            match crate::keyboard::KeyboardSimulator::new() {
                                Ok(mut simulator) => {
                                    if let Err(e) = simulator.simulate_enter_keypress() {
                                        crate::warn!("Failed to simulate enter keypress: {}", e);
                                    } else {
                                        crate::debug!("Successfully simulated Enter keypress");
                                    }
                                }
                                Err(e) => {
                                    crate::warn!("Failed to create keyboard simulator: {}", e);
                                }
                            }
                        }
                    }
                }
            }

            // Always emit transcription_completed with expanded text (whether command handled or not)
            crate::info!("Emitting transcription_completed");
            transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                text: expanded_text,
                duration_ms,
            });

            // Reset transcription state to idle
            if let Err(e) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", e);
            }

            // Clear recording buffer to free memory
            clear_recording_buffer();
        });
    }

    /// Try to match the transcribed text against voice commands
    ///
    /// Returns true if a command was matched and handled, false otherwise.
    async fn try_command_matching(
        text: &str,
        command_registry: &Option<Arc<Mutex<CommandRegistry>>>,
        command_matcher: &Option<Arc<CommandMatcher>>,
        action_dispatcher: &Option<Arc<ActionDispatcher>>,
        command_emitter: &Option<Arc<C>>,
        transcription_emitter: &Arc<T>,
    ) -> bool {
        // Check if all voice command components are configured
        let (registry, matcher, dispatcher, emitter) = match (
            command_registry,
            command_matcher,
            action_dispatcher,
            command_emitter,
        ) {
            (Some(r), Some(m), Some(d), Some(e)) => (r, m, d, e),
            _ => {
                crate::debug!("Voice commands not configured, skipping command matching");
                return false;
            }
        };

        // Local enum to capture match results before releasing the registry lock.
        // IMPORTANT: registry_guard must be dropped before any await to ensure
        // this async block remains Send.
        enum MatchOutcome {
            Matched {
                cmd: crate::voice_commands::registry::CommandDefinition,
                trigger: String,
                confidence: f64,
            },
            Ambiguous {
                candidates: Vec<CommandCandidate>,
            },
            NoMatch,
        }

        // Lock registry, match, extract all needed data, then release lock
        let outcome = {
            let registry_guard = match registry.lock() {
                Ok(g) => g,
                Err(_) => {
                    crate::error!("Failed to lock command registry - lock poisoned");
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: "Internal error: command registry unavailable. Please restart the application.".to_string(),
                    });
                    return false;
                }
            };

            let match_result = matcher.match_input(text, &registry_guard);

            match match_result {
                MatchResult::Exact {
                    command: matched_cmd,
                    ..
                } => match registry_guard.get(matched_cmd.id).cloned() {
                    Some(cmd) => MatchOutcome::Matched {
                        cmd,
                        trigger: matched_cmd.trigger.clone(),
                        confidence: 1.0,
                    },
                    None => MatchOutcome::NoMatch,
                },
                MatchResult::Fuzzy {
                    command: matched_cmd,
                    score,
                    ..
                } => match registry_guard.get(matched_cmd.id).cloned() {
                    Some(cmd) => MatchOutcome::Matched {
                        cmd,
                        trigger: matched_cmd.trigger.clone(),
                        confidence: score,
                    },
                    None => MatchOutcome::NoMatch,
                },
                MatchResult::Ambiguous { candidates } => {
                    let candidate_data: Vec<_> = candidates
                        .iter()
                        .map(|c| CommandCandidate {
                            id: c.command.id.to_string(),
                            trigger: c.command.trigger.clone(),
                            confidence: c.score,
                        })
                        .collect();
                    MatchOutcome::Ambiguous {
                        candidates: candidate_data,
                    }
                }
                MatchResult::NoMatch => MatchOutcome::NoMatch,
            }
            // registry_guard is dropped here - before any await
        };

        match outcome {
            MatchOutcome::Matched {
                cmd,
                trigger,
                confidence,
            } => {
                crate::info!(
                    "Command matched: {} (confidence: {:.2})",
                    trigger, confidence
                );

                // Emit command_matched event
                emitter.emit_command_matched(CommandMatchedPayload {
                    transcription: text.to_string(),
                    command_id: cmd.id.to_string(),
                    trigger: trigger.clone(),
                    confidence,
                });

                // Execute command
                match dispatcher.execute(&cmd).await {
                    Ok(action_result) => {
                        crate::info!("Command executed: {}", action_result.message);
                        emitter.emit_command_executed(CommandExecutedPayload {
                            command_id: cmd.id.to_string(),
                            trigger: trigger.clone(),
                            message: action_result.message,
                        });
                    }
                    Err(action_error) => {
                        crate::error!("Command execution failed: {}", action_error);
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
                crate::info!("Ambiguous match: {} candidates", candidates.len());

                // Emit command_ambiguous event for disambiguation UI
                emitter.emit_command_ambiguous(CommandAmbiguousPayload {
                    transcription: text.to_string(),
                    candidates,
                });
                true // Command matching was handled (ambiguous)
            }
            MatchOutcome::NoMatch => {
                crate::debug!("No command match for: {}", text);
                false // Fall through to clipboard
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::DictionaryEntry;
    use crate::events::{
        CommandAmbiguousPayload, CommandExecutedPayload, CommandFailedPayload,
        CommandMatchedPayload, TranscriptionCompletedPayload, TranscriptionErrorPayload,
        TranscriptionStartedPayload,
    };
    use std::sync::atomic::{AtomicBool, Ordering};

    // Mock transcription emitter for tests
    struct MockTranscriptionEmitter {
        started_called: AtomicBool,
        completed_called: AtomicBool,
        error_called: AtomicBool,
    }

    impl MockTranscriptionEmitter {
        fn new() -> Self {
            Self {
                started_called: AtomicBool::new(false),
                completed_called: AtomicBool::new(false),
                error_called: AtomicBool::new(false),
            }
        }
    }

    impl TranscriptionEventEmitter for MockTranscriptionEmitter {
        fn emit_transcription_started(&self, _payload: TranscriptionStartedPayload) {
            self.started_called.store(true, Ordering::SeqCst);
        }

        fn emit_transcription_completed(&self, _payload: TranscriptionCompletedPayload) {
            self.completed_called.store(true, Ordering::SeqCst);
        }

        fn emit_transcription_error(&self, _payload: TranscriptionErrorPayload) {
            self.error_called.store(true, Ordering::SeqCst);
        }
    }

    // Mock command emitter for tests
    struct MockCommandEmitter;

    impl CommandEventEmitter for MockCommandEmitter {
        fn emit_command_matched(&self, _payload: CommandMatchedPayload) {}
        fn emit_command_executed(&self, _payload: CommandExecutedPayload) {}
        fn emit_command_failed(&self, _payload: CommandFailedPayload) {}
        fn emit_command_ambiguous(&self, _payload: CommandAmbiguousPayload) {}
    }

    #[test]
    fn test_service_skips_transcription_when_model_not_loaded() {
        // This test verifies the early return path when the model is not loaded.
        // We can't fully test process_recording without a Tauri app context,
        // but we can verify the model check logic.
        let model = Arc::new(SharedTranscriptionModel::new());
        assert!(!model.is_loaded(), "Model should not be loaded by default");
        // The actual process_recording would return early due to !is_loaded()
    }

    #[test]
    fn test_mock_emitter_tracks_calls() {
        // Verify our mock emitter properly tracks method calls
        let emitter = MockTranscriptionEmitter::new();

        assert!(!emitter.started_called.load(Ordering::SeqCst));
        emitter.emit_transcription_started(TranscriptionStartedPayload {
            timestamp: "test".to_string(),
        });
        assert!(emitter.started_called.load(Ordering::SeqCst));

        assert!(!emitter.completed_called.load(Ordering::SeqCst));
        emitter.emit_transcription_completed(TranscriptionCompletedPayload {
            text: "test".to_string(),
            duration_ms: 100,
        });
        assert!(emitter.completed_called.load(Ordering::SeqCst));

        assert!(!emitter.error_called.load(Ordering::SeqCst));
        emitter.emit_transcription_error(TranscriptionErrorPayload {
            error: "test error".to_string(),
        });
        assert!(emitter.error_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_dictionary_expander_integration_with_transcription_flow() {
        // This test verifies the dictionary expander correctly transforms text
        // in the same way it would be used in the transcription pipeline.
        //
        // The actual process_recording method requires a Tauri runtime,
        // but we can verify the DictionaryExpander integration pattern here.

        let entries = vec![
            DictionaryEntry {
                id: "1".to_string(),
                trigger: "brb".to_string(),
                expansion: "be right back".to_string(),
                suffix: None,
                auto_enter: false,
                disable_suffix: false,
            },
            DictionaryEntry {
                id: "2".to_string(),
                trigger: "api".to_string(),
                expansion: "API".to_string(),
                suffix: None,
                auto_enter: false,
                disable_suffix: false,
            },
        ];

        let expander = Arc::new(DictionaryExpander::new(&entries));

        // Simulate transcription text that would come from Parakeet
        let transcribed_text = "i need to brb and check the api docs";

        // Apply expansion (same pattern as in process_recording)
        let result = expander.expand(transcribed_text);

        // Verify expansion was applied correctly
        assert_eq!(
            result.expanded_text,
            "i need to be right back and check the API docs"
        );

        // This expanded text would then be:
        // 1. Passed to command matcher
        // 2. Copied to clipboard
        // 3. Included in transcription_completed event payload
    }

    #[test]
    fn test_dictionary_expander_graceful_fallback_no_expander() {
        // When no expander is configured, text should pass through unchanged
        let dictionary_expander: Arc<RwLock<Option<DictionaryExpander>>> =
            Arc::new(RwLock::new(None));

        let text = "i need to brb and check the api docs";

        // Simulate the expansion logic from process_recording (using RwLock pattern)
        let result = match dictionary_expander.read() {
            Ok(guard) => {
                if let Some(ref expander) = *guard {
                    expander.expand(text)
                } else {
                    ExpansionResult {
                        expanded_text: text.to_string(),
                        should_press_enter: false,
                    }
                }
            }
            Err(_) => ExpansionResult {
                expanded_text: text.to_string(),
                should_press_enter: false,
            },
        };

        // Text should be unchanged when no expander is present
        assert_eq!(result.expanded_text, text);
    }

    #[test]
    fn test_dictionary_expander_graceful_fallback_empty_entries() {
        // When expander has no entries, text should pass through unchanged
        let expander = DictionaryExpander::new(&[]);
        let dictionary_expander: Arc<RwLock<Option<DictionaryExpander>>> =
            Arc::new(RwLock::new(Some(expander)));

        let text = "i need to brb and check the api docs";

        // Simulate the expansion logic from process_recording (using RwLock pattern)
        let result = match dictionary_expander.read() {
            Ok(guard) => {
                if let Some(ref exp) = *guard {
                    exp.expand(text)
                } else {
                    ExpansionResult {
                        expanded_text: text.to_string(),
                        should_press_enter: false,
                    }
                }
            }
            Err(_) => ExpansionResult {
                expanded_text: text.to_string(),
                should_press_enter: false,
            },
        };

        // Text should be unchanged when expander has no entries
        assert_eq!(result.expanded_text, text);
    }

    #[test]
    fn test_dictionary_expander_runtime_update() {
        // Test that the RwLock-based expander can be updated at runtime
        let dictionary_expander: Arc<RwLock<Option<DictionaryExpander>>> =
            Arc::new(RwLock::new(None));

        let text = "i need to brb";

        // Initially no expander, text unchanged
        let result1 = match dictionary_expander.read() {
            Ok(guard) => match *guard {
                Some(ref exp) => exp.expand(text),
                None => ExpansionResult {
                    expanded_text: text.to_string(),
                    should_press_enter: false,
                },
            },
            Err(_) => ExpansionResult {
                expanded_text: text.to_string(),
                should_press_enter: false,
            },
        };
        assert_eq!(result1.expanded_text, "i need to brb");

        // Update with new entries (simulating what update_dictionary does)
        let entries = vec![DictionaryEntry {
            id: "1".to_string(),
            trigger: "brb".to_string(),
            expansion: "be right back".to_string(),
            suffix: None,
            auto_enter: false,
            disable_suffix: false,
        }];
        {
            let mut guard = dictionary_expander.write().unwrap();
            *guard = Some(DictionaryExpander::new(&entries));
        }

        // Now expansion should work
        let result2 = match dictionary_expander.read() {
            Ok(guard) => match *guard {
                Some(ref exp) => exp.expand(text),
                None => ExpansionResult {
                    expanded_text: text.to_string(),
                    should_press_enter: false,
                },
            },
            Err(_) => ExpansionResult {
                expanded_text: text.to_string(),
                should_press_enter: false,
            },
        };
        assert_eq!(result2.expanded_text, "i need to be right back");
    }
}

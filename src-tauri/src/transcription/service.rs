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
use crate::turso::TursoClient;
use crate::voice_commands::executor::ActionDispatcher;
use crate::voice_commands::matcher::{CommandMatcher, MatchResult};
use crate::voice_commands::registry::CommandDefinition;
use crate::window_context::ContextResolver;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::sync::Semaphore;

/// Type alias for Turso client state
pub type TursoClientState = Arc<TursoClient>;

/// Maximum concurrent transcriptions allowed
const MAX_CONCURRENT_TRANSCRIPTIONS: usize = 2;

/// Default transcription timeout in seconds
pub const DEFAULT_TRANSCRIPTION_TIMEOUT_SECS: u64 = 60;

/// Simulate Cmd+V paste keystroke on macOS using CoreGraphics
#[cfg(target_os = "macos")]
fn simulate_paste() -> Result<(), String> {
    // Safety check: don't paste during shutdown
    if crate::shutdown::is_shutting_down() {
        crate::debug!("Skipping paste - app is shutting down");
        return Ok(());
    }

    // Centralized synthesis ensures key-up always follows key-down and sequences don't interleave.
    crate::keyboard::synth::simulate_cmd_v_paste()?;

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
    /// Optional Turso client for fetching voice commands
    turso_client: Option<Arc<TursoClient>>,
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
    /// Optional context resolver for window-aware command/dictionary resolution
    context_resolver: Option<Arc<ContextResolver>>,
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
            turso_client: None,
            command_matcher: None,
            action_dispatcher: None,
            command_emitter: None,
            transcription_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
            app_handle,
            transcription_timeout: Duration::from_secs(DEFAULT_TRANSCRIPTION_TIMEOUT_SECS),
            dictionary_expander: Arc::new(RwLock::new(None)),
            context_resolver: None,
        }
    }

    /// Add Turso client for voice command queries (builder pattern)
    pub fn with_turso_client(mut self, client: Arc<TursoClient>) -> Self {
        self.turso_client = Some(client);
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

    /// Add context resolver for window-aware command/dictionary resolution (builder pattern)
    pub fn with_context_resolver(mut self, resolver: Arc<ContextResolver>) -> Self {
        self.context_resolver = Some(resolver);
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
        let turso_client = self.turso_client.clone();
        let command_matcher = self.command_matcher.clone();
        let action_dispatcher = self.action_dispatcher.clone();
        let command_emitter = self.command_emitter.clone();
        let app_handle = self.app_handle.clone();
        let semaphore = self.transcription_semaphore.clone();
        let timeout_duration = self.transcription_timeout;
        let dictionary_expander = self.dictionary_expander.clone();
        let context_resolver = self.context_resolver.clone();

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

            // Clone file_path before it's moved into the closure
            let file_path_for_storage = file_path.clone();

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

            // Store transcription in Turso using storage abstraction (async since we're in async context)
            if let Some(turso) = app_handle.try_state::<TursoClientState>() {
                if let Err(e) = crate::storage::TranscriptionStorage::store(
                    &turso,
                    &file_path_for_storage,
                    &text,
                    duration_ms,
                    &app_handle,
                )
                .await
                {
                    crate::warn!("Failed to store transcription: {}", e);
                }
            }

            // Apply dictionary expansion using context-resolved entries when available
            let expansion_result = {
                // Try context-aware dictionary expansion first
                // Get TursoClient from app state for dictionary entries
                let turso_client: Option<tauri::State<'_, TursoClientState>> =
                    app_handle.try_state();

                let context_entries = match (&context_resolver, &turso_client) {
                    (Some(resolver), Some(client)) => {
                        crate::debug!("[DictionaryExpansion] Context resolver available, attempting context-aware expansion");
                        // Get all dictionary entries from Turso
                        let all_entries = match client.list_dictionary_entries().await {
                            Ok(entries) => {
                                crate::debug!(
                                    "[DictionaryExpansion] Retrieved {} total dictionary entries",
                                    entries.len()
                                );
                                Some(entries)
                            }
                            Err(e) => {
                                crate::warn!("[DictionaryExpansion] Failed to get dictionary entries: {}", e);
                                None
                            }
                        };
                        // Apply context resolver to filter entries
                        match all_entries {
                            Some(all_entries) => {
                                let entries = resolver.get_effective_dictionary(&all_entries);
                                if !entries.is_empty() {
                                    crate::debug!(
                                        "[DictionaryExpansion] Using {} context-resolved entries for expansion",
                                        entries.len()
                                    );
                                    Some(entries)
                                } else {
                                    crate::debug!(
                                        "[DictionaryExpansion] Context resolver returned empty, will fall back to global expander"
                                    );
                                    None
                                }
                            }
                            None => None,
                        }
                    }
                    _ => {
                        crate::debug!("[DictionaryExpansion] No context resolver or Turso client available");
                        None
                    }
                };

                // Use context entries if available, otherwise fall back to global expander
                if let Some(entries) = context_entries {
                    crate::debug!("[DictionaryExpansion] Using context-resolved expander");
                    let context_expander = DictionaryExpander::new(&entries);
                    let result = context_expander.expand(&text);
                    if result.expanded_text != text {
                        crate::debug!(
                            "[DictionaryExpansion] Context-aware expansion applied: '{}' -> '{}'",
                            text,
                            result.expanded_text
                        );
                    } else {
                        crate::debug!("[DictionaryExpansion] No expansion matched in context entries");
                    }
                    result
                } else {
                    // Fall back to global dictionary expander
                    crate::debug!("[DictionaryExpansion] Falling back to global dictionary expander");
                    match dictionary_expander.read() {
                        Ok(guard) => {
                            if let Some(ref expander) = *guard {
                                crate::debug!("[DictionaryExpansion] Global expander available, expanding text");
                                let result = expander.expand(&text);
                                if result.expanded_text != text {
                                    crate::debug!(
                                        "[DictionaryExpansion] Global expansion applied: '{}' -> '{}'",
                                        text,
                                        result.expanded_text
                                    );
                                } else {
                                    crate::debug!("[DictionaryExpansion] No expansion matched in global entries");
                                }
                                result
                            } else {
                                crate::debug!("[DictionaryExpansion] No global expander configured");
                                ExpansionResult {
                                    expanded_text: text,
                                    should_press_enter: false,
                                }
                            }
                        }
                        Err(e) => {
                            crate::warn!("[DictionaryExpansion] Failed to acquire dictionary expander lock: {}", e);
                            ExpansionResult {
                                expanded_text: text,
                                should_press_enter: false,
                            }
                        }
                    }
                }
            };
            let expanded_text = expansion_result.expanded_text;

            // Try voice command matching if configured (using expanded text)
            let command_handled =
                Self::try_command_matching(&expanded_text, &turso_client, &command_matcher, &action_dispatcher, &command_emitter, &transcription_emitter, &context_resolver)
                    .await;

            // Fallback to clipboard if no command was handled (using expanded text)
            // Safety check: don't paste during shutdown
            if !command_handled && !crate::shutdown::is_shutting_down() {
                if let Err(e) = app_handle.clipboard().write_text(&expanded_text) {
                    crate::warn!("Failed to copy to clipboard: {}", e);
                } else {
                    crate::debug!("Transcribed text copied to clipboard");
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
    /// When a context_resolver is provided, uses context-resolved commands for matching.
    async fn try_command_matching(
        text: &str,
        turso_client: &Option<Arc<TursoClient>>,
        command_matcher: &Option<Arc<CommandMatcher>>,
        action_dispatcher: &Option<Arc<ActionDispatcher>>,
        command_emitter: &Option<Arc<C>>,
        transcription_emitter: &Arc<T>,
        context_resolver: &Option<Arc<ContextResolver>>,
    ) -> bool {
        // Check if all voice command components are configured
        let (client, matcher, dispatcher, emitter) = match (
            turso_client,
            command_matcher,
            action_dispatcher,
            command_emitter,
        ) {
            (Some(c), Some(m), Some(d), Some(e)) => (c, m, d, e),
            _ => {
                crate::debug!("Voice commands not configured, skipping command matching");
                return false;
            }
        };

        // Fetch all commands from Turso
        let all_commands = match client.list_voice_commands().await {
            Ok(commands) => commands,
            Err(e) => {
                crate::error!("Failed to fetch voice commands from Turso: {}", e);
                transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                    error: "Failed to load voice commands. Please try again.".to_string(),
                });
                return false;
            }
        };

        // Local enum to capture match results
        enum MatchOutcome {
            Matched {
                cmd: CommandDefinition,
                trigger: String,
                confidence: f64,
            },
            Ambiguous {
                candidates: Vec<CommandCandidate>,
            },
            NoMatch,
        }

        // Get effective commands - either context-resolved or all commands
        let match_result = match context_resolver {
            Some(resolver) => {
                let effective_commands = resolver.get_effective_commands(&all_commands);
                if effective_commands.is_empty() {
                    crate::debug!("No effective commands for current context, falling back to global");
                    matcher.match_commands(text, &all_commands)
                } else {
                    crate::debug!(
                        "Using {} context-resolved commands for matching",
                        effective_commands.len()
                    );
                    matcher.match_commands(text, &effective_commands)
                }
            }
            None => matcher.match_commands(text, &all_commands),
        };

        // Build a lookup map for finding commands by ID
        let commands_by_id: std::collections::HashMap<uuid::Uuid, &CommandDefinition> =
            all_commands.iter().map(|cmd| (cmd.id, cmd)).collect();

        let outcome = match match_result {
            MatchResult::Exact {
                command: matched_cmd,
                ..
            } => match commands_by_id.get(&matched_cmd.id) {
                Some(cmd) => MatchOutcome::Matched {
                    cmd: (*cmd).clone(),
                    trigger: matched_cmd.trigger.clone(),
                    confidence: 1.0,
                },
                None => MatchOutcome::NoMatch,
            },
            MatchResult::Fuzzy {
                command: matched_cmd,
                score,
                ..
            } => match commands_by_id.get(&matched_cmd.id) {
                Some(cmd) => MatchOutcome::Matched {
                    cmd: (*cmd).clone(),
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
#[path = "service_test.rs"]
mod tests;

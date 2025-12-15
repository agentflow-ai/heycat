// Tauri IPC commands module
// This file contains Tauri-specific wrappers and is excluded from coverage.
// The actual logic is in logic.rs which is fully tested.
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod logic;

pub use logic::{RecordingInfo, RecordingStateInfo};
use logic::{
    clear_last_recording_buffer_impl, disable_listening_impl, enable_listening_impl,
    get_last_recording_buffer_impl, get_listening_status_impl, get_recording_state_impl,
    list_recordings_impl, start_recording_impl, stop_recording_impl, transcribe_file_impl,
};

use crate::listening::{ListeningManager, ListeningPipeline, ListeningStatus};

use crate::events::{
    command_events, event_names, listening_events, CommandAmbiguousPayload, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload, ListeningEventEmitter,
    RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::audio::AudioThreadHandle;
use crate::parakeet::TranscriptionManager;
use crate::recording::{AudioData, RecordingManager, RecordingMetadata};
use crate::warn;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Helper macro to emit events with error logging
macro_rules! emit_or_warn {
    ($handle:expr, $event:expr, $payload:expr) => {
        if let Err(e) = $handle.emit($event, $payload) {
            warn!("Failed to emit event '{}': {}", $event, e);
        }
    };
}

/// Type alias for audio thread state
pub type AudioThreadState = Arc<AudioThreadHandle>;

/// Type alias for production state (RecordingManager is Send+Sync)
pub type ProductionState = Arc<Mutex<RecordingManager>>;

/// Type alias for listening manager state
pub type ListeningState = Arc<Mutex<ListeningManager>>;

/// Type alias for listening pipeline state
pub type ListeningPipelineState = Arc<Mutex<ListeningPipeline>>;

/// Type alias for hotkey integration state
pub type HotkeyIntegrationState = Arc<Mutex<crate::hotkey::HotkeyIntegration<TauriEventEmitter, TauriEventEmitter, TauriEventEmitter>>>;

/// Type alias for recording detectors state
pub type RecordingDetectorsState = Arc<Mutex<crate::listening::RecordingDetectors>>;

/// Tauri AppHandle-based event emitter for production use
pub struct TauriEventEmitter {
    app_handle: AppHandle,
}

impl TauriEventEmitter {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl RecordingEventEmitter for TauriEventEmitter {
    fn emit_recording_started(&self, payload: RecordingStartedPayload) {
        emit_or_warn!(self.app_handle, event_names::RECORDING_STARTED, payload);
    }

    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload) {
        emit_or_warn!(self.app_handle, event_names::RECORDING_STOPPED, payload);
    }

    fn emit_recording_error(&self, payload: RecordingErrorPayload) {
        emit_or_warn!(self.app_handle, event_names::RECORDING_ERROR, payload);
    }
}

impl TranscriptionEventEmitter for TauriEventEmitter {
    fn emit_transcription_started(&self, payload: TranscriptionStartedPayload) {
        emit_or_warn!(self.app_handle, event_names::TRANSCRIPTION_STARTED, payload);
    }

    fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload) {
        emit_or_warn!(self.app_handle, event_names::TRANSCRIPTION_COMPLETED, payload);
    }

    fn emit_transcription_error(&self, payload: TranscriptionErrorPayload) {
        emit_or_warn!(self.app_handle, event_names::TRANSCRIPTION_ERROR, payload);
    }
}

impl CommandEventEmitter for TauriEventEmitter {
    fn emit_command_matched(&self, payload: CommandMatchedPayload) {
        emit_or_warn!(self.app_handle, command_events::COMMAND_MATCHED, payload);
    }

    fn emit_command_executed(&self, payload: CommandExecutedPayload) {
        emit_or_warn!(self.app_handle, command_events::COMMAND_EXECUTED, payload);
    }

    fn emit_command_failed(&self, payload: CommandFailedPayload) {
        emit_or_warn!(self.app_handle, command_events::COMMAND_FAILED, payload);
    }

    fn emit_command_ambiguous(&self, payload: CommandAmbiguousPayload) {
        emit_or_warn!(self.app_handle, command_events::COMMAND_AMBIGUOUS, payload);
    }
}

impl ListeningEventEmitter for TauriEventEmitter {
    fn emit_wake_word_detected(&self, payload: listening_events::WakeWordDetectedPayload) {
        emit_or_warn!(
            self.app_handle,
            listening_events::WAKE_WORD_DETECTED,
            payload
        );
    }

    fn emit_listening_started(&self, payload: listening_events::ListeningStartedPayload) {
        emit_or_warn!(
            self.app_handle,
            listening_events::LISTENING_STARTED,
            payload
        );
    }

    fn emit_listening_stopped(&self, payload: listening_events::ListeningStoppedPayload) {
        emit_or_warn!(
            self.app_handle,
            listening_events::LISTENING_STOPPED,
            payload
        );
    }

    fn emit_listening_unavailable(&self, payload: listening_events::ListeningUnavailablePayload) {
        emit_or_warn!(
            self.app_handle,
            listening_events::LISTENING_UNAVAILABLE,
            payload
        );
    }

    fn emit_recording_cancelled(&self, payload: listening_events::RecordingCancelledPayload) {
        emit_or_warn!(
            self.app_handle,
            listening_events::RECORDING_CANCELLED,
            payload
        );
    }
}

/// Start recording audio from the microphone
#[tauri::command]
pub fn start_recording(
    app_handle: AppHandle,
    state: State<'_, ProductionState>,
    audio_thread: State<'_, AudioThreadState>,
) -> Result<(), String> {
    // Check model availability before starting recording (check TDT model for batch transcription)
    let model_available =
        match crate::model::check_model_exists_for_type(crate::model::ModelType::ParakeetTDT) {
            Ok(available) => available,
            Err(e) => {
                warn!("Failed to check model status: {}", e);
                false
            }
        };
    let result = start_recording_impl(state.as_ref(), Some(audio_thread.as_ref()), model_available);

    // Emit event on success for frontend state sync
    if result.is_ok() {
        emit_or_warn!(
            app_handle,
            event_names::RECORDING_STARTED,
            RecordingStartedPayload {
                timestamp: crate::events::current_timestamp(),
            }
        );
    }

    result
}

/// Stop recording and save the audio to a WAV file
#[tauri::command]
pub fn stop_recording(
    app_handle: AppHandle,
    state: State<'_, ProductionState>,
    audio_thread: State<'_, AudioThreadState>,
    listening_state: State<'_, ListeningState>,
) -> Result<RecordingMetadata, String> {
    // Check if listening mode is enabled to determine return state
    let return_to_listening = listening_state
        .lock()
        .map(|lm| lm.is_enabled())
        .unwrap_or(false);

    let result = stop_recording_impl(state.as_ref(), Some(audio_thread.as_ref()), return_to_listening);

    // Emit event on success for frontend state sync
    if let Ok(ref metadata) = result {
        emit_or_warn!(
            app_handle,
            event_names::RECORDING_STOPPED,
            RecordingStoppedPayload {
                metadata: metadata.clone(),
            }
        );
    }

    result
}

/// Get the current recording state
#[tauri::command]
pub fn get_recording_state(state: State<'_, ProductionState>) -> Result<RecordingStateInfo, String> {
    get_recording_state_impl(state.as_ref())
}

/// Get the audio data from the last completed recording for transcription
#[tauri::command]
pub fn get_last_recording_buffer(state: State<'_, ProductionState>) -> Result<AudioData, String> {
    get_last_recording_buffer_impl(state.as_ref())
}

/// Clear the retained last recording buffer to free memory
#[tauri::command]
pub fn clear_last_recording_buffer(state: State<'_, ProductionState>) -> Result<(), String> {
    clear_last_recording_buffer_impl(state.as_ref())
}

/// List all recordings from the app data directory
#[tauri::command]
pub fn list_recordings() -> Result<Vec<RecordingInfo>, String> {
    list_recordings_impl()
}

/// Transcribe an audio file and copy result to clipboard
#[tauri::command]
pub async fn transcribe_file(
    app_handle: AppHandle,
    transcription_manager: State<'_, Arc<TranscriptionManager>>,
    file_path: String,
) -> Result<String, String> {
    // Emit transcription started event
    emit_or_warn!(
        app_handle,
        event_names::TRANSCRIPTION_STARTED,
        TranscriptionStartedPayload {
            timestamp: crate::events::current_timestamp(),
        }
    );

    // Clone what we need for the blocking task
    let manager = transcription_manager.inner().clone();
    let path = file_path.clone();

    // Run transcription on blocking thread pool
    let result = tokio::task::spawn_blocking(move || {
        transcribe_file_impl(&manager, &path)
    })
    .await
    .map_err(|e| format!("Transcription task failed: {}", e))?;

    match result {
        Ok(text) => {
            // Copy to clipboard
            if let Err(e) = app_handle.clipboard().write_text(&text) {
                warn!("Failed to copy transcription to clipboard: {}", e);
            }

            // Emit transcription completed event
            emit_or_warn!(
                app_handle,
                event_names::TRANSCRIPTION_COMPLETED,
                TranscriptionCompletedPayload {
                    text: text.clone(),
                    duration_ms: 0, // Duration not tracked for manual transcription
                }
            );

            Ok(text)
        }
        Err(e) => {
            // Emit transcription error event
            emit_or_warn!(
                app_handle,
                event_names::TRANSCRIPTION_ERROR,
                TranscriptionErrorPayload {
                    error: e.clone(),
                }
            );

            Err(e)
        }
    }
}

// =============================================================================
// Listening Commands
// =============================================================================

/// Enable listening mode (always-on wake word detection)
#[tauri::command]
pub fn enable_listening(
    app_handle: AppHandle,
    listening_state: State<'_, ListeningState>,
    recording_state: State<'_, ProductionState>,
    listening_pipeline: State<'_, ListeningPipelineState>,
    audio_thread: State<'_, AudioThreadState>,
    hotkey_integration: State<'_, HotkeyIntegrationState>,
    recording_detectors: State<'_, RecordingDetectorsState>,
) -> Result<(), String> {
    let emitter = Arc::new(TauriEventEmitter::new(app_handle.clone()));

    // Set wake word callback to start recording when "Hey Cat" is detected
    // The callback captures Arc clones so the original references can be dropped
    let integration_for_callback = hotkey_integration.inner().clone();
    let recording_for_callback = recording_state.inner().clone();
    let detectors_for_callback = recording_detectors.inner().clone();
    let audio_thread_for_callback = audio_thread.inner().clone();
    let emitter_for_callback = Arc::new(TauriEventEmitter::new(app_handle.clone()));

    if let Ok(pipeline_guard) = listening_pipeline.lock() {
        pipeline_guard.set_wake_word_callback(Box::new(move || {
            crate::info!("Wake word detected! Starting recording...");

            // Start recording via HotkeyIntegration
            let recording_started = if let Ok(mut guard) = integration_for_callback.lock() {
                guard.handle_toggle(&recording_for_callback)
            } else {
                crate::error!("Failed to acquire integration lock for wake word callback");
                false
            };

            // If recording started, start the silence/cancel detection
            if recording_started {
                // Get the audio buffer for detection
                if let Ok(rm) = recording_for_callback.lock() {
                    if let Ok(buffer) = rm.get_audio_buffer() {
                        if let Ok(mut det) = detectors_for_callback.lock() {
                            if let Err(e) = det.start_monitoring(
                                buffer,
                                recording_for_callback.clone(),
                                audio_thread_for_callback.clone(),
                                emitter_for_callback.clone(),
                                true, // return_to_listening
                            ) {
                                crate::warn!("Failed to start recording detectors: {}", e);
                            } else {
                                crate::info!("Recording detectors started");
                            }
                        }
                    }
                }
            }
        }));
    }

    let result = enable_listening_impl(
        listening_state.as_ref(),
        recording_state.as_ref(),
        listening_pipeline.as_ref(),
        audio_thread.as_ref(),
        emitter,
    );

    // Emit event on success
    if result.is_ok() {
        emit_or_warn!(
            app_handle,
            listening_events::LISTENING_STARTED,
            listening_events::ListeningStartedPayload {
                timestamp: crate::events::current_timestamp(),
            }
        );
    }

    result
}

/// Disable listening mode
#[tauri::command]
pub fn disable_listening(
    app_handle: AppHandle,
    listening_state: State<'_, ListeningState>,
    recording_state: State<'_, ProductionState>,
    listening_pipeline: State<'_, ListeningPipelineState>,
    audio_thread: State<'_, AudioThreadState>,
) -> Result<(), String> {
    let result = disable_listening_impl(
        listening_state.as_ref(),
        recording_state.as_ref(),
        listening_pipeline.as_ref(),
        audio_thread.as_ref(),
    );

    // Emit event on success
    if result.is_ok() {
        emit_or_warn!(
            app_handle,
            listening_events::LISTENING_STOPPED,
            listening_events::ListeningStoppedPayload {
                timestamp: crate::events::current_timestamp(),
            }
        );
    }

    result
}

/// Get the current listening status
#[tauri::command]
pub fn get_listening_status(
    listening_state: State<'_, ListeningState>,
    recording_state: State<'_, ProductionState>,
) -> Result<ListeningStatus, String> {
    get_listening_status_impl(listening_state.as_ref(), recording_state.as_ref())
}

#[cfg(test)]
mod tests;

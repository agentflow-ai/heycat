// Tauri IPC commands module
// This file contains Tauri-specific wrappers and is excluded from coverage.
// The actual logic is in logic.rs which is fully tested.
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod logic;

pub use logic::{RecordingInfo, RecordingStateInfo};
use logic::{
    clear_last_recording_buffer_impl, get_last_recording_buffer_impl, get_recording_state_impl,
    list_recordings_impl, start_recording_impl, stop_recording_impl,
};

use crate::events::{
    command_events, event_names, CommandAmbiguousPayload, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload, RecordingErrorPayload,
    RecordingEventEmitter, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::audio::AudioThreadHandle;
use crate::recording::{AudioData, RecordingManager, RecordingMetadata};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

/// Type alias for audio thread state
pub type AudioThreadState = Arc<AudioThreadHandle>;

/// Type alias for production state (RecordingManager is Send+Sync)
pub type ProductionState = Arc<Mutex<RecordingManager>>;

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
        let _ = self.app_handle.emit(event_names::RECORDING_STARTED, payload);
    }

    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload) {
        let _ = self.app_handle.emit(event_names::RECORDING_STOPPED, payload);
    }

    fn emit_recording_error(&self, payload: RecordingErrorPayload) {
        let _ = self.app_handle.emit(event_names::RECORDING_ERROR, payload);
    }
}

impl TranscriptionEventEmitter for TauriEventEmitter {
    fn emit_transcription_started(&self, payload: TranscriptionStartedPayload) {
        let _ = self
            .app_handle
            .emit(event_names::TRANSCRIPTION_STARTED, payload);
    }

    fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload) {
        let _ = self
            .app_handle
            .emit(event_names::TRANSCRIPTION_COMPLETED, payload);
    }

    fn emit_transcription_error(&self, payload: TranscriptionErrorPayload) {
        let _ = self
            .app_handle
            .emit(event_names::TRANSCRIPTION_ERROR, payload);
    }
}

impl CommandEventEmitter for TauriEventEmitter {
    fn emit_command_matched(&self, payload: CommandMatchedPayload) {
        let _ = self
            .app_handle
            .emit(command_events::COMMAND_MATCHED, payload);
    }

    fn emit_command_executed(&self, payload: CommandExecutedPayload) {
        let _ = self
            .app_handle
            .emit(command_events::COMMAND_EXECUTED, payload);
    }

    fn emit_command_failed(&self, payload: CommandFailedPayload) {
        let _ = self
            .app_handle
            .emit(command_events::COMMAND_FAILED, payload);
    }

    fn emit_command_ambiguous(&self, payload: CommandAmbiguousPayload) {
        let _ = self
            .app_handle
            .emit(command_events::COMMAND_AMBIGUOUS, payload);
    }
}

/// Start recording audio from the microphone
#[tauri::command]
pub fn start_recording(
    app_handle: AppHandle,
    state: State<'_, ProductionState>,
    audio_thread: State<'_, AudioThreadState>,
) -> Result<(), String> {
    // Check model availability before starting recording
    let model_available = crate::model::check_model_exists().unwrap_or(false);
    let result = start_recording_impl(state.as_ref(), Some(audio_thread.as_ref()), model_available);

    // Emit event on success for frontend state sync
    if result.is_ok() {
        let _ = app_handle.emit(
            event_names::RECORDING_STARTED,
            RecordingStartedPayload {
                timestamp: crate::events::current_timestamp(),
            },
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
) -> Result<RecordingMetadata, String> {
    let result = stop_recording_impl(state.as_ref(), Some(audio_thread.as_ref()));

    // Emit event on success for frontend state sync
    if let Ok(ref metadata) = result {
        let _ = app_handle.emit(
            event_names::RECORDING_STOPPED,
            RecordingStoppedPayload {
                metadata: metadata.clone(),
            },
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

#[cfg(test)]
mod tests;

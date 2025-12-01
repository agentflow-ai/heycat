// Tauri IPC commands module
// This file contains Tauri-specific wrappers and is excluded from coverage.
// The actual logic is in logic.rs which is fully tested.
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod logic;

pub use logic::RecordingStateInfo;
use logic::{
    clear_last_recording_buffer_impl, get_last_recording_buffer_impl, get_recording_state_impl,
    start_recording_impl, stop_recording_impl,
};

use crate::events::{
    event_names, RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload,
    RecordingStoppedPayload,
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

/// Start recording audio from the microphone
#[tauri::command]
pub fn start_recording(
    state: State<'_, ProductionState>,
    audio_thread: State<'_, AudioThreadState>,
) -> Result<(), String> {
    start_recording_impl(state.as_ref(), Some(audio_thread.as_ref()))
}

/// Stop recording and save the audio to a WAV file
#[tauri::command]
pub fn stop_recording(
    state: State<'_, ProductionState>,
    audio_thread: State<'_, AudioThreadState>,
) -> Result<RecordingMetadata, String> {
    stop_recording_impl(state.as_ref(), Some(audio_thread.as_ref()))
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

#[cfg(test)]
mod tests;

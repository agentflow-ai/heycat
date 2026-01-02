//! Recording commands for Tauri IPC.
//!
//! Contains commands for starting, stopping, and managing recordings.

use tauri::{AppHandle, Emitter, Manager, State};

use crate::audio::{encode_wav, AudioDeviceError, SystemFileWriter, StopReason};
use crate::emit_or_warn;
use crate::events::{event_names, RecordingStartedPayload, RecordingStoppedPayload};
use crate::recording::{AudioData, RecordingMetadata};
use crate::turso::events as turso_events;

use super::logic::{
    clear_last_recording_buffer_impl, delete_recording_impl, get_last_recording_buffer_impl,
    get_recording_state_impl, list_recordings_impl, start_recording_impl,
    stop_recording_impl_extended, PaginatedRecordingsResponse, RecordingContextData,
    RecordingStateInfo, MICROPHONE_ERROR_MARKER,
};
use super::{AudioMonitorState, AudioThreadState, ProductionState, TranscriptionServiceState, TursoClientState};

/// Start recording audio from the microphone
///
/// # Arguments
/// * `device_name` - Optional device name to use; falls back to default if not found
#[tauri::command]
pub fn start_recording(
    app_handle: AppHandle,
    state: State<'_, ProductionState>,
    audio_thread: State<'_, AudioThreadState>,
    _monitor_state: State<'_, AudioMonitorState>,
    device_name: Option<String>,
) -> Result<(), String> {
    // Check for audio devices first
    let devices = crate::audio::list_input_devices();
    if devices.is_empty() {
        let error = AudioDeviceError::NoDevicesAvailable;
        emit_or_warn!(app_handle, event_names::AUDIO_DEVICE_ERROR, error.clone());
        return Err(error.to_string());
    }

    // Check if specified device exists (emit warning but don't block - will fallback to default)
    if let Some(ref name) = device_name {
        if !devices.iter().any(|d| &d.name == name) {
            let error = AudioDeviceError::DeviceNotFound {
                device_name: name.clone(),
            };
            emit_or_warn!(app_handle, event_names::AUDIO_DEVICE_ERROR, error);
        }
    }

    // Check model availability before starting recording
    let model_available =
        match crate::model::check_model_exists_for_type(crate::model::ModelType::ParakeetTDT) {
            Ok(available) => available,
            Err(e) => {
                crate::warn!("Failed to check model status: {}", e);
                false
            }
        };

    let result = start_recording_impl(
        state.as_ref(),
        Some(audio_thread.as_ref()),
        model_available,
        device_name,
    );

    match &result {
        Ok(()) => {
            emit_or_warn!(
                app_handle,
                event_names::RECORDING_STARTED,
                RecordingStartedPayload {
                    timestamp: crate::events::current_timestamp(),
                }
            );
        }
        Err(err_msg) => {
            // Use error marker constant instead of fragile string matching
            if err_msg.contains(MICROPHONE_ERROR_MARKER) {
                let error = AudioDeviceError::CaptureError {
                    message: err_msg.clone(),
                };
                emit_or_warn!(app_handle, event_names::AUDIO_DEVICE_ERROR, error);
            }
        }
    }

    result
}

/// Stop recording and save the audio to a WAV file
///
/// After successfully stopping the recording, this command triggers transcription
/// via the TranscriptionService. Also stores recording metadata in Turso.
#[tauri::command]
pub async fn stop_recording(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    state: State<'_, ProductionState>,
    audio_thread: State<'_, AudioThreadState>,
    transcription_service: State<'_, TranscriptionServiceState>,
) -> Result<RecordingMetadata, String> {
    // Get worktree-aware recordings directory
    let worktree_context = app_handle
        .try_state::<crate::worktree::WorktreeState>()
        .and_then(|s| s.context.clone());
    let recordings_dir = crate::paths::get_recordings_dir(worktree_context.as_ref())
        .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat").join("recordings"));

    let result = stop_recording_impl_extended(
        state.as_ref(),
        Some(audio_thread.as_ref()),
        false,
        recordings_dir.clone(),
    );

    if let Ok(ref stop_result) = result {
        let metadata = &stop_result.metadata;

        // Check if recording was stopped due to a device error
        if let Some(StopReason::StreamError) = metadata.stop_reason {
            emit_or_warn!(
                app_handle,
                event_names::AUDIO_DEVICE_ERROR,
                AudioDeviceError::DeviceDisconnected
            );
        }

        // Emit quality warnings to frontend
        for warning in &stop_result.warnings {
            emit_or_warn!(app_handle, event_names::RECORDING_QUALITY_WARNING, warning);
            crate::info!("[DIAGNOSTICS] Emitted quality warning: {:?}", warning.warning_type);
        }

        // Save raw audio file if debug mode was enabled
        if let Some((raw_samples, device_sample_rate)) = &stop_result.raw_audio {
            let raw_writer = SystemFileWriter::new(recordings_dir);
            match encode_wav(raw_samples, *device_sample_rate, &raw_writer) {
                Ok(raw_path) => {
                    let raw_path_with_suffix = raw_path.replace(".wav", "-raw.wav");
                    if let Err(e) = std::fs::rename(&raw_path, &raw_path_with_suffix) {
                        crate::warn!("Failed to rename raw audio file: {}", e);
                    } else {
                        crate::info!("[DIAGNOSTICS] Saved raw audio to: {}", raw_path_with_suffix);
                    }
                }
                Err(e) => {
                    crate::warn!("Failed to save raw audio file: {:?}", e);
                }
            }
        }

        // Store recording metadata in Turso
        if !metadata.file_path.is_empty() {
            let window_context = crate::storage::WindowContext::capture();
            if let Err(e) = crate::storage::RecordingStorage::store(
                turso_client.as_ref(),
                metadata,
                window_context,
                &app_handle,
            )
            .await
            {
                crate::warn!("Failed to store recording in Turso: {}", e);
            } else {
                crate::debug!("Recording metadata stored in Turso (button flow)");
            }
        }

        emit_or_warn!(
            app_handle,
            event_names::RECORDING_STOPPED,
            RecordingStoppedPayload {
                metadata: metadata.clone(),
            }
        );

        // Trigger transcription via TranscriptionService
        if !metadata.file_path.is_empty() {
            transcription_service.process_recording(metadata.file_path.clone());
        }
    }

    result.map(|r| r.metadata)
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

/// List recordings from the app data directory with pagination
///
/// # Arguments
/// * `limit` - Maximum number of recordings to return (default: 20)
/// * `offset` - Number of recordings to skip (default: 0)
#[tauri::command]
pub async fn list_recordings(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<PaginatedRecordingsResponse, String> {
    // Get worktree-aware recordings directory
    let worktree_context = app_handle
        .try_state::<crate::worktree::WorktreeState>()
        .and_then(|s| s.context.clone());
    let recordings_dir = crate::paths::get_recordings_dir(worktree_context.as_ref())
        .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat").join("recordings"));

    // Fetch recording context from Turso
    let mut recording_context: std::collections::HashMap<String, RecordingContextData> =
        std::collections::HashMap::new();

    if let Ok(turso_recordings) = turso_client.list_recordings().await {
        let mut file_to_recording_id: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for recording in &turso_recordings {
            file_to_recording_id.insert(recording.file_path.clone(), recording.id.clone());
            recording_context.insert(
                recording.file_path.clone(),
                RecordingContextData {
                    transcription: None,
                    active_window_app_name: recording.active_window_app_name.clone(),
                    active_window_bundle_id: recording.active_window_bundle_id.clone(),
                    active_window_title: recording.active_window_title.clone(),
                },
            );
        }

        if let Ok(all_transcriptions) = turso_client.list_transcriptions().await {
            for trans in all_transcriptions {
                for (file_path, recording_id) in &file_to_recording_id {
                    if *recording_id == trans.recording_id {
                        if let Some(ctx) = recording_context.get_mut(file_path) {
                            ctx.transcription = Some(trans.text.clone());
                        }
                        break;
                    }
                }
            }
        }
    }

    list_recordings_impl(recordings_dir, limit, offset, recording_context)
}

/// Delete a recording file
///
/// Also removes recording metadata from Turso.
#[tauri::command]
pub async fn delete_recording(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    file_path: String,
) -> Result<(), String> {
    // Try to delete from Turso first
    if let Err(e) = turso_client.delete_recording_by_path(&file_path).await {
        crate::debug!("Turso recording delete (may not exist): {}", e);
    } else {
        turso_events::emit_recordings_updated(&app_handle, "delete", Some(&file_path));
    }

    delete_recording_impl(&file_path)
}

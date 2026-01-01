// Tauri IPC commands module
// This file contains Tauri-specific wrappers and is excluded from coverage.
// The actual logic is in logic.rs which is fully tested.
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod common;
pub mod dictionary;
pub mod logic;
pub mod window_context;

// Re-export TauriEventEmitter from common module for backward compatibility
pub use common::TauriEventEmitter;

pub use logic::{PaginatedRecordingsResponse, RecordingContextData, RecordingStateInfo};
use logic::{
    clear_last_recording_buffer_impl, delete_recording_impl,
    get_last_recording_buffer_impl,
    get_recording_state_impl, list_recordings_impl, start_recording_impl,
    stop_recording_impl_extended, transcribe_file_impl,
};

use crate::events::{
    event_names, RecordingErrorPayload, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionStartedPayload,
};
use crate::audio::{AudioDeviceError, AudioInputDevice, AudioThreadHandle, StopReason, encode_wav, SystemFileWriter};
use crate::parakeet::SharedTranscriptionModel;
use crate::recording::{AudioData, RecordingManager, RecordingMetadata};
use crate::turso::{events as turso_events, TursoClient};
use crate::window_context::get_active_window;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Type alias for Turso client state
pub type TursoClientState = Arc<TursoClient>;

// Use the emit_or_warn macro from common module (re-exported from crate root)
use crate::emit_or_warn;

// Re-export get_settings_file for backward compatibility
pub use common::get_settings_file;

/// Type alias for audio thread state
pub type AudioThreadState = Arc<AudioThreadHandle>;

/// Type alias for production state (RecordingManager is Send+Sync)
pub type ProductionState = Arc<Mutex<RecordingManager>>;

/// Type alias for hotkey integration state
pub type HotkeyIntegrationState = Arc<Mutex<crate::hotkey::HotkeyIntegration<TauriEventEmitter, TauriEventEmitter, TauriEventEmitter>>>;

/// Type alias for transcription service state
pub type TranscriptionServiceState = Arc<crate::transcription::RecordingTranscriptionService<TauriEventEmitter, TauriEventEmitter>>;

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
            // Device not found - emit event but continue with fallback to default
            let error = AudioDeviceError::DeviceNotFound {
                device_name: name.clone(),
            };
            emit_or_warn!(app_handle, event_names::AUDIO_DEVICE_ERROR, error);
            // Note: We don't return here - the backend will fallback to default device
            // This allows recording to continue even if the preferred device is unavailable
        }
    }

    // Note: Audio monitor uses unified SharedAudioEngine with capture, so no need to stop it.
    // Level monitoring continues during recording via the shared engine.

    // Check model availability before starting recording (check TDT model for batch transcription)
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
            // Emit audio device error for microphone access failures
            if err_msg.contains("microphone") {
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
/// via the TranscriptionService. This enables button-initiated recordings to get
/// the same transcription flow as hotkey-initiated recordings.
/// Also stores recording metadata in Turso.
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

    let result = stop_recording_impl_extended(state.as_ref(), Some(audio_thread.as_ref()), false, recordings_dir.clone());

    // Handle success case
    if let Ok(ref stop_result) = result {
        let metadata = &stop_result.metadata;

        // Check if recording was stopped due to a device error and emit appropriate event
        // Other stop reasons (BufferFull, SilenceAfterSpeech, etc.) are not device errors
        if let Some(StopReason::StreamError) = metadata.stop_reason {
            emit_or_warn!(
                app_handle,
                event_names::AUDIO_DEVICE_ERROR,
                AudioDeviceError::DeviceDisconnected
            );
        }

        // Emit quality warnings to frontend
        for warning in &stop_result.warnings {
            emit_or_warn!(
                app_handle,
                event_names::RECORDING_QUALITY_WARNING,
                warning
            );
            crate::info!("[DIAGNOSTICS] Emitted quality warning: {:?}", warning.warning_type);
        }

        // Save raw audio file if debug mode was enabled
        if let Some((raw_samples, device_sample_rate)) = &stop_result.raw_audio {
            let raw_writer = SystemFileWriter::new(recordings_dir);
            match encode_wav(raw_samples, *device_sample_rate, &raw_writer) {
                Ok(raw_path) => {
                    // Rename to add -raw suffix
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
            // Capture active window info
            let (app_name, bundle_id, title) = match get_active_window() {
                Ok(info) => (
                    Some(info.app_name),
                    info.bundle_id,
                    info.window_title,
                ),
                Err(e) => {
                    crate::debug!("Could not get active window info: {}", e);
                    (None, None, None)
                }
            };

            let recording_id = uuid::Uuid::new_v4().to_string();
            if let Err(e) = turso_client
                .add_recording(
                    recording_id.clone(),
                    metadata.file_path.clone(),
                    metadata.duration_secs,
                    metadata.sample_count as u64,
                    metadata.stop_reason.clone(),
                    app_name,
                    bundle_id,
                    title,
                )
                .await
            {
                crate::warn!("Failed to store recording in Turso: {}", e);
            } else {
                crate::debug!("Recording metadata stored in Turso");
                // Emit recordings_updated event
                turso_events::emit_recordings_updated(&app_handle, "add", Some(&recording_id));
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
        // This is what enables button-initiated recordings to get transcribed
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

    // Fetch recording context from Turso to merge with recordings
    let mut recording_context: std::collections::HashMap<String, RecordingContextData> = std::collections::HashMap::new();

    // Get all recordings from Turso
    if let Ok(turso_recordings) = turso_client.list_recordings().await {
        // Build file_path -> recording_id map and populate context
        let mut file_to_recording_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for recording in &turso_recordings {
            file_to_recording_id.insert(recording.file_path.clone(), recording.id.clone());

            // Initialize context with window info
            recording_context.insert(recording.file_path.clone(), RecordingContextData {
                transcription: None, // Will be filled in below
                active_window_app_name: recording.active_window_app_name.clone(),
                active_window_bundle_id: recording.active_window_bundle_id.clone(),
                active_window_title: recording.active_window_title.clone(),
            });
        }

        // Get all transcriptions and add to context
        if let Ok(all_transcriptions) = turso_client.list_transcriptions().await {
            for trans in all_transcriptions {
                // Find the file_path for this transcription's recording_id
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
    // Try to delete from Turso first (non-blocking, ignore errors)
    if let Err(e) = turso_client.delete_recording_by_path(&file_path).await {
        crate::debug!("Turso recording delete (may not exist): {}", e);
    } else {
        // Emit recordings_updated event on successful delete
        turso_events::emit_recordings_updated(&app_handle, "delete", Some(&file_path));
    }

    // Delete the actual file
    delete_recording_impl(&file_path)
}

/// Transcribe an audio file and copy result to clipboard
///
/// Also stores the transcription in Turso.
#[tauri::command]
pub async fn transcribe_file(
    app_handle: AppHandle,
    turso_client: State<'_, TursoClientState>,
    shared_model: State<'_, Arc<SharedTranscriptionModel>>,
    file_path: String,
) -> Result<String, String> {
    // Emit transcription started event
    let start_time = std::time::Instant::now();
    emit_or_warn!(
        app_handle,
        event_names::TRANSCRIPTION_STARTED,
        TranscriptionStartedPayload {
            timestamp: crate::events::current_timestamp(),
        }
    );

    // Clone what we need for the blocking task
    let model = shared_model.inner().clone();
    let path = file_path.clone();

    // Run transcription on blocking thread pool
    let result = tokio::task::spawn_blocking(move || {
        transcribe_file_impl(&model, &path)
    })
    .await
    .map_err(|e| format!("Transcription task failed: {}", e))?;

    match result {
        Ok(text) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;

            // Copy to clipboard
            if let Err(e) = app_handle.clipboard().write_text(&text) {
                crate::warn!("Failed to copy transcription to clipboard: {}", e);
            }

            // Store transcription in Turso
            // Find recording ID for this file path
            if let Ok(Some(recording)) = turso_client.get_recording_by_path(&file_path).await {
                let recording_id = recording.id.clone();
                let transcription_id = uuid::Uuid::new_v4().to_string();
                if let Err(e) = turso_client
                    .add_transcription(
                        transcription_id.clone(),
                        recording_id.clone(),
                        text.clone(),
                        None, // language not detected
                        "parakeet-tdt".to_string(),
                        duration_ms,
                    )
                    .await
                {
                    crate::warn!("Failed to store transcription in Turso: {}", e);
                } else {
                    crate::debug!("Transcription stored in Turso");
                    // Emit transcriptions_updated event
                    turso_events::emit_transcriptions_updated(&app_handle, "add", Some(&transcription_id), Some(&recording_id));
                }
            } else {
                crate::debug!("No Turso recording found for path: {}", file_path);
            }

            // Emit transcription completed event
            emit_or_warn!(
                app_handle,
                event_names::TRANSCRIPTION_COMPLETED,
                TranscriptionCompletedPayload {
                    text: text.clone(),
                    duration_ms,
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
// Audio Device Commands
// =============================================================================

/// List all available audio input devices
///
/// Returns a list of audio input devices sorted with the default device first.
/// Returns an empty array (not an error) when no devices are available.
#[tauri::command]
pub fn list_audio_devices() -> Vec<AudioInputDevice> {
    crate::audio::list_input_devices()
}

/// Type alias for audio monitor state (the thread handle)
pub type AudioMonitorState = Arc<crate::audio::AudioMonitorHandle>;

/// Start audio level monitoring for device testing
///
/// Starts capturing audio from the specified device and emits "audio-level" events
/// with the current input level (0-100). Used for visual feedback in the device selector.
#[tauri::command]
pub fn start_audio_monitor(
    app_handle: AppHandle,
    monitor_state: State<'_, AudioMonitorState>,
    device_name: Option<String>,
) -> Result<(), String> {
    // Start monitoring and get the level receiver
    let level_rx = monitor_state.start(device_name)?;

    // Spawn a thread to forward levels to frontend
    // (Receiver is not Clone, so we need a dedicated thread)
    std::thread::spawn(move || {
        while let Ok(level) = level_rx.recv() {
            let _ = app_handle.emit(event_names::AUDIO_LEVEL, level);
        }
        // Channel closed - monitoring stopped
    });

    Ok(())
}

/// Stop audio level monitoring
///
/// Stops capturing audio and releasing the device.
#[tauri::command]
pub fn stop_audio_monitor(monitor_state: State<'_, AudioMonitorState>) -> Result<(), String> {
    monitor_state.stop()
}

/// Initialize audio monitor at app startup
///
/// Pre-warms the AVAudioEngine so that audio settings UI is instant when opened.
/// This is called during app initialization to eliminate UI jank.
/// Gracefully returns Ok if no audio devices are available.
#[tauri::command]
pub fn init_audio_monitor(
    app_handle: AppHandle,
    monitor_state: State<'_, AudioMonitorState>,
) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    // Check if any audio devices are available
    let devices = crate::audio::list_input_devices();
    if devices.is_empty() {
        crate::info!("No audio devices available, skipping audio monitor pre-initialization");
        return Ok(());
    }

    // Read saved device from settings store
    let settings_file = get_settings_file(&app_handle);
    let device_name = app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("audio.selectedDevice"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    // Pre-initialize the audio engine
    monitor_state.init(device_name)
}

// =============================================================================
// Hotkey Management Commands
// =============================================================================

/// Type alias for hotkey service state (uses dynamic backend)
pub type HotkeyServiceState = crate::hotkey::HotkeyServiceDyn;

/// Suspend the global recording shortcut
///
/// Temporarily unregisters the recording shortcut to allow the webview to capture
/// keyboard events (e.g., when recording a new shortcut in settings).
#[tauri::command]
pub fn suspend_recording_shortcut(
    app_handle: AppHandle,
    service: State<'_, HotkeyServiceState>,
) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    // Get current shortcut from settings
    let settings_file = get_settings_file(&app_handle);
    let shortcut = app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("hotkey.recordingShortcut"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    if let Some(shortcut) = shortcut {
        crate::info!("Suspending recording shortcut: {}", shortcut);
        service.backend.unregister(&shortcut).map_err(|e| e.to_string())
    } else {
        crate::info!("No recording shortcut to suspend");
        Ok(())
    }
}

/// Resume the global recording shortcut
///
/// Re-registers the recording shortcut after it was suspended.
/// Requires a shortcut to be set in settings (user sets one during onboarding).
#[tauri::command]
pub fn resume_recording_shortcut(
    app_handle: AppHandle,
    service: State<'_, HotkeyServiceState>,
    integration: State<'_, HotkeyIntegrationState>,
    recording_state: State<'_, ProductionState>,
) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    // Get shortcut from settings - must be set by user during onboarding
    let settings_file = get_settings_file(&app_handle);
    let shortcut = app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("hotkey.recordingShortcut"))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| "No recording shortcut configured".to_string())?;

    crate::info!("Resuming recording shortcut: {}", shortcut);

    // Clone the Arcs for the callback closure
    let integration_clone = integration.inner().clone();
    let state_clone = recording_state.inner().clone();
    let app_handle_clone = app_handle.clone();

    service
        .backend
        .register(&shortcut, Box::new(move || {
            crate::debug!("Hotkey pressed!");
            match integration_clone.lock() {
                Ok(mut guard) => {
                    guard.handle_toggle(&state_clone);
                }
                Err(e) => {
                    crate::error!("Failed to acquire integration lock: {}", e);
                    let _ = app_handle_clone.emit(
                        crate::events::event_names::RECORDING_ERROR,
                        crate::events::RecordingErrorPayload {
                            message: "Internal error: please restart the application".to_string(),
                        },
                    );
                }
            }
        }))
        .map_err(|e| e.to_string())
}

/// Update the global recording shortcut
///
/// Unregisters the current shortcut and registers a new one.
/// The new shortcut is persisted to settings.
#[tauri::command]
pub fn update_recording_shortcut(
    app_handle: AppHandle,
    service: State<'_, HotkeyServiceState>,
    integration: State<'_, HotkeyIntegrationState>,
    recording_state: State<'_, ProductionState>,
    new_shortcut: String,
) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    crate::info!("Updating recording shortcut to: {}", new_shortcut);

    // Get current shortcut from settings to unregister it (if any)
    let settings_file = get_settings_file(&app_handle);
    let current_shortcut = app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("hotkey.recordingShortcut"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    // Unregister current shortcut if one exists
    if let Some(ref current) = current_shortcut {
        if let Err(e) = service.backend.unregister(current) {
            crate::warn!("Failed to unregister old shortcut '{}': {}", current, e);
            // Continue anyway - the old shortcut might not be registered
        }
    }

    // Clone the Arcs for the callback closure
    let integration_clone = integration.inner().clone();
    let state_clone = recording_state.inner().clone();
    let app_handle_clone = app_handle.clone();

    // Register new shortcut
    service
        .backend
        .register(&new_shortcut, Box::new(move || {
            crate::debug!("Hotkey pressed!");
            match integration_clone.lock() {
                Ok(mut guard) => {
                    guard.handle_toggle(&state_clone);
                }
                Err(e) => {
                    crate::error!("Failed to acquire integration lock: {}", e);
                    let _ = app_handle_clone.emit(
                        crate::events::event_names::RECORDING_ERROR,
                        crate::events::RecordingErrorPayload {
                            message: "Internal error: please restart the application".to_string(),
                        },
                    );
                }
            }
        }))
        .map_err(|e| format!("Failed to register new shortcut: {}", e))?;

    // Save to settings
    if let Ok(store) = app_handle.store(&settings_file) {
        store.set("hotkey.recordingShortcut", serde_json::json!(new_shortcut));
        if let Err(e) = store.save() {
            crate::warn!("Failed to persist settings: {}", e);
        }
    }

    crate::info!("Recording shortcut updated successfully to: {}", new_shortcut);
    Ok(())
}

/// Get the current recording shortcut from settings
///
/// Returns the configured shortcut, or empty string if none is set.
/// User sets a shortcut during onboarding.
#[tauri::command]
pub fn get_recording_shortcut(app_handle: AppHandle) -> String {
    use tauri_plugin_store::StoreExt;

    let settings_file = get_settings_file(&app_handle);
    app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("hotkey.recordingShortcut"))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// Get the current recording mode from settings
///
/// Returns the configured recording mode: "toggle" (default) or "push-to-talk".
#[tauri::command]
pub fn get_recording_mode(app_handle: AppHandle) -> crate::hotkey::RecordingMode {
    use tauri_plugin_store::StoreExt;

    let settings_file = get_settings_file(&app_handle);
    app_handle
        .store(&settings_file)
        .ok()
        .and_then(|store| store.get("shortcuts.recordingMode"))
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// Set the recording mode in settings
///
/// Persists the recording mode to settings. Rejects if recording is currently active.
///
/// # Arguments
/// * `mode` - The new recording mode: "toggle" or "push-to-talk"
///
/// # Errors
/// Returns an error if recording is currently active (not Idle state).
#[tauri::command]
pub fn set_recording_mode(
    app_handle: AppHandle,
    state: State<'_, ProductionState>,
    integration: State<'_, HotkeyIntegrationState>,
    mode: crate::hotkey::RecordingMode,
) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    // Check if recording is active
    let manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;

    let current_state = manager.get_state();
    if current_state != crate::recording::RecordingState::Idle {
        return Err("Cannot change recording mode while recording is active.".to_string());
    }
    drop(manager); // Release lock before saving

    // Update HotkeyIntegration in memory for immediate effect
    let mut integration_guard = integration.lock().map_err(|_| {
        "Unable to access hotkey integration. Please try again or restart the application."
    })?;
    integration_guard.set_recording_mode(mode);
    drop(integration_guard);

    // Persist to settings for restart persistence
    let settings_file = get_settings_file(&app_handle);
    if let Ok(store) = app_handle.store(&settings_file) {
        store.set("shortcuts.recordingMode", serde_json::to_value(&mode).unwrap_or_default());
        if let Err(e) = store.save() {
            crate::warn!("Failed to persist settings: {}", e);
            return Err(format!("Failed to save settings: {}", e));
        }
    } else {
        return Err("Failed to access settings store.".to_string());
    }

    crate::info!("Recording mode updated to: {:?}", mode);
    Ok(())
}

// =============================================================================
// Keyboard Capture Commands (for fn key and special key recording)
// =============================================================================

/// Type alias for keyboard capture state
pub type KeyboardCaptureState = Arc<Mutex<crate::keyboard_capture::KeyboardCapture>>;

/// Start capturing keyboard events for shortcut recording
///
/// This uses CGEventTap to capture all keyboard events including the fn key,
/// media keys (volume, brightness, playback), and left/right modifier distinction,
/// which JavaScript's KeyboardEvent API cannot detect. Captured keys are
/// emitted via the "shortcut_key_captured" event.
///
/// Requires Accessibility permission on macOS (System Settings > Privacy & Security > Accessibility).
#[tauri::command]
pub fn start_shortcut_recording(
    app_handle: AppHandle,
    capture_state: State<'_, KeyboardCaptureState>,
) -> Result<(), String> {
    crate::info!("Starting shortcut recording...");

    let mut capture = capture_state.lock().map_err(|e| e.to_string())?;

    if capture.is_running() {
        return Err("Shortcut recording is already running".to_string());
    }

    let app_handle_clone = app_handle.clone();
    capture.start(move |event| {
        // Emit the captured key event to the frontend
        if let Err(e) = app_handle_clone.emit(event_names::SHORTCUT_KEY_CAPTURED, &event) {
            crate::warn!("Failed to emit shortcut_key_captured event: {}", e);
        }
    })?;

    crate::info!("Shortcut recording started");
    Ok(())
}

/// Stop capturing keyboard events
#[tauri::command]
pub fn stop_shortcut_recording(
    capture_state: State<'_, KeyboardCaptureState>,
) -> Result<(), String> {
    crate::info!("Stopping shortcut recording...");

    let mut capture = capture_state.lock().map_err(|e| e.to_string())?;
    capture.stop()?;

    crate::info!("Shortcut recording stopped");
    Ok(())
}

/// Open System Preferences to the Accessibility pane
///
/// This allows users to grant the Accessibility permission required
/// for fn key and media key capture in the shortcut editor.
#[tauri::command]
pub fn open_accessibility_preferences() -> Result<(), String> {
    crate::info!("Opening Accessibility preferences...");

    crate::keyboard_capture::permissions::open_accessibility_settings().map_err(|e| {
        crate::error!("Failed to open preferences: {}", e);
        e
    })?;

    crate::info!("Opened Accessibility preferences");
    Ok(())
}

// =============================================================================
// Transcription Storage Commands (Turso)
// =============================================================================

/// Transcription record for frontend consumption
#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscriptionInfo {
    pub id: String,
    pub recording_id: String,
    pub text: String,
    pub language: Option<String>,
    pub model_version: String,
    pub duration_ms: u64,
    pub created_at: String,
}

/// List all transcriptions from Turso
///
/// Returns all stored transcriptions.
#[tauri::command]
pub async fn list_transcriptions(
    turso_client: State<'_, TursoClientState>,
) -> Result<Vec<TranscriptionInfo>, String> {
    turso_client
        .list_transcriptions()
        .await
        .map(|transcriptions| {
            transcriptions
                .into_iter()
                .map(|t| TranscriptionInfo {
                    id: t.id,
                    recording_id: t.recording_id,
                    text: t.text,
                    language: t.language,
                    model_version: t.model_version,
                    duration_ms: t.duration_ms,
                    created_at: t.created_at,
                })
                .collect()
        })
        .map_err(|e| format!("Failed to list transcriptions: {}", e))
}

/// Get transcriptions for a specific recording
///
/// Returns all transcriptions linked to the given recording ID.
#[tauri::command]
pub async fn get_transcriptions_by_recording(
    turso_client: State<'_, TursoClientState>,
    recording_id: String,
) -> Result<Vec<TranscriptionInfo>, String> {
    turso_client
        .get_transcriptions_by_recording(&recording_id)
        .await
        .map(|transcriptions| {
            transcriptions
                .into_iter()
                .map(|t| TranscriptionInfo {
                    id: t.id,
                    recording_id: t.recording_id,
                    text: t.text,
                    language: t.language,
                    model_version: t.model_version,
                    duration_ms: t.duration_ms,
                    created_at: t.created_at,
                })
                .collect()
        })
        .map_err(|e| format!("Failed to get transcriptions: {}", e))
}

// =============================================================================
// Worktree Commands
// =============================================================================

/// Get the settings file name for the current worktree context
///
/// Returns the appropriate settings file name based on worktree context:
/// - `settings-{identifier}.json` when running in a worktree
/// - `settings.json` when running in main repository
///
/// This enables worktree-specific settings isolation so multiple heycat
/// instances can run with different configurations (e.g., different hotkeys).
#[tauri::command]
pub fn get_settings_file_name(worktree_state: State<'_, crate::worktree::WorktreeState>) -> String {
    worktree_state.settings_file_name()
}

/// Show the main window, close the splash window, and give main focus
///
/// Called by the frontend when the app is ready to be displayed (e.g., after
/// initialization completes). This enables a seamless splash-to-app transition.
///
/// Includes error recovery with retry logic for splash window operations.
#[tauri::command]
pub fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    // Show the main window first (before closing splash) for smoother UX
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    window.show().map_err(|e| format!("Failed to show window: {}", e))?;
    window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;

    crate::info!("Main window shown and focused");

    // Close the splash window with retry logic
    if let Some(splash) = app_handle.get_webview_window("splash") {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        const RETRY_DELAY_MS: u64 = 50;

        loop {
            attempts += 1;
            match splash.close() {
                Ok(()) => {
                    crate::debug!("Splash window closed");
                    break;
                }
                Err(e) => {
                    if attempts >= MAX_ATTEMPTS {
                        // Log warning but don't fail - main window is already visible
                        crate::warn!(
                            "Failed to close splash window after {} attempts: {}",
                            attempts,
                            e
                        );
                        break;
                    }
                    crate::debug!(
                        "Splash close attempt {} failed, retrying: {}",
                        attempts,
                        e
                    );
                    std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;

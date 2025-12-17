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

use crate::listening::{ListeningManager, ListeningPipeline, ListeningStatus, WakeWordEvent};

use crate::events::{
    command_events, event_names, listening_events, CommandAmbiguousPayload, CommandEventEmitter,
    CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload, ListeningEventEmitter,
    RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::audio::{AudioDeviceError, AudioInputDevice, AudioThreadHandle, StopReason};
use crate::parakeet::SharedTranscriptionModel;
use crate::recording::{AudioData, RecordingManager, RecordingMetadata, RecordingState};
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
}

/// Start recording audio from the microphone
///
/// # Arguments
/// * `device_name` - Optional device name to use; falls back to default if not found
#[tauri::command]
pub fn start_recording(
    app_handle: AppHandle,
    state: State<'_, ProductionState>,
    audio_thread: State<'_, AudioThreadState>,
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

    // Check model availability before starting recording (check TDT model for batch transcription)
    let model_available =
        match crate::model::check_model_exists_for_type(crate::model::ModelType::ParakeetTDT) {
            Ok(available) => available,
            Err(e) => {
                warn!("Failed to check model status: {}", e);
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
        // Check if recording was stopped due to a device error and emit appropriate event
        if let Some(ref reason) = metadata.stop_reason {
            match reason {
                StopReason::StreamError => {
                    emit_or_warn!(
                        app_handle,
                        event_names::AUDIO_DEVICE_ERROR,
                        AudioDeviceError::DeviceDisconnected
                    );
                }
                _ => {} // Other stop reasons (BufferFull, SilenceAfterSpeech, etc.) are not device errors
            }
        }

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
    shared_model: State<'_, Arc<SharedTranscriptionModel>>,
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
    device_name: Option<String>,
) -> Result<(), String> {
    let emitter = Arc::new(TauriEventEmitter::new(app_handle.clone()));

    // Subscribe to wake word events before starting the pipeline
    // This replaces the callback mechanism with a safer async event channel
    let event_rx = {
        let mut pipeline = listening_pipeline.lock().map_err(|_| {
            crate::error!("Failed to lock listening pipeline for event subscription");
            "Unable to access listening pipeline."
        })?;
        pipeline.subscribe_events()
    };

    // Spawn an async task to handle wake word events
    // This runs separately from the analysis thread, eliminating deadlock risk
    let listening_pipeline_for_handler = listening_pipeline.inner().clone();
    let recording_for_handler = recording_state.inner().clone();
    let detectors_for_handler = recording_detectors.inner().clone();
    let audio_thread_for_handler = audio_thread.inner().clone();
    let app_handle_for_handler = app_handle.clone();
    let emitter_for_handler = Arc::new(TauriEventEmitter::new(app_handle.clone()));
    let hotkey_for_handler = hotkey_integration.inner().clone();

    tauri::async_runtime::spawn(async move {
        handle_wake_word_events(
            event_rx,
            listening_pipeline_for_handler,
            recording_for_handler,
            detectors_for_handler,
            audio_thread_for_handler,
            app_handle_for_handler,
            emitter_for_handler,
            hotkey_for_handler,
        ).await;
    });

    let result = enable_listening_impl(
        listening_state.as_ref(),
        recording_state.as_ref(),
        listening_pipeline.as_ref(),
        audio_thread.as_ref(),
        emitter,
        device_name,
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

/// Handle wake word events from the listening pipeline
///
/// This async function processes events from the wake word event channel,
/// running separately from the analysis thread to eliminate deadlock risk.
/// It replaces the direct callback invocation with a safe async pattern.
async fn handle_wake_word_events(
    mut event_rx: tokio::sync::mpsc::Receiver<WakeWordEvent>,
    listening_pipeline: ListeningPipelineState,
    recording_state: ProductionState,
    recording_detectors: RecordingDetectorsState,
    audio_thread: AudioThreadState,
    app_handle: AppHandle,
    emitter: Arc<TauriEventEmitter>,
    hotkey_integration: HotkeyIntegrationState,
) {
    crate::info!("[event_handler] Wake word event handler started");

    while let Some(event) = event_rx.recv().await {
        match event {
            WakeWordEvent::Detected { text, confidence } => {
                crate::info!(
                    "[event_handler] Wake word detected: '{}' (confidence: {:.2})",
                    text, confidence
                );
                // Read selected device from persistent settings store
                use tauri_plugin_store::StoreExt;
                let device_name = app_handle
                    .store("settings.json")
                    .ok()
                    .and_then(|store| store.get("audio.selectedDevice"))
                    .and_then(|v| v.as_str().map(|s| s.to_string()));
                handle_wake_word_detected(
                    &listening_pipeline,
                    &recording_state,
                    &recording_detectors,
                    &audio_thread,
                    &app_handle,
                    &emitter,
                    &hotkey_integration,
                    device_name,
                );
            }
            WakeWordEvent::Unavailable { reason } => {
                crate::warn!("[event_handler] Listening became unavailable: {}", reason);
                // The pipeline already emits the frontend event, we just log here
            }
            WakeWordEvent::Error { message } => {
                crate::warn!("[event_handler] Wake word detection error: {}", message);
                // Errors are transient, the pipeline continues
            }
        }
    }

    crate::info!("[event_handler] Wake word event handler exiting (channel closed)");
}

/// Handle a wake word detected event - start recording
///
/// This function contains the logic previously in the wake word callback,
/// now safely executing in an async context outside the analysis thread.
fn handle_wake_word_detected(
    listening_pipeline: &ListeningPipelineState,
    recording_state: &ProductionState,
    recording_detectors: &RecordingDetectorsState,
    audio_thread: &AudioThreadState,
    app_handle: &AppHandle,
    emitter: &Arc<TauriEventEmitter>,
    hotkey_integration: &HotkeyIntegrationState,
    device_name: Option<String>,
) {
    crate::info!("Wake word detected! Stopping pipeline and starting recording...");

    // 1. Stop the listening pipeline and get the buffer for handoff
    let shared_buffer = {
        // Use try_lock to avoid deadlock with hotkey recording flow:
        // If hotkey is stopping the pipeline, it holds the lock and waits for us.
        // If we block on lock(), we'd deadlock. try_lock fails gracefully instead.
        let mut pipeline = match listening_pipeline.try_lock() {
            Ok(p) => p,
            Err(_) => {
                crate::warn!("Pipeline busy (likely hotkey recording), skipping wake word event");
                return;
            }
        };

        match pipeline.stop_and_get_buffer(audio_thread.as_ref()) {
            Ok(Some(buffer)) => buffer,
            Ok(None) => {
                crate::error!("Pipeline had no buffer to hand off");
                return;
            }
            Err(e) => {
                crate::error!("Failed to stop pipeline: {:?}", e);
                return;
            }
        }
    };

    // 2. Clear the buffer to prevent old audio bleeding into new recording
    if let Ok(mut guard) = shared_buffer.lock() {
        crate::debug!("[event_handler] Clearing buffer before recording ({} samples)", guard.len());
        guard.clear();
    }

    // 3. Start recording with the cleared buffer
    let recording_started = {
        let mut manager = match recording_state.lock() {
            Ok(m) => m,
            Err(_) => {
                crate::error!("Failed to lock recording manager");
                return;
            }
        };

        match manager.start_recording_with_buffer(16000, shared_buffer.clone()) {
            Ok(_) => {
                // Restart audio capture with the shared buffer
                match audio_thread.start_with_device(shared_buffer.clone(), device_name.clone()) {
                    Ok(_) => {
                        crate::info!("Recording started with shared buffer");
                        true
                    }
                    Err(e) => {
                        crate::error!("Failed to restart audio capture: {:?}", e);
                        let _ = manager.abort_recording(RecordingState::Idle);
                        false
                    }
                }
            }
            Err(e) => {
                crate::error!("Failed to start recording: {:?}", e);
                false
            }
        }
    };

    // 4. If recording started, emit event and start detection
    if recording_started {
        // Emit recording_started event
        emit_or_warn!(
            app_handle,
            event_names::RECORDING_STARTED,
            RecordingStartedPayload {
                timestamp: crate::events::current_timestamp(),
            }
        );

        // Start silence/cancel detection (return_to_listening=true)
        // Create transcription callback that calls HotkeyIntegration.spawn_transcription()
        let hotkey_for_transcription = hotkey_integration.clone();
        let transcription_callback: Box<dyn Fn(String) + Send + 'static> = Box::new(move |file_path: String| {
            if let Ok(integration) = hotkey_for_transcription.lock() {
                crate::info!("[event_handler] Spawning transcription via HotkeyIntegration");
                integration.spawn_transcription(file_path);
            } else {
                crate::error!("[event_handler] Failed to lock HotkeyIntegration for transcription");
            }
        });

        if let Ok(mut det) = recording_detectors.lock() {
            if let Err(e) = det.start_monitoring(
                shared_buffer,
                recording_state.clone(),
                audio_thread.clone(),
                emitter.clone(),
                true, // Auto-restart listening after wake word recording
                Some(listening_pipeline.clone()), // Pass pipeline for restart
                Some(transcription_callback), // Transcription callback
            ) {
                crate::warn!("Failed to start recording detectors: {}", e);
            } else {
                crate::info!("Recording detectors started");
            }
        }
    }
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
            let _ = app_handle.emit("audio-level", level);
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

#[cfg(test)]
mod tests;

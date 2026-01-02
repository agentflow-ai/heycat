//! Push-to-Talk (PTT) handler for HotkeyIntegration.
//!
//! Handles the PTT recording mode where holding the hotkey records
//! and releasing it stops recording.

use crate::commands::logic::{start_recording_impl, stop_recording_impl};
use crate::events::{
    current_timestamp, RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload,
    RecordingStoppedPayload,
};
#[cfg(target_os = "macos")]
use crate::keyboard_capture::cgeventtap::set_consume_escape;
use crate::model::{check_model_exists_for_type, ModelType};
use crate::recording::{RecordingManager, RecordingState};
use std::sync::Mutex;

use super::HotkeyIntegration;

impl<R, T, C> HotkeyIntegration<R, T, C>
where
    R: RecordingEventEmitter,
    T: crate::events::TranscriptionEventEmitter + 'static,
    C: crate::events::CommandEventEmitter + 'static,
{
    /// Handle hotkey press event for push-to-talk mode
    ///
    /// In PTT mode, pressing the hotkey starts recording immediately (no debounce).
    /// This is called when the hotkey key is pressed down.
    ///
    /// Returns true if recording was started, false otherwise.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn handle_hotkey_press(&mut self, state: &Mutex<RecordingManager>) -> bool {
        // PTT mode skips debounce for immediate response
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                crate::error!("Failed to acquire lock: {}", e);
                self.recording_emitter
                    .emit_recording_error(RecordingErrorPayload {
                        message: "Internal error: state lock poisoned".to_string(),
                    });
                return false;
            }
        };

        crate::debug!("PTT press received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Idle => {
                crate::info!("PTT: Starting recording on key press...");

                // Check model availability
                let model_available =
                    check_model_exists_for_type(ModelType::ParakeetTDT).unwrap_or_else(|e| {
                        crate::warn!("Failed to check model availability: {}", e);
                        false
                    });

                let device_name = self.get_selected_audio_device();
                match start_recording_impl(
                    state,
                    self.audio_thread.as_deref(),
                    model_available,
                    device_name,
                ) {
                    Ok(()) => {
                        self.recording_emitter
                            .emit_recording_started(RecordingStartedPayload {
                                timestamp: current_timestamp(),
                            });
                        crate::info!("PTT: Recording started");

                        // Register Escape key listener for emergency cancel
                        self.register_escape_listener();

                        // Enable Escape key consumption
                        #[cfg(target_os = "macos")]
                        set_consume_escape(true);

                        // Note: PTT mode does NOT start silence detection
                        // Recording stops on key release, not on silence

                        true
                    }
                    Err(e) => {
                        crate::error!("PTT: Failed to start recording: {}", e);
                        self.recording_emitter
                            .emit_recording_error(RecordingErrorPayload { message: e });
                        false
                    }
                }
            }
            RecordingState::Recording => {
                // Already recording - ignore (user might have double-pressed)
                crate::debug!("PTT press ignored - already recording");
                false
            }
            RecordingState::Processing => {
                // Busy - ignore
                crate::debug!("PTT press ignored - processing");
                false
            }
        }
    }

    /// Handle hotkey release event for push-to-talk mode
    ///
    /// In PTT mode, releasing the hotkey stops recording immediately.
    /// This is called when the hotkey key is released.
    ///
    /// Returns true if recording was stopped, false otherwise.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn handle_hotkey_release(&mut self, state: &Mutex<RecordingManager>) -> bool {
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                crate::error!("Failed to acquire lock: {}", e);
                self.recording_emitter
                    .emit_recording_error(RecordingErrorPayload {
                        message: "Internal error: state lock poisoned".to_string(),
                    });
                return false;
            }
        };

        crate::debug!("PTT release received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Recording => {
                crate::info!("PTT: Stopping recording on key release...");

                // Unregister Escape key listener
                self.unregister_escape_listener();

                // Disable Escape key consumption
                #[cfg(target_os = "macos")]
                set_consume_escape(false);

                // Stop recording and process
                match stop_recording_impl(
                    state,
                    self.audio_thread.as_deref(),
                    false,
                    self.recordings_dir.clone(),
                ) {
                    Ok(metadata) => {
                        crate::info!(
                            "PTT: Recording stopped: {} samples, {:.2}s duration",
                            metadata.sample_count,
                            metadata.duration_secs
                        );

                        // Store recording metadata in Turso using storage abstraction
                        if let Some(ref app_handle) = self.app_handle {
                            if !metadata.file_path.is_empty() {
                                crate::storage::store_recording(app_handle, &metadata, "PTT");
                            }
                        }

                        let file_path_for_transcription = metadata.file_path.clone();
                        self.recording_emitter
                            .emit_recording_stopped(RecordingStoppedPayload { metadata });
                        crate::debug!("PTT: Emitted recording_stopped event");

                        // Auto-transcribe
                        self.spawn_transcription(file_path_for_transcription);

                        true
                    }
                    Err(e) => {
                        crate::error!("PTT: Failed to stop recording: {}", e);
                        self.recording_emitter
                            .emit_recording_error(RecordingErrorPayload { message: e });
                        false
                    }
                }
            }
            RecordingState::Idle => {
                // Not recording - ignore (user might have released after cancel)
                crate::debug!("PTT release ignored - not recording");
                false
            }
            RecordingState::Processing => {
                // Busy - ignore
                crate::debug!("PTT release ignored - processing");
                false
            }
        }
    }
}

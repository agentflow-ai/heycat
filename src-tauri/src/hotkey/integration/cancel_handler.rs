//! Cancel recording handler for HotkeyIntegration.
//!
//! Handles the cancellation of recording via double-tap Escape key.

use crate::events::{
    current_timestamp, RecordingCancelledPayload, RecordingErrorPayload, RecordingEventEmitter,
};
#[cfg(target_os = "macos")]
use crate::keyboard_capture::cgeventtap::set_consume_escape;
use crate::recording::{RecordingManager, RecordingState};
use std::sync::Mutex;

use super::HotkeyIntegration;

impl<R, T, C> HotkeyIntegration<R, T, C>
where
    R: RecordingEventEmitter,
    T: crate::events::TranscriptionEventEmitter + 'static,
    C: crate::events::CommandEventEmitter + 'static,
{
    /// Cancel recording without transcription
    ///
    /// This method is called when the user double-taps Escape during recording.
    /// It stops the recording immediately, discards the audio buffer (no WAV file
    /// created, no transcription triggered), and transitions directly to Idle state.
    ///
    /// # Arguments
    /// * `state` - The recording state mutex
    /// * `reason` - The reason for cancellation (e.g., "double-tap-escape")
    ///
    /// # Returns
    /// * `true` if cancellation was successful
    /// * `false` if not in recording state or an error occurred
    pub fn cancel_recording(&mut self, state: &Mutex<RecordingManager>, reason: &str) -> bool {
        // Check current state - can only cancel from Recording state
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                crate::error!("Failed to acquire lock for cancel: {}", e);
                self.recording_emitter
                    .emit_recording_error(RecordingErrorPayload {
                        message: "Internal error: state lock poisoned".to_string(),
                    });
                return false;
            }
        };

        if current_state != RecordingState::Recording {
            crate::debug!(
                "Cancel ignored - not in recording state (current: {:?})",
                current_state
            );
            return false;
        }

        crate::info!("Cancelling recording (reason: {})", reason);

        // 1. Unregister Escape key listener first
        self.unregister_escape_listener();

        // 2. Disable Escape key consumption since recording is being cancelled
        #[cfg(target_os = "macos")]
        set_consume_escape(false);

        // 3. Stop silence detection if active
        self.stop_silence_detection();

        // 4. Stop audio capture (discard result - we don't want the audio)
        if let Some(ref audio_thread) = self.audio_thread {
            // Stop the audio thread to halt capture
            if let Err(e) = audio_thread.stop() {
                crate::warn!("Failed to stop audio thread during cancel: {:?}", e);
                // Continue anyway - the buffer will be discarded
            }
        }

        // 5. Abort recording - this clears the buffer and transitions directly to Idle
        //    (bypassing Processing state, so no transcription will be triggered)
        let abort_result = match state.lock() {
            Ok(mut guard) => guard.abort_recording(RecordingState::Idle),
            Err(e) => {
                crate::error!("Failed to acquire lock for abort: {}", e);
                self.recording_emitter
                    .emit_recording_error(RecordingErrorPayload {
                        message: "Internal error: state lock poisoned".to_string(),
                    });
                return false;
            }
        };

        match abort_result {
            Ok(()) => {
                // 6. Emit recording_cancelled event
                self.recording_emitter
                    .emit_recording_cancelled(RecordingCancelledPayload {
                        reason: reason.to_string(),
                        timestamp: current_timestamp(),
                    });

                crate::info!("Recording cancelled successfully");
                true
            }
            Err(e) => {
                crate::error!("Failed to abort recording: {}", e);
                self.recording_emitter
                    .emit_recording_error(RecordingErrorPayload {
                        message: format!("Failed to cancel recording: {}", e),
                    });
                false
            }
        }
    }
}

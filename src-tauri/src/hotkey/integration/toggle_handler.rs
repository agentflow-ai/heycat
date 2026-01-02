//! Toggle mode handler for HotkeyIntegration.
//!
//! Handles the toggle recording mode where pressing the hotkey toggles between
//! recording and idle states.

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
use std::time::Instant;

use super::HotkeyIntegration;

impl<R, T, C> HotkeyIntegration<R, T, C>
where
    R: RecordingEventEmitter,
    T: crate::events::TranscriptionEventEmitter + 'static,
    C: crate::events::CommandEventEmitter + 'static,
{
    /// Handle hotkey toggle - debounces rapid presses
    ///
    /// Toggles recording state (Idle → Recording → Idle) and emits events.
    /// Delegates to unified command implementations for start/stop logic.
    ///
    /// Returns true if the toggle was accepted, false if debounced or busy
    ///
    /// Coverage exclusion: Error paths (lock poisoning, command failures) cannot
    /// be triggered without mocking std::sync primitives. The happy path is tested
    /// via integration_test.rs with mock emitters.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn handle_toggle(&mut self, state: &Mutex<RecordingManager>) -> bool {
        let now = Instant::now();

        // Check debounce
        if let Some(last) = self.last_toggle_time {
            if now.duration_since(last) < self.debounce_duration {
                crate::trace!("Toggle debounced");
                return false;
            }
        }

        self.last_toggle_time = Some(now);

        // Check current state to decide action
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

        crate::debug!("Toggle received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Idle => self.start_recording_toggle(state),
            RecordingState::Recording => self.stop_recording_toggle(state),
            RecordingState::Processing => {
                // In Processing state - ignore toggle (busy)
                crate::debug!("Toggle ignored - already processing");
                false
            }
        }
    }

    /// Start recording in toggle mode
    fn start_recording_toggle(&mut self, state: &Mutex<RecordingManager>) -> bool {
        crate::info!("Starting recording from Idle state...");

        // Check model availability (TDT for batch transcription)
        let model_available = check_model_exists_for_type(ModelType::ParakeetTDT).unwrap_or_else(|e| {
            crate::warn!("Failed to check model availability: {}", e);
            false
        });

        // Read selected device from persistent settings store
        let device_name = self.get_selected_audio_device();
        match start_recording_impl(state, self.audio_thread.as_deref(), model_available, device_name)
        {
            Ok(()) => {
                self.recording_emitter
                    .emit_recording_started(RecordingStartedPayload {
                        timestamp: current_timestamp(),
                    });
                crate::info!("Recording started, emitted recording_started event");

                // Register Escape key listener for cancel functionality
                self.register_escape_listener();

                // Enable Escape key consumption to prevent propagation to other apps
                #[cfg(target_os = "macos")]
                set_consume_escape(true);

                // Start silence detection if enabled and configured
                self.start_silence_detection(state);

                true
            }
            Err(e) => {
                crate::error!("Failed to start recording: {}", e);
                self.recording_emitter
                    .emit_recording_error(RecordingErrorPayload { message: e });
                false
            }
        }
    }

    /// Stop recording in toggle mode
    fn stop_recording_toggle(&mut self, state: &Mutex<RecordingManager>) -> bool {
        crate::info!("Stopping recording (manual stop via hotkey)...");

        // Unregister Escape key listener first
        self.unregister_escape_listener();

        // Disable Escape key consumption since recording is stopping
        #[cfg(target_os = "macos")]
        set_consume_escape(false);

        // Stop silence detection first to prevent it from interfering
        // Manual stop takes precedence over auto-stop
        self.stop_silence_detection();

        // Use unified command implementation (always return to Idle)
        match stop_recording_impl(
            state,
            self.audio_thread.as_deref(),
            false,
            self.recordings_dir.clone(),
        ) {
            Ok(metadata) => {
                crate::info!(
                    "Recording stopped: {} samples, {:.2}s duration",
                    metadata.sample_count,
                    metadata.duration_secs
                );

                // Store recording metadata in Turso using storage abstraction
                if let Some(ref app_handle) = self.app_handle {
                    if !metadata.file_path.is_empty() {
                        crate::storage::store_recording(app_handle, &metadata, "hotkey");
                    }
                }

                // Clone file_path before metadata is moved
                let file_path_for_transcription = metadata.file_path.clone();
                self.recording_emitter
                    .emit_recording_stopped(RecordingStoppedPayload { metadata });
                crate::debug!("Emitted recording_stopped event");

                // Auto-transcribe if transcription manager is configured
                self.spawn_transcription(file_path_for_transcription);

                true
            }
            Err(e) => {
                crate::error!("Failed to stop recording: {}", e);
                self.recording_emitter
                    .emit_recording_error(RecordingErrorPayload { message: e });
                false
            }
        }
    }
}

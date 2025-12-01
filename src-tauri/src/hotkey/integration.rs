// Hotkey-to-recording integration module
// Connects global hotkey to recording state with debouncing
// Uses unified command implementations for start/stop logic

use crate::audio::AudioThreadHandle;
use crate::commands::logic::{start_recording_impl, stop_recording_impl};
use crate::events::{
    current_timestamp, RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload,
    RecordingStoppedPayload,
};
use crate::recording::{RecordingManager, RecordingState};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Debounce duration for hotkey presses (200ms)
pub const DEBOUNCE_DURATION_MS: u64 = 200;

/// Handles hotkey toggle with debouncing and event emission
pub struct HotkeyIntegration<E: RecordingEventEmitter> {
    last_toggle_time: Option<Instant>,
    debounce_duration: Duration,
    emitter: E,
    /// Optional audio thread handle - when present, starts/stops capture on toggle
    audio_thread: Option<Arc<AudioThreadHandle>>,
}

impl<E: RecordingEventEmitter> HotkeyIntegration<E> {
    /// Create a new HotkeyIntegration with default debounce duration
    pub fn new(emitter: E) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(DEBOUNCE_DURATION_MS),
            emitter,
            audio_thread: None,
        }
    }

    /// Add an audio thread handle (builder pattern)
    pub fn with_audio_thread(mut self, handle: Arc<AudioThreadHandle>) -> Self {
        self.audio_thread = Some(handle);
        self
    }

    /// Create with custom debounce duration (for testing)
    #[cfg(test)]
    pub fn with_debounce(emitter: E, debounce_ms: u64) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(debounce_ms),
            emitter,
            audio_thread: None,
        }
    }

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
                eprintln!("[hotkey] Toggle debounced");
                return false;
            }
        }

        self.last_toggle_time = Some(now);

        // Check current state to decide action
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                eprintln!("[hotkey] Failed to acquire lock: {}", e);
                self.emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        eprintln!(
            "[hotkey] Toggle received, current state: {:?}",
            current_state
        );

        match current_state {
            RecordingState::Idle => {
                eprintln!("[hotkey] Starting recording...");
                // Use unified command implementation
                match start_recording_impl(state, self.audio_thread.as_deref()) {
                    Ok(()) => {
                        self.emitter
                            .emit_recording_started(RecordingStartedPayload {
                                timestamp: current_timestamp(),
                            });
                        eprintln!("[hotkey] Recording started, emitted recording_started event");
                        true
                    }
                    Err(e) => {
                        eprintln!("[hotkey] Failed to start recording: {}", e);
                        self.emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Recording => {
                eprintln!("[hotkey] Stopping recording...");
                // Use unified command implementation
                match stop_recording_impl(state, self.audio_thread.as_deref()) {
                    Ok(metadata) => {
                        eprintln!(
                            "[hotkey] Recording stopped: {} samples, {:.2}s duration",
                            metadata.sample_count, metadata.duration_secs
                        );
                        self.emitter
                            .emit_recording_stopped(RecordingStoppedPayload { metadata });
                        eprintln!("[hotkey] Emitted recording_stopped event");
                        true
                    }
                    Err(e) => {
                        eprintln!("[hotkey] Failed to stop recording: {}", e);
                        self.emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Processing => {
                // In Processing state - ignore toggle (busy)
                eprintln!("[hotkey] Toggle ignored - already processing");
                false
            }
        }
    }

    /// Check if currently in debounce window (for testing)
    #[cfg(test)]
    pub fn is_debouncing(&self) -> bool {
        if let Some(last) = self.last_toggle_time {
            Instant::now().duration_since(last) < self.debounce_duration
        } else {
            false
        }
    }
}

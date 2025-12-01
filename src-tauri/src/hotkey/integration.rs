// Hotkey-to-recording integration module
// Connects global hotkey to recording state with debouncing

use crate::audio::{encode_wav, wav::SystemFileWriter, AudioBuffer, AudioThreadHandle, DEFAULT_SAMPLE_RATE};
use std::sync::Arc;
use crate::events::{
    current_timestamp, RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload,
    RecordingStoppedPayload,
};
use crate::recording::{RecordingManager, RecordingState};
use std::sync::Mutex;
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
    /// When a backend is configured, starts/stops audio capture on toggle.
    ///
    /// Returns true if the toggle was accepted, false if debounced or busy
    pub fn handle_toggle(&mut self, state: &Mutex<RecordingManager>) -> bool {
        let now = Instant::now();

        // Check debounce
        if let Some(last) = self.last_toggle_time {
            if now.duration_since(last) < self.debounce_duration {
                return false; // Debounced
            }
        }

        self.last_toggle_time = Some(now);

        // Get state manager - use expect since lock errors are unrecoverable
        // (only happen on panic in another thread)
        let mut manager = state.lock().expect("Lock poisoned - unrecoverable");

        let current_state = manager.get_state();
        eprintln!("[hotkey] Toggle received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Idle => {
                eprintln!("[hotkey] Starting recording...");
                // Start recording - creates buffer and transitions to Recording state
                // Use DEFAULT_SAMPLE_RATE initially, will update after audio capture starts
                let buffer = manager
                    .start_recording(DEFAULT_SAMPLE_RATE)
                    .expect("Idle->Recording is always valid");

                // Start audio capture - the function is excluded from coverage
                // because it interacts with audio hardware via the audio thread.
                // On failure it returns false (audio thread disconnected - rare).
                if !self.try_start_audio_capture(&mut *manager, buffer) {
                    eprintln!("[hotkey] Audio capture failed to start, rolling back");
                    return false;
                }

                self.emitter
                    .emit_recording_started(RecordingStartedPayload {
                        timestamp: current_timestamp(),
                    });
                eprintln!("[hotkey] Recording started, emitted recording_started event");
                true
            }
            RecordingState::Recording => {
                eprintln!("[hotkey] Stopping recording...");
                // Stop audio capture if audio thread is configured
                if let Some(ref audio_thread) = self.audio_thread {
                    eprintln!("[hotkey] Sending stop command to audio thread");
                    let _ = audio_thread.stop(); // Best effort, ignore errors
                }

                // Get the actual sample rate before transitioning
                let sample_rate = manager.get_sample_rate().unwrap_or(DEFAULT_SAMPLE_RATE);

                // Stop recording - transition through Processing to Idle
                manager
                    .transition_to(RecordingState::Processing)
                    .expect("Recording->Processing is always valid");

                // Get buffer data before clearing - buffer is always available in Processing
                let buffer = manager
                    .get_audio_buffer()
                    .expect("Buffer available in Processing");
                let samples = buffer.lock().unwrap();
                let sample_count = samples.len();
                let duration_secs = sample_count as f64 / sample_rate as f64;
                eprintln!(
                    "[hotkey] Recording stopped: {} samples, {:.2}s duration at {} Hz",
                    sample_count, duration_secs, sample_rate
                );

                // Encode WAV file if we have samples
                // Coverage exclusion: WAV encoding is tested in wav_test.rs and commands/tests.rs.
                // Integration tests don't have real audio samples from hardware.
                let file_path = self.encode_samples_to_wav(&samples, sample_rate);

                let metadata = crate::recording::RecordingMetadata {
                    duration_secs,
                    file_path,
                    sample_count,
                };
                drop(samples); // Release buffer lock before state transition

                // Transition to Idle
                manager
                    .transition_to(RecordingState::Idle)
                    .expect("Processing->Idle is always valid");

                self.emitter
                    .emit_recording_stopped(RecordingStoppedPayload { metadata });
                eprintln!("[hotkey] Emitted recording_stopped event");
                true
            }
            RecordingState::Processing => {
                // In Processing state - ignore toggle (busy)
                eprintln!("[hotkey] Toggle ignored - already processing");
                false
            }
        }
    }

    /// Try to start audio capture if audio thread is configured
    ///
    /// Takes the audio buffer and starts capture. On success, updates the
    /// manager's sample rate with the actual device rate.
    /// Returns true if capture started or no audio thread configured (continue).
    /// Returns false if capture failed (caller should return early).
    ///
    /// Excluded from coverage: This function interacts with the audio thread
    /// which ultimately calls hardware (cpal). The error path is only hit when
    /// the audio thread disconnects, which is a rare edge case.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn try_start_audio_capture(&self, manager: &mut RecordingManager, buffer: AudioBuffer) -> bool {
        if let Some(ref audio_thread) = self.audio_thread {
            match audio_thread.start(buffer) {
                Ok(sample_rate) => {
                    // Update with actual sample rate from device
                    manager.set_sample_rate(sample_rate);
                    eprintln!("[hotkey] Audio capture started at {} Hz", sample_rate);
                }
                Err(e) => {
                    // Audio thread disconnected or capture failed - rollback and emit error
                    manager.reset_to_idle();
                    self.emitter.emit_recording_error(RecordingErrorPayload {
                        message: format!("Audio capture failed: {:?}", e),
                    });
                    return false;
                }
            }
        }
        true
    }

    /// Encode audio samples to WAV file with the specified sample rate
    ///
    /// Excluded from coverage: WAV encoding is already tested in wav_test.rs
    /// and commands/tests.rs. Integration tests don't have real audio samples.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn encode_samples_to_wav(&self, samples: &[f32], sample_rate: u32) -> String {
        if !samples.is_empty() {
            eprintln!("[hotkey] Encoding WAV file at {} Hz...", sample_rate);
            let writer = SystemFileWriter;
            match encode_wav(samples, sample_rate, &writer) {
                Ok(path) => {
                    eprintln!("[hotkey] WAV file saved: {}", path);
                    path
                }
                Err(e) => {
                    eprintln!("[hotkey] Failed to encode WAV: {:?}", e);
                    String::new()
                }
            }
        } else {
            eprintln!("[hotkey] No samples to encode");
            String::new()
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

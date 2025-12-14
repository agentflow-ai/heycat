// Recording state management for Tauri application

use crate::audio::{AudioBuffer, StopReason, TARGET_SAMPLE_RATE};
use crate::error;
use serde::Serialize;

/// Recording state enum representing the current state of the recording process
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RecordingState {
    /// Not recording, ready to start
    Idle,
    /// Always-on listening mode, detecting wake word
    Listening,
    /// Actively recording audio
    Recording,
    /// Recording stopped, processing audio (encoding, saving)
    Processing,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Errors that can occur during state transitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingStateError {
    /// Invalid state transition attempted
    InvalidTransition {
        from: RecordingState,
        to: RecordingState,
    },
    /// Audio buffer not available
    NoAudioBuffer,
}

impl std::fmt::Display for RecordingStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordingStateError::InvalidTransition { from, to } => {
                write!(f, "Invalid state transition from {:?} to {:?}", from, to)
            }
            RecordingStateError::NoAudioBuffer => {
                write!(f, "Audio buffer not available")
            }
        }
    }
}

impl std::error::Error for RecordingStateError {}

/// Audio data returned for transcription pipeline integration
#[derive(Debug, Clone, Serialize)]
pub struct AudioData {
    /// Raw audio samples as f32 values normalized to [-1.0, 1.0]
    pub samples: Vec<f32>,
    /// Sample rate in Hz (e.g., 44100)
    pub sample_rate: u32,
    /// Duration of the audio in seconds
    pub duration_secs: f64,
}

/// Metadata returned after a successful recording
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecordingMetadata {
    /// Duration of the recording in seconds
    pub duration_secs: f64,
    /// Path to the saved WAV file
    pub file_path: String,
    /// Number of audio samples recorded
    pub sample_count: usize,
    /// Why recording stopped (None = user initiated, Some = auto-stopped)
    pub stop_reason: Option<StopReason>,
}

/// Retained recording data from the last completed recording
#[derive(Debug, Clone)]
struct LastRecording {
    samples: Vec<f32>,
    sample_rate: u32,
}

/// Active recording data (during Recording state)
#[derive(Debug, Clone)]
struct ActiveRecording {
    /// Actual sample rate from the audio device
    sample_rate: u32,
}

/// Manager for recording state with thread-safe access
/// Designed to be wrapped in Mutex and managed by Tauri state
pub struct RecordingManager {
    state: RecordingState,
    audio_buffer: Option<AudioBuffer>,
    /// Active recording info (sample rate from device)
    active_recording: Option<ActiveRecording>,
    /// Retained audio data from the last recording for transcription
    last_recording: Option<LastRecording>,
}

impl RecordingManager {
    /// Create a new RecordingManager in Idle state
    pub fn new() -> Self {
        Self {
            state: RecordingState::Idle,
            audio_buffer: None,
            active_recording: None,
            last_recording: None,
        }
    }

    /// Get the current recording state
    pub fn get_state(&self) -> RecordingState {
        self.state
    }

    /// Start recording with the given sample rate
    ///
    /// Transitions from Idle or Listening to Recording state and creates the audio buffer.
    /// Returns the audio buffer for use with audio capture.
    /// The sample rate is stored for use when the recording completes.
    ///
    /// # Errors
    /// Returns error if not in Idle or Listening state
    #[must_use = "this returns a Result that should be handled"]
    pub fn start_recording(&mut self, sample_rate: u32) -> Result<AudioBuffer, RecordingStateError> {
        if self.state != RecordingState::Idle && self.state != RecordingState::Listening {
            return Err(RecordingStateError::InvalidTransition {
                from: self.state,
                to: RecordingState::Recording,
            });
        }

        let buffer = AudioBuffer::new();
        self.audio_buffer = Some(buffer.clone());
        self.active_recording = Some(ActiveRecording { sample_rate });
        self.state = RecordingState::Recording;
        Ok(buffer)
    }

    /// Update the sample rate for the current recording
    ///
    /// Call this after audio capture starts to set the actual device sample rate.
    /// Only works in Recording state.
    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        if let Some(ref mut active) = self.active_recording {
            active.sample_rate = sample_rate;
        }
    }

    /// Transition to a new state with validation
    ///
    /// Valid transitions:
    /// - Idle -> Listening (start listening mode)
    /// - Listening -> Idle (stop listening mode)
    /// - Recording -> Processing (stops recording, keeps buffer)
    /// - Processing -> Idle (clears buffer, retains samples for transcription)
    /// - Processing -> Listening (clears buffer, returns to listening mode)
    ///
    /// Note: Use `start_recording(sample_rate)` for Idle/Listening -> Recording transition
    ///
    /// Returns error for invalid transitions
    #[must_use = "this returns a Result that should be handled"]
    pub fn transition_to(&mut self, new_state: RecordingState) -> Result<(), RecordingStateError> {
        let valid = matches!(
            (self.state, new_state),
            (RecordingState::Idle, RecordingState::Listening)
                | (RecordingState::Listening, RecordingState::Idle)
                | (RecordingState::Recording, RecordingState::Processing)
                | (RecordingState::Processing, RecordingState::Idle)
                | (RecordingState::Processing, RecordingState::Listening)
        );

        if !valid {
            return Err(RecordingStateError::InvalidTransition {
                from: self.state,
                to: new_state,
            });
        }

        // Handle buffer lifecycle during transitions
        if self.state == RecordingState::Processing
            && (new_state == RecordingState::Idle || new_state == RecordingState::Listening)
        {
            self.retain_recording_buffer();
            self.audio_buffer = None;
            self.active_recording = None;
        }

        self.state = new_state;
        Ok(())
    }

    /// Get the sample rate of the current recording
    ///
    /// Returns the sample rate if currently recording or processing, None otherwise
    pub fn get_sample_rate(&self) -> Option<u32> {
        self.active_recording.as_ref().map(|r| r.sample_rate)
    }

    /// Retain audio buffer samples for transcription before clearing.
    ///
    /// Called during Processing -> Idle transition. The audio_buffer should always
    /// be Some in Processing state through normal API usage; the None case is
    /// defensive and unreachable through the public API.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn retain_recording_buffer(&mut self) {
        if let Some(ref buffer) = self.audio_buffer {
            match buffer.lock() {
                Ok(samples) => {
                    let sample_rate = self
                        .active_recording
                        .as_ref()
                        .map(|r| r.sample_rate)
                        .unwrap_or(TARGET_SAMPLE_RATE);
                    self.last_recording = Some(LastRecording {
                        samples: samples.clone(),
                        sample_rate,
                    });
                }
                Err(e) => {
                    error!("Failed to retain buffer (lock poisoned): {}", e);
                }
            }
        }
    }

    /// Get a reference to the audio buffer (available during Recording and Processing states)
    pub fn get_audio_buffer(&self) -> Result<AudioBuffer, RecordingStateError> {
        self.audio_buffer
            .clone()
            .ok_or(RecordingStateError::NoAudioBuffer)
    }

    /// Get the last recording's audio data for transcription
    ///
    /// Returns the audio data from the most recent completed recording.
    /// The buffer is retained in memory after the recording is saved.
    pub fn get_last_recording_buffer(&self) -> Result<AudioData, RecordingStateError> {
        match &self.last_recording {
            Some(recording) => {
                let sample_count = recording.samples.len();
                // Guard against division by zero (should never happen, but defensive)
                let duration_secs = if recording.sample_rate > 0 {
                    sample_count as f64 / recording.sample_rate as f64
                } else {
                    0.0
                };
                Ok(AudioData {
                    samples: recording.samples.clone(),
                    sample_rate: recording.sample_rate,
                    duration_secs,
                })
            }
            None => Err(RecordingStateError::NoAudioBuffer),
        }
    }

    /// Clear the retained last recording buffer
    ///
    /// Call this to free memory when the transcription is complete
    pub fn clear_last_recording(&mut self) {
        self.last_recording = None;
    }

    /// Force reset to Idle state, clearing any audio buffer
    ///
    /// Use for error recovery when normal state transitions aren't possible
    /// (e.g., capture failure during start_recording)
    pub fn reset_to_idle(&mut self) {
        self.state = RecordingState::Idle;
        self.audio_buffer = None;
        self.active_recording = None;
    }
}

impl Default for RecordingManager {
    fn default() -> Self {
        Self::new()
    }
}

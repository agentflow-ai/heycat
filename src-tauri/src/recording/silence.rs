// Silence detection for automatic recording stop
// Uses VAD (Voice Activity Detection) to identify end of speech

use super::vad::{create_vad, VadConfig};
use crate::audio_constants::{
    DEFAULT_SAMPLE_RATE, NO_SPEECH_TIMEOUT_MS, PAUSE_TOLERANCE_MS, SILENCE_DURATION_MS,
    SILENCE_MIN_SPEECH_FRAMES, VAD_CHUNK_SIZE_16KHZ, VAD_THRESHOLD_SILENCE,
};
use std::time::Instant;
use voice_activity_detector::VoiceActivityDetector;

/// Reason why recording was automatically stopped due to silence detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SilenceStopReason {
    /// Recording stopped because user finished speaking (silence after speech)
    SilenceAfterSpeech,
    /// Recording stopped because no speech was detected (false activation)
    NoSpeechTimeout,
}

/// Configuration for silence detection
#[derive(Debug, Clone)]
pub struct SilenceConfig {
    /// VAD speech probability threshold (0.0 - 1.0, default: 0.5)
    pub vad_speech_threshold: f32,
    /// Duration of silence before stopping recording in milliseconds (default: 2000)
    pub silence_duration_ms: u32,
    /// Duration before canceling if no speech detected in milliseconds (default: 5000)
    pub no_speech_timeout_ms: u32,
    /// Duration of pause that doesn't trigger stop in milliseconds (default: 1000)
    #[allow(dead_code)] // Reserved for future pause detection refinement
    pub pause_tolerance_ms: u32,
    /// Sample rate for VAD processing (default: 16000)
    pub sample_rate: u32,
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            vad_speech_threshold: VAD_THRESHOLD_SILENCE,
            silence_duration_ms: SILENCE_DURATION_MS,
            no_speech_timeout_ms: NO_SPEECH_TIMEOUT_MS,
            pause_tolerance_ms: PAUSE_TOLERANCE_MS,
            sample_rate: DEFAULT_SAMPLE_RATE,
        }
    }
}

/// Result of processing audio samples
#[derive(Debug, Clone, PartialEq)]
pub enum SilenceDetectionResult {
    /// Continue recording, no action needed
    Continue,
    /// Stop recording due to silence
    Stop(SilenceStopReason),
}

/// Silence detector for automatic recording stop
///
/// Processes audio samples and determines when to stop recording based on:
/// - Silence after speech (user finished talking)
/// - No speech timeout (false activation)
pub struct SilenceDetector {
    config: SilenceConfig,
    /// Whether we've detected any speech since recording started
    has_detected_speech: bool,
    /// When the current silence period started (if currently silent)
    silence_start: Option<Instant>,
    /// When recording started (for no-speech timeout)
    recording_start: Instant,
    /// Voice activity detector for speech detection
    vad: Option<VoiceActivityDetector>,
}

impl SilenceDetector {
    /// Create a new silence detector with default configuration
    pub fn new() -> Self {
        Self::with_config(SilenceConfig::default())
    }

    /// Create a new silence detector with custom configuration
    ///
    /// Uses the unified VadConfig with silence preset for optimal precision.
    pub fn with_config(config: SilenceConfig) -> Self {
        // Initialize VAD using unified factory
        let vad_config = VadConfig {
            speech_threshold: config.vad_speech_threshold,
            sample_rate: config.sample_rate,
            min_speech_frames: SILENCE_MIN_SPEECH_FRAMES,
        };

        let vad = create_vad(&vad_config).ok();

        if vad.is_some() {
            crate::debug!("[silence] VAD initialized (threshold={})", config.vad_speech_threshold);
        } else {
            crate::debug!("[silence] VAD initialization failed, speech detection will be disabled");
        }

        Self {
            config,
            has_detected_speech: false,
            silence_start: None,
            recording_start: Instant::now(),
            vad,
        }
    }

    /// Reset the detector state for a new recording session
    pub fn reset(&mut self) {
        crate::debug!("[silence] Detector reset for new recording session");
        self.has_detected_speech = false;
        self.silence_start = None;
        self.recording_start = Instant::now();

        // Reinitialize VAD for fresh state using unified factory
        let vad_config = VadConfig {
            speech_threshold: self.config.vad_speech_threshold,
            sample_rate: self.config.sample_rate,
            min_speech_frames: SILENCE_MIN_SPEECH_FRAMES,
        };
        self.vad = create_vad(&vad_config).ok();
    }

    /// Get the configuration
    #[allow(dead_code)] // Utility method for introspection
    pub fn config(&self) -> &SilenceConfig {
        &self.config
    }

    /// Check if speech has been detected
    #[allow(dead_code)] // Utility method for status checks
    pub fn has_detected_speech(&self) -> bool {
        self.has_detected_speech
    }

    /// Check if speech is present using VAD
    ///
    /// Processes audio in 512-sample chunks (required by Silero VAD at 16kHz).
    /// Returns true if any chunk has speech probability above threshold.
    fn check_vad(&mut self, samples: &[f32]) -> bool {
        let vad = match &mut self.vad {
            Some(v) => v,
            None => {
                crate::trace!("[silence] VAD not available, assuming no speech");
                return false;
            }
        };

        // Process in VAD_CHUNK_SIZE_16KHZ chunks (required by Silero VAD at 16kHz)
        let chunk_size = VAD_CHUNK_SIZE_16KHZ;
        let mut max_probability: f32 = 0.0;

        for chunk in samples.chunks(chunk_size) {
            if chunk.len() == chunk_size {
                let probability = vad.predict(chunk.to_vec());
                max_probability = max_probability.max(probability);
                if probability >= self.config.vad_speech_threshold {
                    return true; // Speech detected
                }
            }
        }

        crate::trace!("[silence] VAD max_probability={:.3}, threshold={}", max_probability, self.config.vad_speech_threshold);
        false
    }

    /// Process a frame of audio samples and return detection result
    ///
    /// Call this periodically with frames of audio (e.g., 100ms chunks).
    /// Returns whether to continue recording or stop (with reason).
    pub fn process_samples(&mut self, samples: &[f32]) -> SilenceDetectionResult {
        let now = Instant::now();

        // Use VAD to detect speech
        let has_speech = self.check_vad(samples);
        let is_silent = !has_speech;

        if is_silent {
            // Audio is silent (no speech detected by VAD)
            if self.silence_start.is_none() {
                // Start tracking silence period
                crate::debug!("[silence] Silence period started (VAD)");
                self.silence_start = Some(now);
            }

            let silence_duration = self.silence_start.unwrap().elapsed();

            if !self.has_detected_speech {
                // No speech yet - check for no-speech timeout
                let total_elapsed = self.recording_start.elapsed();
                crate::trace!(
                    "[silence] No speech yet, elapsed={:?}, timeout={}ms",
                    total_elapsed,
                    self.config.no_speech_timeout_ms
                );
                if total_elapsed.as_millis() >= self.config.no_speech_timeout_ms as u128 {
                    crate::info!(
                        "[silence] NO_SPEECH_TIMEOUT triggered after {:?}",
                        total_elapsed
                    );
                    return SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout);
                }
            } else {
                // Had speech - check for silence after speech (ignoring brief pauses)
                crate::trace!(
                    "[silence] Silence after speech, duration={:?}, threshold={}ms",
                    silence_duration,
                    self.config.silence_duration_ms
                );
                if silence_duration.as_millis() >= self.config.silence_duration_ms as u128 {
                    crate::info!(
                        "[silence] SILENCE_AFTER_SPEECH triggered after {:?} of silence",
                        silence_duration
                    );
                    return SilenceDetectionResult::Stop(SilenceStopReason::SilenceAfterSpeech);
                }
            }
        } else {
            // Speech detected by VAD
            if !self.has_detected_speech {
                crate::debug!("[silence] First speech detected via VAD!");
            }
            if self.silence_start.is_some() {
                crate::debug!("[silence] Speech resumed after silence");
            }
            self.has_detected_speech = true;
            self.silence_start = None;
        }

        SilenceDetectionResult::Continue
    }
}

impl Default for SilenceDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "silence_test.rs"]
mod tests;

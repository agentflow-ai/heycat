// Silence detection for automatic recording stop
// Uses energy-based (RMS) detection to identify end of speech

use crate::{debug, info, trace};
use std::time::Instant;

/// Reason why recording was automatically stopped due to silence detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SilenceStopReason {
    /// Recording stopped because user finished speaking (silence after speech)
    SilenceAfterSpeech,
    /// Recording stopped because no speech was detected after wake word (false activation)
    NoSpeechTimeout,
}

/// Configuration for silence detection
#[derive(Debug, Clone)]
pub struct SilenceConfig {
    /// RMS threshold below which audio is considered silent (default: 0.01)
    pub silence_threshold: f32,
    /// Duration of silence before stopping recording in milliseconds (default: 2000)
    pub silence_duration_ms: u32,
    /// Duration before canceling if no speech detected after wake word in milliseconds (default: 5000)
    pub no_speech_timeout_ms: u32,
    /// Duration of pause that doesn't trigger stop in milliseconds (default: 1000)
    #[allow(dead_code)] // Reserved for future pause detection refinement
    pub pause_tolerance_ms: u32,
    /// Sample rate for calculating frame durations (default: 16000)
    #[allow(dead_code)] // Reserved for future frame-based timing calculations
    pub sample_rate: u32,
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            silence_threshold: 0.01,
            silence_duration_ms: 2000,
            no_speech_timeout_ms: 5000,
            pause_tolerance_ms: 1000,
            sample_rate: 16000,
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
/// - No speech timeout (false activation after wake word)
pub struct SilenceDetector {
    config: SilenceConfig,
    /// Whether we've detected any speech since recording started
    has_detected_speech: bool,
    /// When the current silence period started (if currently silent)
    silence_start: Option<Instant>,
    /// When recording started (for no-speech timeout)
    recording_start: Instant,
}

impl SilenceDetector {
    /// Create a new silence detector with default configuration
    pub fn new() -> Self {
        Self::with_config(SilenceConfig::default())
    }

    /// Create a new silence detector with custom configuration
    pub fn with_config(config: SilenceConfig) -> Self {
        Self {
            config,
            has_detected_speech: false,
            silence_start: None,
            recording_start: Instant::now(),
        }
    }

    /// Reset the detector state for a new recording session
    pub fn reset(&mut self) {
        debug!("[silence] Detector reset for new recording session");
        self.has_detected_speech = false;
        self.silence_start = None;
        self.recording_start = Instant::now();
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

    /// Calculate RMS (root mean square) energy of audio samples
    ///
    /// RMS is a common measure of audio energy/loudness.
    /// Returns 0.0 for empty input.
    pub fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Process a frame of audio samples and return detection result
    ///
    /// Call this periodically with frames of audio (e.g., 100ms chunks).
    /// Returns whether to continue recording or stop (with reason).
    pub fn process_samples(&mut self, samples: &[f32]) -> SilenceDetectionResult {
        let now = Instant::now();
        let rms = Self::calculate_rms(samples);
        let is_silent = rms < self.config.silence_threshold;

        // Log audio level on every call (trace level for high-frequency)
        trace!(
            "[silence] RMS={:.4}, threshold={:.4}, is_silent={}, samples={}",
            rms,
            self.config.silence_threshold,
            is_silent,
            samples.len()
        );

        if is_silent {
            // Audio is silent
            if self.silence_start.is_none() {
                // Start tracking silence period
                debug!("[silence] Silence period started");
                self.silence_start = Some(now);
            }

            let silence_duration = self.silence_start.unwrap().elapsed();

            if !self.has_detected_speech {
                // No speech yet - check for no-speech timeout
                let total_elapsed = self.recording_start.elapsed();
                trace!(
                    "[silence] No speech yet, elapsed={:?}, timeout={}ms",
                    total_elapsed,
                    self.config.no_speech_timeout_ms
                );
                if total_elapsed.as_millis() >= self.config.no_speech_timeout_ms as u128 {
                    info!(
                        "[silence] NO_SPEECH_TIMEOUT triggered after {:?}",
                        total_elapsed
                    );
                    return SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout);
                }
            } else {
                // Had speech - check for silence after speech (ignoring brief pauses)
                trace!(
                    "[silence] Silence after speech, duration={:?}, threshold={}ms",
                    silence_duration,
                    self.config.silence_duration_ms
                );
                if silence_duration.as_millis() >= self.config.silence_duration_ms as u128 {
                    info!(
                        "[silence] SILENCE_AFTER_SPEECH triggered after {:?} of silence",
                        silence_duration
                    );
                    return SilenceDetectionResult::Stop(SilenceStopReason::SilenceAfterSpeech);
                }
            }
        } else {
            // Audio is not silent (speech detected)
            if !self.has_detected_speech {
                debug!("[silence] First speech detected! RMS={:.4}", rms);
            }
            if self.silence_start.is_some() {
                debug!("[silence] Speech resumed after silence");
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
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_calculate_rms_empty() {
        assert_eq!(SilenceDetector::calculate_rms(&[]), 0.0);
    }

    #[test]
    fn test_calculate_rms_silence() {
        let samples = vec![0.0; 100];
        assert_eq!(SilenceDetector::calculate_rms(&samples), 0.0);
    }

    #[test]
    fn test_calculate_rms_constant() {
        // RMS of constant signal equals the constant
        let samples = vec![0.5; 100];
        let rms = SilenceDetector::calculate_rms(&samples);
        assert!((rms - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_calculate_rms_speech_like() {
        // Simulate speech-like audio with varying amplitudes
        let samples: Vec<f32> = (0..1600)
            .map(|i| (i as f32 * 0.1).sin() * 0.3)
            .collect();
        let rms = SilenceDetector::calculate_rms(&samples);
        // Sin wave RMS = peak / sqrt(2) ≈ 0.3 / 1.414 ≈ 0.212
        assert!(rms > 0.1, "RMS {} should be > 0.1", rms);
        assert!(rms < 0.3, "RMS {} should be < 0.3", rms);
    }

    #[test]
    fn test_silence_config_default() {
        let config = SilenceConfig::default();
        assert_eq!(config.silence_threshold, 0.01);
        assert_eq!(config.silence_duration_ms, 2000);
        assert_eq!(config.no_speech_timeout_ms, 5000);
        assert_eq!(config.pause_tolerance_ms, 1000);
        assert_eq!(config.sample_rate, 16000);
    }

    #[test]
    fn test_silence_detector_new() {
        let detector = SilenceDetector::new();
        assert!(!detector.has_detected_speech());
    }

    #[test]
    fn test_silence_detector_default() {
        let detector = SilenceDetector::default();
        assert!(!detector.has_detected_speech());
    }

    #[test]
    fn test_speech_detection_sets_flag() {
        let mut detector = SilenceDetector::new();
        let speech_samples = vec![0.5; 100];

        assert!(!detector.has_detected_speech());
        let _ = detector.process_samples(&speech_samples);
        assert!(detector.has_detected_speech());
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = SilenceDetector::new();
        let speech_samples = vec![0.5; 100];

        let _ = detector.process_samples(&speech_samples);
        assert!(detector.has_detected_speech());

        detector.reset();
        assert!(!detector.has_detected_speech());
    }

    #[test]
    fn test_continues_during_speech() {
        let mut detector = SilenceDetector::new();
        let speech_samples = vec![0.5; 100];

        let result = detector.process_samples(&speech_samples);
        assert_eq!(result, SilenceDetectionResult::Continue);
    }

    #[test]
    fn test_brief_silence_continues() {
        let mut detector = SilenceDetector::new();
        let speech_samples = vec![0.5; 100];
        let silent_samples = vec![0.001; 100];

        // First some speech
        let _ = detector.process_samples(&speech_samples);

        // Then brief silence - should continue
        let result = detector.process_samples(&silent_samples);
        assert_eq!(result, SilenceDetectionResult::Continue);
    }

    #[test]
    fn test_no_speech_timeout() {
        let config = SilenceConfig {
            no_speech_timeout_ms: 50, // Very short for testing
            ..Default::default()
        };
        let mut detector = SilenceDetector::with_config(config);
        let silent_samples = vec![0.001; 100];

        // Process silence until timeout
        thread::sleep(Duration::from_millis(60));
        let result = detector.process_samples(&silent_samples);

        assert_eq!(result, SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout));
    }

    #[test]
    fn test_silence_after_speech() {
        let config = SilenceConfig {
            silence_duration_ms: 50, // Very short for testing
            ..Default::default()
        };
        let mut detector = SilenceDetector::with_config(config);
        let speech_samples = vec![0.5; 100];
        let silent_samples = vec![0.001; 100];

        // First some speech
        let _ = detector.process_samples(&speech_samples);
        assert!(detector.has_detected_speech());

        // Then silence
        let _ = detector.process_samples(&silent_samples);

        // Wait and check again
        thread::sleep(Duration::from_millis(60));
        let result = detector.process_samples(&silent_samples);

        assert_eq!(result, SilenceDetectionResult::Stop(SilenceStopReason::SilenceAfterSpeech));
    }

    #[test]
    fn test_speech_resets_silence_tracking() {
        let config = SilenceConfig {
            silence_duration_ms: 50,
            ..Default::default()
        };
        let mut detector = SilenceDetector::with_config(config);
        let speech_samples = vec![0.5; 100];
        let silent_samples = vec![0.001; 100];

        // Speech
        let _ = detector.process_samples(&speech_samples);

        // Some silence
        let _ = detector.process_samples(&silent_samples);
        thread::sleep(Duration::from_millis(30));

        // More speech before timeout - should reset
        let _ = detector.process_samples(&speech_samples);

        // More silence - should continue since timer was reset
        let result = detector.process_samples(&silent_samples);
        assert_eq!(result, SilenceDetectionResult::Continue);
    }

    #[test]
    fn test_config_accessor() {
        let config = SilenceConfig {
            silence_threshold: 0.05,
            ..Default::default()
        };
        let detector = SilenceDetector::with_config(config);
        assert_eq!(detector.config().silence_threshold, 0.05);
    }

    #[test]
    fn test_silence_stop_reason_debug() {
        let reason = SilenceStopReason::SilenceAfterSpeech;
        let debug = format!("{:?}", reason);
        assert!(debug.contains("SilenceAfterSpeech"));
    }

    #[test]
    fn test_silence_detection_result_eq() {
        let r1 = SilenceDetectionResult::Continue;
        let r2 = SilenceDetectionResult::Continue;
        assert_eq!(r1, r2);

        let r3 = SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout);
        let r4 = SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout);
        assert_eq!(r3, r4);

        assert_ne!(r1, r3);
    }

    #[test]
    fn test_background_noise_doesnt_trigger_speech() {
        let mut detector = SilenceDetector::new();
        // Low-level background noise (below threshold)
        let noise_samples = vec![0.005; 100];

        let _ = detector.process_samples(&noise_samples);
        assert!(!detector.has_detected_speech());
    }

    #[test]
    fn test_varying_speech_patterns() {
        let mut detector = SilenceDetector::new();

        // Simulate alternating speech and brief pauses
        let speech = vec![0.3; 100];
        let brief_pause = vec![0.005; 100];

        for _ in 0..5 {
            let r1 = detector.process_samples(&speech);
            let r2 = detector.process_samples(&brief_pause);
            assert_eq!(r1, SilenceDetectionResult::Continue);
            assert_eq!(r2, SilenceDetectionResult::Continue);
        }

        assert!(detector.has_detected_speech());
    }
}

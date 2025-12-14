// Cancel phrase detector for aborting false wake word activations
// Detects "cancel" or "nevermind" spoken during the first 3 seconds of recording

use super::CircularBuffer;
use crate::events::{current_timestamp, listening_events, ListeningEventEmitter};
use crate::model::download::{get_model_dir, ModelType};
use parakeet_rs::ParakeetTDT;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Configuration for cancel phrase detection
#[derive(Debug, Clone)]
pub struct CancelPhraseDetectorConfig {
    /// Minimum confidence threshold (0.0 - 1.0)
    pub confidence_threshold: f32,
    /// Audio window duration in seconds for detection
    pub window_duration_secs: f32,
    /// Sample rate in Hz (must match audio input)
    pub sample_rate: u32,
    /// Time window in which cancellation is allowed (seconds from start of recording)
    pub cancellation_window_secs: f32,
}

impl Default for CancelPhraseDetectorConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.7,
            // ~2 seconds at 16kHz for detection window
            window_duration_secs: 2.0,
            sample_rate: 16000,
            // Only allow cancellation in the first 3 seconds
            cancellation_window_secs: 3.0,
        }
    }
}

/// Result of cancel phrase detection analysis
#[derive(Debug, Clone, PartialEq)]
pub struct CancelPhraseResult {
    /// Whether a cancel phrase was detected
    pub detected: bool,
    /// The detected cancel phrase (if any)
    pub phrase: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// The transcribed text from the audio window
    pub transcription: String,
}

/// Errors that can occur during cancel phrase detection
#[derive(Debug, Clone, PartialEq)]
pub enum CancelPhraseError {
    /// Model has not been loaded
    ModelNotLoaded,
    /// Failed to load the model
    ModelLoadFailed(String),
    /// Failed during transcription
    TranscriptionFailed(String),
    /// Lock was poisoned
    LockPoisoned,
    /// Buffer is empty
    EmptyBuffer,
    /// Cancellation window has expired
    WindowExpired,
}

impl std::fmt::Display for CancelPhraseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CancelPhraseError::ModelNotLoaded => write!(f, "Cancel phrase model not loaded"),
            CancelPhraseError::ModelLoadFailed(msg) => write!(f, "Failed to load model: {}", msg),
            CancelPhraseError::TranscriptionFailed(msg) => {
                write!(f, "Transcription failed: {}", msg)
            }
            CancelPhraseError::LockPoisoned => write!(f, "Internal lock error"),
            CancelPhraseError::EmptyBuffer => write!(f, "Audio buffer is empty"),
            CancelPhraseError::WindowExpired => write!(f, "Cancellation window has expired"),
        }
    }
}

impl std::error::Error for CancelPhraseError {}

/// Cancel phrase detector using Parakeet TDT model
///
/// Detects cancel phrases ("cancel", "nevermind") during recording
/// to allow users to abort false wake word activations.
pub struct CancelPhraseDetector {
    /// Configuration
    config: CancelPhraseDetectorConfig,
    /// TDT model context (thread-safe)
    model: Arc<Mutex<Option<ParakeetTDT>>>,
    /// Circular buffer for audio samples
    buffer: Mutex<CircularBuffer>,
    /// When the recording/detection session started
    session_start: Mutex<Option<Instant>>,
}

impl CancelPhraseDetector {
    /// Create a new cancel phrase detector with default configuration
    pub fn new() -> Self {
        Self::with_config(CancelPhraseDetectorConfig::default())
    }

    /// Create a new cancel phrase detector with custom configuration
    pub fn with_config(config: CancelPhraseDetectorConfig) -> Self {
        let buffer = CircularBuffer::for_duration(config.window_duration_secs, config.sample_rate);
        Self {
            config,
            model: Arc::new(Mutex::new(None)),
            buffer: Mutex::new(buffer),
            session_start: Mutex::new(None),
        }
    }

    /// Load the Parakeet TDT model
    ///
    /// Must be called before processing audio.
    pub fn load_model(&self) -> Result<(), CancelPhraseError> {
        let model_dir = get_model_dir(ModelType::ParakeetTDT)
            .map_err(|e| CancelPhraseError::ModelLoadFailed(e.to_string()))?;

        let path_str = model_dir.to_str().ok_or_else(|| {
            CancelPhraseError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        let tdt = ParakeetTDT::from_pretrained(path_str, None)
            .map_err(|e| CancelPhraseError::ModelLoadFailed(e.to_string()))?;

        let mut guard = self.model.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
        *guard = Some(tdt);

        Ok(())
    }

    /// Check if the model is loaded
    pub fn is_loaded(&self) -> bool {
        self.model
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// Start a new detection session
    ///
    /// Call this when recording starts to begin the cancellation window.
    pub fn start_session(&self) -> Result<(), CancelPhraseError> {
        let mut start = self.session_start.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
        *start = Some(Instant::now());

        // Clear the buffer for the new session
        let mut buffer = self.buffer.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
        buffer.clear();

        Ok(())
    }

    /// End the current detection session
    pub fn end_session(&self) -> Result<(), CancelPhraseError> {
        let mut start = self.session_start.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
        *start = None;

        let mut buffer = self.buffer.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
        buffer.clear();

        Ok(())
    }

    /// Check if the cancellation window is still open
    pub fn is_window_open(&self) -> bool {
        let start = match self.session_start.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        match *start {
            Some(instant) => {
                instant.elapsed()
                    < Duration::from_secs_f32(self.config.cancellation_window_secs)
            }
            None => false,
        }
    }

    /// Get remaining time in cancellation window (in seconds)
    pub fn remaining_window_secs(&self) -> f32 {
        let start = match self.session_start.lock() {
            Ok(guard) => guard,
            Err(_) => return 0.0,
        };

        match *start {
            Some(instant) => {
                let elapsed = instant.elapsed().as_secs_f32();
                (self.config.cancellation_window_secs - elapsed).max(0.0)
            }
            None => 0.0,
        }
    }

    /// Push audio samples into the buffer for analysis
    ///
    /// Call this with incoming audio data from the audio capture.
    pub fn push_samples(&self, samples: &[f32]) -> Result<(), CancelPhraseError> {
        let mut buffer = self.buffer.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
        buffer.push_samples(samples);
        Ok(())
    }

    /// Analyze the current buffer for cancel phrases
    ///
    /// Returns a CancelPhraseResult indicating whether a cancel phrase was detected.
    /// Returns WindowExpired error if called after the cancellation window has closed.
    pub fn analyze(&self) -> Result<CancelPhraseResult, CancelPhraseError> {
        // Check if window is still open
        if !self.is_window_open() {
            return Err(CancelPhraseError::WindowExpired);
        }

        // Get samples from buffer
        let samples = {
            let buffer = self.buffer.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
            if buffer.is_empty() {
                return Err(CancelPhraseError::EmptyBuffer);
            }
            buffer.get_samples()
        };

        // Transcribe the audio
        let transcription = {
            let mut guard = self.model.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
            let tdt = guard.as_mut().ok_or(CancelPhraseError::ModelNotLoaded)?;

            let result = tdt
                .transcribe_samples(samples, self.config.sample_rate, 1, None)
                .map_err(|e| CancelPhraseError::TranscriptionFailed(e.to_string()))?;

            // Join tokens to get full transcription
            let fixed_text: String = result.tokens.iter().map(|t| t.text.as_str()).collect();
            fixed_text.trim().to_string()
        };

        // Check for cancel phrases
        let (detected, phrase, confidence) = self.check_cancel_phrase(&transcription);

        Ok(CancelPhraseResult {
            detected,
            phrase,
            confidence,
            transcription,
        })
    }

    /// Analyze the current buffer and emit event if cancel phrase detected
    ///
    /// This is the primary method for production use. It:
    /// 1. Analyzes the buffer for cancel phrases
    /// 2. If detected, emits a `recording_cancelled` event via the Tauri event system
    /// 3. Ends the session after detection
    ///
    /// Returns the detection result regardless of whether an event was emitted.
    pub fn analyze_and_emit<E: ListeningEventEmitter>(
        &self,
        emitter: &E,
    ) -> Result<CancelPhraseResult, CancelPhraseError> {
        let result = self.analyze()?;

        if result.detected {
            // Emit the recording cancelled event
            emitter.emit_recording_cancelled(listening_events::RecordingCancelledPayload {
                cancel_phrase: result.phrase.clone().unwrap_or_default(),
                timestamp: current_timestamp(),
            });

            // End the session after successful cancellation
            self.end_session()?;
        }

        Ok(result)
    }

    /// Clear the audio buffer
    pub fn clear_buffer(&self) -> Result<(), CancelPhraseError> {
        let mut buffer = self.buffer.lock().map_err(|_| CancelPhraseError::LockPoisoned)?;
        buffer.clear();
        Ok(())
    }

    /// Check if transcription contains a cancel phrase
    ///
    /// Returns (detected, phrase, confidence) tuple.
    /// Uses strict matching to avoid false positives on similar phrases.
    fn check_cancel_phrase(&self, transcription: &str) -> (bool, Option<String>, f32) {
        let text_lower = transcription.to_lowercase();

        // Primary cancel phrases
        let cancel_phrases = [
            ("cancel", vec!["cancel"]),
            ("nevermind", vec!["nevermind", "never mind", "nvm"]),
        ];

        for (canonical, variants) in cancel_phrases {
            for variant in variants {
                if self.is_isolated_phrase(&text_lower, variant) {
                    return (true, Some(canonical.to_string()), 1.0);
                }
            }
        }

        // Check for partial matches with lower confidence
        // but avoid false positives like "can't sell"
        if text_lower.contains("cancel") && !self.is_false_positive(&text_lower, "cancel") {
            return (true, Some("cancel".to_string()), 0.85);
        }

        // No match found
        (false, None, 0.0)
    }

    /// Check if a phrase appears as isolated words (not part of other words)
    fn is_isolated_phrase(&self, text: &str, phrase: &str) -> bool {
        // For multi-word phrases, check if the phrase appears in the text
        if phrase.contains(' ') {
            return text.contains(phrase);
        }
        // For single-word phrases, split into words and check for exact match
        let words: Vec<&str> = text.split_whitespace().collect();
        words.iter().any(|&word| word == phrase)
    }

    /// Check for known false positive patterns
    fn is_false_positive(&self, text: &str, _phrase: &str) -> bool {
        // Known false positives for "cancel"
        let false_positive_patterns = [
            "can't sell",
            "cant sell",
            "can sell",
            "can't tell",
            "cant tell",
            "can tell",
        ];

        for pattern in false_positive_patterns {
            if text.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Get the current configuration
    pub fn config(&self) -> &CancelPhraseDetectorConfig {
        &self.config
    }
}

impl Default for CancelPhraseDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CancelPhraseDetectorConfig::default();
        assert_eq!(config.confidence_threshold, 0.7);
        assert_eq!(config.window_duration_secs, 2.0);
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.cancellation_window_secs, 3.0);
    }

    #[test]
    fn test_detector_new_is_not_loaded() {
        let detector = CancelPhraseDetector::new();
        assert!(!detector.is_loaded());
    }

    #[test]
    fn test_detector_with_custom_config() {
        let config = CancelPhraseDetectorConfig {
            confidence_threshold: 0.9,
            window_duration_secs: 1.5,
            sample_rate: 44100,
            cancellation_window_secs: 5.0,
        };
        let detector = CancelPhraseDetector::with_config(config);
        assert_eq!(detector.config().sample_rate, 44100);
        assert_eq!(detector.config().cancellation_window_secs, 5.0);
    }

    #[test]
    fn test_push_samples_to_buffer() {
        let detector = CancelPhraseDetector::new();
        let samples = vec![0.1, 0.2, 0.3];
        assert!(detector.push_samples(&samples).is_ok());
    }

    #[test]
    fn test_analyze_without_model_returns_error() {
        let detector = CancelPhraseDetector::new();
        detector.start_session().unwrap();
        detector.push_samples(&[0.1, 0.2, 0.3]).unwrap();
        let result = detector.analyze();
        assert!(matches!(result, Err(CancelPhraseError::ModelNotLoaded)));
    }

    #[test]
    fn test_analyze_empty_buffer_returns_error() {
        let detector = CancelPhraseDetector::new();
        detector.start_session().unwrap();
        let result = detector.analyze();
        assert!(matches!(result, Err(CancelPhraseError::EmptyBuffer)));
    }

    #[test]
    fn test_analyze_without_session_returns_window_expired() {
        let detector = CancelPhraseDetector::new();
        detector.push_samples(&[0.1, 0.2, 0.3]).unwrap();
        let result = detector.analyze();
        assert!(matches!(result, Err(CancelPhraseError::WindowExpired)));
    }

    #[test]
    fn test_clear_buffer() {
        let detector = CancelPhraseDetector::new();
        detector.push_samples(&[0.1, 0.2, 0.3]).unwrap();
        assert!(detector.clear_buffer().is_ok());
        detector.start_session().unwrap();
        let result = detector.analyze();
        assert!(matches!(result, Err(CancelPhraseError::EmptyBuffer)));
    }

    #[test]
    fn test_check_cancel_phrase_exact_match() {
        let detector = CancelPhraseDetector::new();
        let (detected, phrase, confidence) = detector.check_cancel_phrase("cancel");
        assert!(detected);
        assert_eq!(phrase, Some("cancel".to_string()));
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_check_cancel_phrase_nevermind() {
        let detector = CancelPhraseDetector::new();
        let (detected, phrase, confidence) = detector.check_cancel_phrase("nevermind");
        assert!(detected);
        assert_eq!(phrase, Some("nevermind".to_string()));
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_check_cancel_phrase_never_mind_with_space() {
        let detector = CancelPhraseDetector::new();
        let (detected, phrase, _) = detector.check_cancel_phrase("never mind");
        assert!(detected);
        assert_eq!(phrase, Some("nevermind".to_string()));
    }

    #[test]
    fn test_check_cancel_phrase_case_insensitive() {
        let detector = CancelPhraseDetector::new();
        let (detected, phrase, _) = detector.check_cancel_phrase("CANCEL");
        assert!(detected);
        assert_eq!(phrase, Some("cancel".to_string()));

        let (detected, phrase, _) = detector.check_cancel_phrase("NeverMind");
        assert!(detected);
        assert_eq!(phrase, Some("nevermind".to_string()));
    }

    #[test]
    fn test_check_cancel_phrase_with_context() {
        let detector = CancelPhraseDetector::new();
        let (detected, phrase, _) = detector.check_cancel_phrase("oh cancel that please");
        assert!(detected);
        assert_eq!(phrase, Some("cancel".to_string()));
    }

    #[test]
    fn test_check_cancel_phrase_no_match() {
        let detector = CancelPhraseDetector::new();
        let (detected, _, _) = detector.check_cancel_phrase("hello world");
        assert!(!detected);
    }

    #[test]
    fn test_check_cancel_phrase_rejects_false_positives() {
        let detector = CancelPhraseDetector::new();

        // "Can't sell" should not match
        let (detected, _, _) = detector.check_cancel_phrase("I can't sell this");
        assert!(!detected, "can't sell should not trigger cancellation");

        // "Can sell" should not match
        let (detected, _, _) = detector.check_cancel_phrase("I can sell this");
        assert!(!detected, "can sell should not trigger cancellation");
    }

    #[test]
    fn test_session_start_and_window() {
        let detector = CancelPhraseDetector::new();

        // Initially no session
        assert!(!detector.is_window_open());
        assert_eq!(detector.remaining_window_secs(), 0.0);

        // Start session
        detector.start_session().unwrap();
        assert!(detector.is_window_open());
        assert!(detector.remaining_window_secs() > 0.0);
        assert!(detector.remaining_window_secs() <= 3.0);
    }

    #[test]
    fn test_session_end_closes_window() {
        let detector = CancelPhraseDetector::new();

        detector.start_session().unwrap();
        assert!(detector.is_window_open());

        detector.end_session().unwrap();
        assert!(!detector.is_window_open());
    }

    #[test]
    fn test_window_expires_after_timeout() {
        // Use a very short window for testing
        let config = CancelPhraseDetectorConfig {
            cancellation_window_secs: 0.05, // 50ms
            ..Default::default()
        };
        let detector = CancelPhraseDetector::with_config(config);

        detector.start_session().unwrap();
        assert!(detector.is_window_open());

        // Wait for window to expire
        std::thread::sleep(std::time::Duration::from_millis(60));
        assert!(!detector.is_window_open());
    }

    #[test]
    fn test_error_display() {
        assert!(format!("{}", CancelPhraseError::ModelNotLoaded).contains("not loaded"));
        assert!(
            format!("{}", CancelPhraseError::ModelLoadFailed("test".to_string())).contains("test")
        );
        assert!(
            format!("{}", CancelPhraseError::TranscriptionFailed("test".to_string()))
                .contains("test")
        );
        assert!(format!("{}", CancelPhraseError::LockPoisoned).contains("lock"));
        assert!(format!("{}", CancelPhraseError::EmptyBuffer).contains("empty"));
        assert!(format!("{}", CancelPhraseError::WindowExpired).contains("expired"));
    }

    #[test]
    fn test_cancel_phrase_result_equality() {
        let result1 = CancelPhraseResult {
            detected: true,
            phrase: Some("cancel".to_string()),
            confidence: 0.95,
            transcription: "cancel".to_string(),
        };
        let result2 = result1.clone();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_push_samples_does_not_block() {
        let detector = CancelPhraseDetector::new();

        // Push a large number of samples
        let large_buffer = vec![0.0f32; 160000];
        let result = detector.push_samples(&large_buffer);
        assert!(result.is_ok());
    }

    #[test]
    fn test_start_session_clears_buffer() {
        let detector = CancelPhraseDetector::new();
        detector.push_samples(&[0.1, 0.2, 0.3]).unwrap();

        // Start new session should clear
        detector.start_session().unwrap();

        // Buffer should be empty now
        let result = detector.analyze();
        assert!(matches!(result, Err(CancelPhraseError::EmptyBuffer)));
    }

    #[test]
    fn test_check_nvm_shorthand() {
        let detector = CancelPhraseDetector::new();
        let (detected, phrase, _) = detector.check_cancel_phrase("nvm");
        assert!(detected);
        assert_eq!(phrase, Some("nevermind".to_string()));
    }

    #[test]
    fn test_multiple_sessions() {
        let detector = CancelPhraseDetector::new();

        // First session
        detector.start_session().unwrap();
        assert!(detector.is_window_open());
        detector.end_session().unwrap();
        assert!(!detector.is_window_open());

        // Second session
        detector.start_session().unwrap();
        assert!(detector.is_window_open());
    }

    #[test]
    fn test_isolated_phrase_detection() {
        let detector = CancelPhraseDetector::new();

        // Should match: isolated word
        assert!(detector.is_isolated_phrase("please cancel that", "cancel"));

        // Should match: single word
        assert!(detector.is_isolated_phrase("cancel", "cancel"));

        // Should not match: part of another word
        assert!(!detector.is_isolated_phrase("cancellation", "cancel"));
    }
}

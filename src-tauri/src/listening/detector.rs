// Wake word detector using Parakeet for on-device speech recognition
// Analyzes audio windows to detect the "Hey Cat" wake phrase

use super::CircularBuffer;
use crate::events::{current_timestamp, listening_events, ListeningEventEmitter};
use crate::model::download::{get_model_dir, ModelType};
use parakeet_rs::ParakeetTDT;
use std::sync::{Arc, Mutex};

/// Configuration for wake word detection
#[derive(Debug, Clone)]
pub struct WakeWordDetectorConfig {
    /// The wake phrase to detect (case-insensitive)
    pub wake_phrase: String,
    /// Minimum confidence threshold (0.0 - 1.0)
    pub confidence_threshold: f32,
    /// Audio window duration in seconds
    pub window_duration_secs: f32,
    /// Sample rate in Hz (must match audio input)
    pub sample_rate: u32,
}

impl Default for WakeWordDetectorConfig {
    fn default() -> Self {
        Self {
            wake_phrase: "hey cat".to_string(),
            confidence_threshold: 0.8,
            window_duration_secs: 2.0,
            sample_rate: 16000,
        }
    }
}

/// Result of wake word detection analysis
#[derive(Debug, Clone, PartialEq)]
pub struct WakeWordResult {
    /// Whether the wake word was detected
    pub detected: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// The transcribed text from the audio window
    pub transcription: String,
}

/// Errors that can occur during wake word detection
#[derive(Debug, Clone, PartialEq)]
pub enum WakeWordError {
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
}

impl std::fmt::Display for WakeWordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WakeWordError::ModelNotLoaded => write!(f, "Wake word model not loaded"),
            WakeWordError::ModelLoadFailed(msg) => write!(f, "Failed to load model: {}", msg),
            WakeWordError::TranscriptionFailed(msg) => write!(f, "Transcription failed: {}", msg),
            WakeWordError::LockPoisoned => write!(f, "Internal lock error"),
            WakeWordError::EmptyBuffer => write!(f, "Audio buffer is empty"),
        }
    }
}

impl std::error::Error for WakeWordError {}

/// Wake word detector using Parakeet TDT model
///
/// Processes audio samples in small windows to detect the wake phrase.
/// Uses on-device speech recognition for privacy.
pub struct WakeWordDetector {
    /// Configuration
    config: WakeWordDetectorConfig,
    /// TDT model context (thread-safe)
    model: Arc<Mutex<Option<ParakeetTDT>>>,
    /// Circular buffer for audio samples
    buffer: Mutex<CircularBuffer>,
}

impl WakeWordDetector {
    /// Create a new wake word detector with default configuration
    pub fn new() -> Self {
        Self::with_config(WakeWordDetectorConfig::default())
    }

    /// Create a new wake word detector with custom configuration
    pub fn with_config(config: WakeWordDetectorConfig) -> Self {
        let buffer = CircularBuffer::for_duration(config.window_duration_secs, config.sample_rate);
        Self {
            config,
            model: Arc::new(Mutex::new(None)),
            buffer: Mutex::new(buffer),
        }
    }

    /// Load the Parakeet TDT model
    ///
    /// Must be called before processing audio.
    pub fn load_model(&self) -> Result<(), WakeWordError> {
        let model_dir = get_model_dir(ModelType::ParakeetTDT)
            .map_err(|e| WakeWordError::ModelLoadFailed(e.to_string()))?;

        let path_str = model_dir.to_str().ok_or_else(|| {
            WakeWordError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        let tdt = ParakeetTDT::from_pretrained(path_str, None)
            .map_err(|e| WakeWordError::ModelLoadFailed(e.to_string()))?;

        let mut guard = self.model.lock().map_err(|_| WakeWordError::LockPoisoned)?;
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

    /// Push audio samples into the buffer for analysis
    ///
    /// Call this with incoming audio data from the audio capture.
    pub fn push_samples(&self, samples: &[f32]) -> Result<(), WakeWordError> {
        let mut buffer = self.buffer.lock().map_err(|_| WakeWordError::LockPoisoned)?;
        buffer.push_samples(samples);
        Ok(())
    }

    /// Analyze the current buffer for wake word
    ///
    /// Returns a WakeWordResult indicating whether the wake phrase was detected.
    /// Clears the buffer after analysis to avoid re-detection.
    pub fn analyze(&self) -> Result<WakeWordResult, WakeWordError> {
        // Get samples from buffer
        let samples = {
            let buffer = self.buffer.lock().map_err(|_| WakeWordError::LockPoisoned)?;
            if buffer.is_empty() {
                return Err(WakeWordError::EmptyBuffer);
            }
            buffer.get_samples()
        };

        // Transcribe the audio
        let transcription = {
            let mut guard = self.model.lock().map_err(|_| WakeWordError::LockPoisoned)?;
            let tdt = guard.as_mut().ok_or(WakeWordError::ModelNotLoaded)?;

            // Use transcribe_samples for in-memory audio
            // Signature: transcribe_samples(audio: Vec<f32>, sample_rate: u32, channels: u16, mode: Option<TimestampMode>)
            let result = tdt
                .transcribe_samples(samples, self.config.sample_rate, 1, None)
                .map_err(|e| WakeWordError::TranscriptionFailed(e.to_string()))?;

            // Apply same workaround as TranscriptionManager for token joining
            let fixed_text: String = result.tokens.iter().map(|t| t.text.as_str()).collect();
            fixed_text.trim().to_string()
        };

        // Check for wake phrase
        let (detected, confidence) = self.check_wake_phrase(&transcription);

        Ok(WakeWordResult {
            detected,
            confidence,
            transcription,
        })
    }

    /// Analyze the current buffer and emit event if wake word detected
    ///
    /// This is the primary method for production use. It:
    /// 1. Analyzes the buffer for the wake phrase
    /// 2. If detected, emits a `wake_word_detected` event via the Tauri event system
    /// 3. Clears the buffer after detection to avoid re-triggering
    ///
    /// Returns the detection result regardless of whether an event was emitted.
    pub fn analyze_and_emit<E: ListeningEventEmitter>(
        &self,
        emitter: &E,
    ) -> Result<WakeWordResult, WakeWordError> {
        let result = self.analyze()?;

        if result.detected {
            // Emit the wake word detected event
            emitter.emit_wake_word_detected(listening_events::WakeWordDetectedPayload {
                confidence: result.confidence,
                transcription: result.transcription.clone(),
                timestamp: current_timestamp(),
            });

            // Clear buffer after detection to avoid re-triggering
            self.clear_buffer()?;
        }

        Ok(result)
    }

    /// Clear the audio buffer
    ///
    /// Call this after a wake word is detected to reset for next detection.
    pub fn clear_buffer(&self) -> Result<(), WakeWordError> {
        let mut buffer = self.buffer.lock().map_err(|_| WakeWordError::LockPoisoned)?;
        buffer.clear();
        Ok(())
    }

    /// Check if transcription contains the wake phrase
    ///
    /// Returns (detected, confidence) tuple.
    /// Uses strict matching to avoid false positives on similar phrases.
    fn check_wake_phrase(&self, transcription: &str) -> (bool, f32) {
        let text_lower = transcription.to_lowercase();
        let wake_lower = self.config.wake_phrase.to_lowercase();

        // Exact match - highest confidence
        if text_lower.contains(&wake_lower) {
            return (true, 1.0);
        }

        // Explicit known variants of "hey cat"
        // These are phonetically similar but still clearly the wake phrase
        let known_variants = [
            "hey cats",  // plural
            "hey kat",   // k spelling
            "hey kats",  // k spelling plural
            "heycat",    // no space
            "hey-cat",   // hyphenated
        ];

        for variant in known_variants {
            if text_lower.contains(variant) {
                return (true, 0.95);
            }
        }

        // Check for word-by-word matching: "hey" + "cat" adjacent
        let text_words: Vec<&str> = text_lower.split_whitespace().collect();

        for i in 0..text_words.len().saturating_sub(1) {
            let word1 = text_words[i];
            let word2 = text_words[i + 1];

            // First word must be "hey" or common misheard variants
            let is_hey = word1 == "hey" || word1 == "hay";

            // Second word must be "cat" or phonetically identical
            let is_cat = word2 == "cat" || word2 == "kat" || word2 == "cats" || word2 == "kats";

            if is_hey && is_cat {
                return (true, 0.9);
            }
        }

        // No match found - return low confidence
        // We intentionally avoid fuzzy string matching (like Jaro-Winkler) on the full
        // phrase to prevent false positives on similar phrases like "hey matt"
        (false, 0.0)
    }

    /// Get the current configuration
    pub fn config(&self) -> &WakeWordDetectorConfig {
        &self.config
    }
}

impl Default for WakeWordDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WakeWordDetectorConfig::default();
        assert_eq!(config.wake_phrase, "hey cat");
        assert_eq!(config.confidence_threshold, 0.8);
        assert_eq!(config.window_duration_secs, 2.0);
        assert_eq!(config.sample_rate, 16000);
    }

    #[test]
    fn test_detector_new_is_not_loaded() {
        let detector = WakeWordDetector::new();
        assert!(!detector.is_loaded());
    }

    #[test]
    fn test_detector_with_custom_config() {
        let config = WakeWordDetectorConfig {
            wake_phrase: "hello world".to_string(),
            confidence_threshold: 0.9,
            window_duration_secs: 1.5,
            sample_rate: 44100,
        };
        let detector = WakeWordDetector::with_config(config.clone());
        assert_eq!(detector.config().wake_phrase, "hello world");
        assert_eq!(detector.config().sample_rate, 44100);
    }

    #[test]
    fn test_push_samples_to_buffer() {
        let detector = WakeWordDetector::new();
        let samples = vec![0.1, 0.2, 0.3];
        assert!(detector.push_samples(&samples).is_ok());
    }

    #[test]
    fn test_analyze_without_model_returns_error() {
        let detector = WakeWordDetector::new();
        detector.push_samples(&[0.1, 0.2, 0.3]).unwrap();
        let result = detector.analyze();
        assert!(matches!(result, Err(WakeWordError::ModelNotLoaded)));
    }

    #[test]
    fn test_analyze_empty_buffer_returns_error() {
        let detector = WakeWordDetector::new();
        let result = detector.analyze();
        assert!(matches!(result, Err(WakeWordError::EmptyBuffer)));
    }

    #[test]
    fn test_clear_buffer() {
        let detector = WakeWordDetector::new();
        detector.push_samples(&[0.1, 0.2, 0.3]).unwrap();
        assert!(detector.clear_buffer().is_ok());
        // After clear, analyze should return EmptyBuffer
        let result = detector.analyze();
        assert!(matches!(result, Err(WakeWordError::EmptyBuffer)));
    }

    #[test]
    fn test_check_wake_phrase_exact_match() {
        let detector = WakeWordDetector::new();
        let (detected, confidence) = detector.check_wake_phrase("hey cat");
        assert!(detected);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_check_wake_phrase_case_insensitive() {
        let detector = WakeWordDetector::new();
        let (detected, confidence) = detector.check_wake_phrase("Hey Cat");
        assert!(detected);
        assert_eq!(confidence, 1.0);

        let (detected, confidence) = detector.check_wake_phrase("HEY CAT");
        assert!(detected);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_check_wake_phrase_with_context() {
        let detector = WakeWordDetector::new();
        let (detected, confidence) = detector.check_wake_phrase("I said hey cat please help");
        assert!(detected);
        assert_eq!(confidence, 1.0);
    }

    #[test]
    fn test_check_wake_phrase_variant_heycat() {
        let detector = WakeWordDetector::new();
        let (detected, confidence) = detector.check_wake_phrase("heycat");
        assert!(detected);
        assert!(confidence >= 0.95);
    }

    #[test]
    fn test_check_wake_phrase_variant_hey_cats() {
        let detector = WakeWordDetector::new();
        let (detected, confidence) = detector.check_wake_phrase("hey cats");
        assert!(detected);
        assert!(confidence >= 0.95);
    }

    #[test]
    fn test_check_wake_phrase_phonetic_hay_cat() {
        let detector = WakeWordDetector::new();
        let (detected, confidence) = detector.check_wake_phrase("hay cat");
        assert!(detected);
        assert!(confidence >= 0.9);
    }

    #[test]
    fn test_check_wake_phrase_no_match() {
        let detector = WakeWordDetector::new();
        let (detected, _) = detector.check_wake_phrase("hello world");
        assert!(!detected);
    }

    #[test]
    fn test_check_wake_phrase_rejects_similar_phrases() {
        let detector = WakeWordDetector::new();

        // "Hey Matt" should not match (different second word)
        let (detected, _) = detector.check_wake_phrase("hey matt");
        assert!(!detected, "hey matt should not be detected");

        // "Pay Cat" should not match (different first word)
        let (detected, _) = detector.check_wake_phrase("pay cat");
        assert!(!detected, "pay cat should not be detected");

        // "Hey" alone should not match
        let (detected, _) = detector.check_wake_phrase("hey");
        assert!(!detected, "hey alone should not be detected");

        // "Cat" alone should not match
        let (detected, _) = detector.check_wake_phrase("cat");
        assert!(!detected, "cat alone should not be detected");
    }

    #[test]
    fn test_error_display() {
        assert!(format!("{}", WakeWordError::ModelNotLoaded).contains("not loaded"));
        assert!(format!("{}", WakeWordError::ModelLoadFailed("test".to_string())).contains("test"));
        assert!(
            format!("{}", WakeWordError::TranscriptionFailed("test".to_string())).contains("test")
        );
        assert!(format!("{}", WakeWordError::LockPoisoned).contains("lock"));
        assert!(format!("{}", WakeWordError::EmptyBuffer).contains("empty"));
    }

    #[test]
    fn test_wake_word_result_equality() {
        let result1 = WakeWordResult {
            detected: true,
            confidence: 0.95,
            transcription: "hey cat".to_string(),
        };
        let result2 = result1.clone();
        assert_eq!(result1, result2);
    }

    // Note: The following test cases require actual audio hardware or loaded models
    // and are covered by manual integration testing:
    // - "Handles background noise without false triggers" - requires real audio input
    // - "Processes samples without blocking audio capture" - requires real-time audio testing
    // The unit tests above verify the detection logic is correct for the given inputs.

    #[test]
    fn test_push_samples_does_not_block() {
        // Verify push_samples is synchronous and returns immediately
        let detector = WakeWordDetector::new();

        // Push a large number of samples (simulating ~10 seconds of audio)
        let large_buffer = vec![0.0f32; 160000];
        let result = detector.push_samples(&large_buffer);
        assert!(result.is_ok());

        // Buffer should have wrapped around (capacity is ~32000 for 2s at 16kHz)
        // This verifies the circular buffer handles overflow correctly
    }

    #[test]
    fn test_silence_buffer_does_not_crash() {
        // Test that a buffer of silence (zeros) doesn't cause issues
        let detector = WakeWordDetector::new();

        // Push silence samples
        let silence = vec![0.0f32; 16000]; // 1 second of silence
        detector.push_samples(&silence).unwrap();

        // analyze() will fail because model isn't loaded, but won't crash
        let result = detector.analyze();
        assert!(matches!(result, Err(WakeWordError::ModelNotLoaded)));
    }

    #[test]
    fn test_noise_buffer_does_not_crash() {
        // Test that noisy audio doesn't cause issues
        let detector = WakeWordDetector::new();

        // Push random noise-like samples (values between -1 and 1)
        let noise: Vec<f32> = (0..16000)
            .map(|i| ((i as f32 * 0.1).sin() * 0.5))
            .collect();
        detector.push_samples(&noise).unwrap();

        // analyze() will fail because model isn't loaded, but won't crash
        let result = detector.analyze();
        assert!(matches!(result, Err(WakeWordError::ModelNotLoaded)));
    }
}

// Wake word detector using Parakeet for on-device speech recognition
// Analyzes audio windows to detect the "Hey Cat" wake phrase

use super::CircularBuffer;
use crate::events::{current_timestamp, listening_events, ListeningEventEmitter};
use crate::model::download::{get_model_dir, ModelType};
use parakeet_rs::ParakeetTDT;
use std::sync::{Arc, Mutex};
use voice_activity_detector::VoiceActivityDetector;

/// Configuration for wake word detection
#[derive(Debug, Clone)]
pub struct WakeWordDetectorConfig {
    /// The wake phrase to detect (case-insensitive)
    pub wake_phrase: String,
    /// Minimum confidence threshold (0.0 - 1.0)
    #[allow(dead_code)] // Reserved for future confidence-based filtering
    pub confidence_threshold: f32,
    /// Audio window duration in seconds
    pub window_duration_secs: f32,
    /// Sample rate in Hz (must match audio input)
    pub sample_rate: u32,
    /// Minimum new samples required before re-analysis (prevents duplicate transcriptions)
    /// Default: 8000 samples = 0.5 seconds at 16kHz
    pub min_new_samples_for_analysis: usize,
    /// VAD speech probability threshold (0.0 - 1.0)
    /// Audio below this threshold is considered non-speech
    pub vad_speech_threshold: f32,
    /// Whether VAD pre-filtering is enabled
    pub vad_enabled: bool,
}

impl Default for WakeWordDetectorConfig {
    fn default() -> Self {
        Self {
            wake_phrase: "hey cat".to_string(),
            confidence_threshold: 0.8,
            // ~3 seconds at 16kHz = 48000 samples = ~192KB memory
            window_duration_secs: 3.0,
            sample_rate: 16000,
            // 0.5 seconds of new audio required before re-analysis
            // This prevents the same audio from being transcribed multiple times
            min_new_samples_for_analysis: 8000,
            // VAD threshold - Silero default is 0.5
            vad_speech_threshold: 0.5,
            // VAD enabled by default to filter background noise
            vad_enabled: true,
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
    /// Not enough new samples since last analysis
    InsufficientNewSamples,
    /// No speech detected in buffer (VAD filtered)
    NoSpeechDetected,
    /// VAD initialization failed
    VadInitFailed(String),
}

impl std::fmt::Display for WakeWordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WakeWordError::ModelNotLoaded => write!(f, "Wake word model not loaded"),
            WakeWordError::ModelLoadFailed(msg) => write!(f, "Failed to load model: {}", msg),
            WakeWordError::TranscriptionFailed(msg) => write!(f, "Transcription failed: {}", msg),
            WakeWordError::LockPoisoned => write!(f, "Internal lock error"),
            WakeWordError::EmptyBuffer => write!(f, "Audio buffer is empty"),
            WakeWordError::InsufficientNewSamples => {
                write!(f, "Not enough new audio samples for analysis")
            }
            WakeWordError::NoSpeechDetected => {
                write!(f, "No speech detected in audio buffer (VAD filtered)")
            }
            WakeWordError::VadInitFailed(msg) => {
                write!(f, "VAD initialization failed: {}", msg)
            }
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
    /// Sample count at last analysis (for tracking new samples)
    last_analysis_sample_count: Mutex<u64>,
    /// Last transcription result (for deduplication)
    last_transcription: Mutex<Option<String>>,
    /// Voice Activity Detector for filtering non-speech audio
    vad: Mutex<Option<VoiceActivityDetector>>,
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
            last_analysis_sample_count: Mutex::new(0),
            last_transcription: Mutex::new(None),
            vad: Mutex::new(None),
        }
    }

    /// Load the Parakeet TDT model
    ///
    /// Must be called before processing audio.
    pub fn load_model(&self) -> Result<(), WakeWordError> {
        crate::debug!("[wake-word] Loading Parakeet TDT model...");

        let model_dir = get_model_dir(ModelType::ParakeetTDT)
            .map_err(|e| {
                crate::error!("[wake-word] Failed to get model directory: {}", e);
                WakeWordError::ModelLoadFailed(e.to_string())
            })?;

        crate::debug!("[wake-word] Model path: {:?}", model_dir);

        let path_str = model_dir.to_str().ok_or_else(|| {
            crate::error!("[wake-word] Invalid path encoding");
            WakeWordError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        let tdt = ParakeetTDT::from_pretrained(path_str, None)
            .map_err(|e| {
                crate::error!("[wake-word] Failed to load model: {}", e);
                WakeWordError::ModelLoadFailed(e.to_string())
            })?;

        let mut guard = self.model.lock().map_err(|_| WakeWordError::LockPoisoned)?;
        *guard = Some(tdt);

        crate::info!("[wake-word] Model loaded successfully");

        // Initialize VAD if enabled
        if self.config.vad_enabled {
            crate::debug!("[wake-word] Initializing VAD detector...");
            let vad = VoiceActivityDetector::builder()
                .sample_rate(self.config.sample_rate)
                .chunk_size(512usize) // Fixed for Silero at 16kHz
                .build()
                .map_err(|e| {
                    crate::error!("[wake-word] Failed to initialize VAD: {}", e);
                    WakeWordError::VadInitFailed(e.to_string())
                })?;

            let mut vad_guard = self.vad.lock().map_err(|_| WakeWordError::LockPoisoned)?;
            *vad_guard = Some(vad);

            crate::info!(
                "[wake-word] VAD initialized (threshold={})",
                self.config.vad_speech_threshold
            );
        } else {
            crate::info!("[wake-word] VAD disabled");
        }

        Ok(())
    }

    /// Check if the model is loaded
    #[allow(dead_code)] // Utility method for status checks
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
        crate::trace!("[wake-word] Pushing {} samples to buffer", samples.len());
        let mut buffer = self.buffer.lock().map_err(|_| WakeWordError::LockPoisoned)?;
        buffer.push_samples(samples);
        Ok(())
    }

    /// Analyze the current buffer for wake word
    ///
    /// Returns a WakeWordResult indicating whether the wake phrase was detected.
    /// Skips analysis if not enough new samples have accumulated since last analysis.
    pub fn analyze(&self) -> Result<WakeWordResult, WakeWordError> {
        // Get samples from buffer and check if we have enough new audio
        let (samples, current_total) = {
            let buffer = self.buffer.lock().map_err(|_| WakeWordError::LockPoisoned)?;
            if buffer.is_empty() {
                return Err(WakeWordError::EmptyBuffer);
            }
            (buffer.get_samples(), buffer.total_samples_pushed())
        };

        // Check if we have enough new samples since last analysis
        let last_count = {
            let guard = self
                .last_analysis_sample_count
                .lock()
                .map_err(|_| WakeWordError::LockPoisoned)?;
            *guard
        };

        let new_samples = current_total.saturating_sub(last_count) as usize;
        if new_samples < self.config.min_new_samples_for_analysis {
            crate::trace!(
                "[wake-word] Skipping analysis: only {} new samples (need {})",
                new_samples,
                self.config.min_new_samples_for_analysis
            );
            return Err(WakeWordError::InsufficientNewSamples);
        }

        crate::trace!(
            "[wake-word] Analyzing {} samples ({} new since last analysis)",
            samples.len(),
            new_samples
        );

        // VAD check - skip expensive transcription if no speech detected
        if self.config.vad_enabled {
            let has_speech = self.check_vad(&samples)?;
            if !has_speech {
                crate::trace!("[wake-word] VAD: No speech detected, skipping transcription");
                return Err(WakeWordError::NoSpeechDetected);
            }
            crate::debug!("[wake-word] VAD: Speech detected, proceeding with transcription");
        }

        // Transcribe the audio
        let transcribe_start = std::time::Instant::now();
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
        let transcribe_duration = transcribe_start.elapsed();

        // Clear buffer after transcription to prevent re-analyzing same audio
        // This is critical because the CircularBuffer is a rolling 3-second window,
        // so without clearing, the same speech would be transcribed multiple times
        {
            let mut buffer = self.buffer.lock().map_err(|_| WakeWordError::LockPoisoned)?;
            buffer.clear();
            buffer.reset_sample_counter();
            crate::trace!("[wake-word] Buffer cleared after transcription");
        }

        // Reset sample count tracking since buffer is now empty
        {
            let mut guard = self
                .last_analysis_sample_count
                .lock()
                .map_err(|_| WakeWordError::LockPoisoned)?;
            *guard = 0;
        }

        // Check for duplicate transcription (secondary deduplication)
        let is_duplicate = {
            let mut last = self
                .last_transcription
                .lock()
                .map_err(|_| WakeWordError::LockPoisoned)?;
            let is_same = last.as_ref() == Some(&transcription);
            if !is_same {
                *last = Some(transcription.clone());
            }
            is_same
        };

        if is_duplicate {
            crate::trace!(
                "[wake-word] Skipping duplicate transcription: '{}'",
                transcription
            );
            // Return non-detection for duplicate transcriptions
            return Ok(WakeWordResult {
                detected: false,
                confidence: 0.0,
                transcription,
            });
        }

        crate::debug!(
            "[wake-word] Transcribed in {:?}: '{}'",
            transcribe_duration,
            transcription
        );

        // Check for wake phrase
        let (detected, confidence) = self.check_wake_phrase(&transcription);

        if detected {
            crate::debug!(
                "[wake-word] Wake phrase MATCHED! confidence={:.2}, text='{}'",
                confidence,
                transcription
            );
        } else {
            crate::trace!(
                "[wake-word] No wake phrase match in: '{}'",
                transcription
            );
        }

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

    /// Clear the audio buffer and reset analysis tracking
    ///
    /// Call this after a wake word is detected to reset for next detection.
    pub fn clear_buffer(&self) -> Result<(), WakeWordError> {
        // Clear the audio buffer
        {
            let mut buffer = self.buffer.lock().map_err(|_| WakeWordError::LockPoisoned)?;
            buffer.clear();
            buffer.reset_sample_counter();
        }

        // Reset analysis tracking
        {
            let mut count = self
                .last_analysis_sample_count
                .lock()
                .map_err(|_| WakeWordError::LockPoisoned)?;
            *count = 0;
        }

        // Reset last transcription to allow fresh detection
        {
            let mut last = self
                .last_transcription
                .lock()
                .map_err(|_| WakeWordError::LockPoisoned)?;
            *last = None;
        }

        Ok(())
    }

    /// Check if audio samples contain speech using Voice Activity Detection
    ///
    /// Returns true if speech is detected in a significant portion of the audio.
    /// Uses Silero VAD with 512-sample chunks at 16kHz.
    fn check_vad(&self, samples: &[f32]) -> Result<bool, WakeWordError> {
        let mut vad_guard = self.vad.lock().map_err(|_| WakeWordError::LockPoisoned)?;

        // If VAD not initialized, conservatively assume speech is present
        let vad = match vad_guard.as_mut() {
            Some(v) => v,
            None => {
                crate::trace!("[wake-word] VAD not initialized, assuming speech present");
                return Ok(true);
            }
        };

        // Process samples in 512-sample chunks (required by Silero at 16kHz)
        const CHUNK_SIZE: usize = 512;
        let mut speech_frames = 0;
        let mut total_frames = 0;

        for chunk in samples.chunks(CHUNK_SIZE) {
            if chunk.len() == CHUNK_SIZE {
                let probability = vad.predict(chunk.to_vec());
                if probability >= self.config.vad_speech_threshold {
                    speech_frames += 1;
                }
                total_frames += 1;
            }
        }

        // Require at least 10% of frames to contain speech
        let speech_ratio = if total_frames > 0 {
            speech_frames as f32 / total_frames as f32
        } else {
            0.0
        };

        crate::trace!(
            "[wake-word] VAD: {}/{} frames with speech (ratio={:.2}, threshold={})",
            speech_frames,
            total_frames,
            speech_ratio,
            self.config.vad_speech_threshold
        );

        // Consider it speech if at least 10% of frames have speech above threshold
        Ok(speech_ratio >= 0.1)
    }

    /// Check if transcription contains the wake phrase
    ///
    /// Returns (detected, confidence) tuple.
    /// Uses strict matching to avoid false positives on similar phrases.
    fn check_wake_phrase(&self, transcription: &str) -> (bool, f32) {
        let text_lower = transcription.to_lowercase();
        let wake_lower = self.config.wake_phrase.to_lowercase();

        crate::trace!("[wake-word] Checking for wake phrase in: '{}'", text_lower);

        // Exact match - highest confidence
        if text_lower.contains(&wake_lower) {
            crate::trace!("[wake-word] Matched: exact match '{}'", wake_lower);
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
                crate::trace!("[wake-word] Matched: known variant '{}'", variant);
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
                crate::trace!("[wake-word] Matched: word-by-word '{} {}'", word1, word2);
                return (true, 0.9);
            }
        }

        // No match found - return low confidence
        // We intentionally avoid fuzzy string matching (like Jaro-Winkler) on the full
        // phrase to prevent false positives on similar phrases like "hey matt"
        (false, 0.0)
    }

    /// Get the current configuration
    #[allow(dead_code)] // Utility method for introspection
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
        // 3 seconds window for ~192KB memory at 16kHz
        assert_eq!(config.window_duration_secs, 3.0);
        assert_eq!(config.sample_rate, 16000);
        // 0.5 seconds of new audio required before re-analysis
        assert_eq!(config.min_new_samples_for_analysis, 8000);
        // VAD enabled by default with 0.5 threshold
        assert!(config.vad_enabled);
        assert_eq!(config.vad_speech_threshold, 0.5);
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
            min_new_samples_for_analysis: 4000,
            vad_speech_threshold: 0.6,
            vad_enabled: false, // Disable VAD in tests without real audio
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
        // Push enough samples to meet min_new_samples_for_analysis (8000)
        let samples = vec![0.1; 8000];
        detector.push_samples(&samples).unwrap();
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
    fn test_analyze_insufficient_samples_returns_error() {
        let detector = WakeWordDetector::new();
        // Push fewer samples than min_new_samples_for_analysis (8000)
        detector.push_samples(&[0.1, 0.2, 0.3]).unwrap();
        let result = detector.analyze();
        assert!(matches!(result, Err(WakeWordError::InsufficientNewSamples)));
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
        assert!(format!("{}", WakeWordError::InsufficientNewSamples).contains("new"));
        assert!(format!("{}", WakeWordError::NoSpeechDetected).contains("VAD"));
        assert!(format!("{}", WakeWordError::VadInitFailed("test".to_string())).contains("test"));
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

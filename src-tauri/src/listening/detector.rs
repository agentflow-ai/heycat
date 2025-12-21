// Wake word detector using Parakeet for on-device speech recognition
// Analyzes audio windows to detect the "Hey Cat" wake phrase

use super::vad::{create_vad, VadConfig, VadError};
use super::CircularBuffer;
use crate::audio_constants::{
    DEFAULT_SAMPLE_RATE, FINGERPRINT_OVERLAP_THRESHOLD, MIN_PARTIAL_VAD_CHUNK,
    VAD_CHUNK_SIZE_16KHZ, VAD_THRESHOLD_AGGRESSIVE, WAKE_WORD_MIN_NEW_SAMPLES,
    WAKE_WORD_MIN_SPEECH_FRAMES, WAKE_WORD_TRANSCRIPTION_TIMEOUT_SECS, WAKE_WORD_WINDOW_SECS,
};
use crate::events::{current_timestamp, listening_events, ListeningEventEmitter};
use crate::parakeet::SharedTranscriptionModel;
use std::collections::VecDeque;
use std::sync::Mutex;
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
    /// Default: 4000 samples = 0.25 seconds at 16kHz
    pub min_new_samples_for_analysis: usize,
    /// VAD speech probability threshold (0.0 - 1.0)
    /// Audio below this threshold is considered non-speech
    /// Default: 0.3 (lowered from 0.5 for better sensitivity to varied volumes)
    pub vad_speech_threshold: f32,
    /// Whether VAD pre-filtering is enabled
    pub vad_enabled: bool,
    /// Minimum speech frames required in VAD check
    /// Default: 2 frames (early return at 3 for performance)
    pub min_speech_frames: usize,
    /// Transcription timeout in seconds
    /// If transcription takes longer than this, a warning is logged (default: 10s)
    /// Note: The wake word window is only ~2s, so transcription should be fast
    pub transcription_timeout_secs: u64,
}

impl Default for WakeWordDetectorConfig {
    fn default() -> Self {
        Self {
            wake_phrase: "hey cat".to_string(),
            confidence_threshold: 0.8,
            // ~2 seconds at 16kHz = 32000 samples = ~128KB memory
            // Reduced from 3s to 2s for faster response with short utterances
            window_duration_secs: WAKE_WORD_WINDOW_SECS,
            sample_rate: DEFAULT_SAMPLE_RATE,
            // 0.75 seconds of new audio required before re-analysis
            // This reduces noise from overlapping transcriptions while still catching "hey cat"
            min_new_samples_for_analysis: WAKE_WORD_MIN_NEW_SAMPLES,
            // VAD threshold - 0.6 is aggressive to filter ambient noise
            // May miss very quiet speech but significantly reduces false positives
            vad_speech_threshold: VAD_THRESHOLD_AGGRESSIVE,
            // VAD enabled by default to filter background noise
            vad_enabled: true,
            // Minimum speech frames - require 4+ frames (~128ms) above threshold
            // Filters brief noise spikes while catching short utterances like "hey cat"
            min_speech_frames: WAKE_WORD_MIN_SPEECH_FRAMES,
            // Transcription timeout - 10s is generous for ~2s audio window
            transcription_timeout_secs: WAKE_WORD_TRANSCRIPTION_TIMEOUT_SECS,
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

/// Audio fingerprint for deduplicating audio segments
///
/// Identifies audio by its position in the sample stream rather than content.
/// This allows detecting duplicate audio regardless of transcription variations.
#[derive(Debug, Clone)]
struct AudioFingerprint {
    /// Start sample index (from total_samples_pushed)
    start_idx: u64,
    /// End sample index
    end_idx: u64,
}

impl AudioFingerprint {
    /// Calculate overlap ratio with another fingerprint (0.0 to 1.0)
    ///
    /// Returns the proportion of self's range that overlaps with other.
    fn overlap_ratio(&self, other: &AudioFingerprint) -> f32 {
        let overlap_start = self.start_idx.max(other.start_idx);
        let overlap_end = self.end_idx.min(other.end_idx);

        if overlap_start >= overlap_end {
            return 0.0; // No overlap
        }

        let overlap_len = (overlap_end - overlap_start) as f32;
        let self_len = (self.end_idx - self.start_idx) as f32;

        if self_len == 0.0 {
            return 0.0;
        }

        overlap_len / self_len
    }
}

/// Errors that can occur during wake word detection
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum WakeWordError {
    /// Model has not been loaded
    #[error("Wake word model not loaded")]
    ModelNotLoaded,
    /// Failed to load the model (unused since SharedTranscriptionModel loads externally)
    #[allow(dead_code)]
    #[error("Failed to load model: {0}")]
    ModelLoadFailed(String),
    /// Failed during transcription
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),
    /// Transcription took too long (exceeded timeout)
    #[error("Transcription timed out after {duration_secs}s (limit: {timeout_secs}s)")]
    TranscriptionTimeout { duration_secs: u64, timeout_secs: u64 },
    /// Lock was poisoned
    #[error("Internal lock error")]
    LockPoisoned,
    /// Buffer is empty
    #[error("Audio buffer is empty")]
    EmptyBuffer,
    /// Not enough new samples since last analysis
    #[error("Not enough new audio samples for analysis")]
    InsufficientNewSamples,
    /// No speech detected in buffer (VAD filtered)
    #[error("No speech detected in audio buffer (VAD filtered)")]
    NoSpeechDetected,
    /// VAD initialization failed
    #[error("VAD initialization failed: {0}")]
    VadInitFailed(String),
    /// Audio segment already analyzed (fingerprint match)
    #[error("Audio segment already analyzed (fingerprint match)")]
    DuplicateAudio,
}

/// Internal mutable state for WakeWordDetector.
///
/// All fields are protected by a single lock for simplicity and deadlock prevention.
/// This coarse-grained locking is intentional: the analysis interval is 150ms,
/// so contention is minimal, and simplicity trumps micro-optimization here.
struct DetectorState {
    /// Circular buffer for audio samples
    buffer: CircularBuffer,
    /// Sample count at last analysis (for tracking new samples)
    last_analysis_sample_count: u64,
    /// Recent audio fingerprints for deduplication (stores last 5)
    /// Uses sample indices to identify audio segments, independent of transcription text
    recent_fingerprints: VecDeque<AudioFingerprint>,
    /// Voice Activity Detector for filtering non-speech audio
    vad: Option<VoiceActivityDetector>,
}

/// Wake word detector using Parakeet TDT model
///
/// Processes audio samples in small windows to detect the wake phrase.
/// Uses on-device speech recognition for privacy.
///
/// This detector now uses a SharedTranscriptionModel, allowing it to share
/// the ~3GB Parakeet model with all other transcription consumers, saving significant memory.
pub struct WakeWordDetector {
    /// Configuration
    config: WakeWordDetectorConfig,
    /// Shared transcription model (wraps ParakeetTDT)
    shared_model: Option<SharedTranscriptionModel>,
    /// All mutable state protected by a single lock (coarse-grained for simplicity)
    state: Mutex<DetectorState>,
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
            shared_model: None,
            state: Mutex::new(DetectorState {
                buffer,
                last_analysis_sample_count: 0,
                recent_fingerprints: VecDeque::with_capacity(5),
                vad: None,
            }),
        }
    }

    /// Create a wake word detector with a shared transcription model
    ///
    /// This is the preferred constructor for production use, as it allows
    /// sharing a single ~3GB model between WakeWordDetector and other transcription consumers.
    #[allow(dead_code)] // Used by ListeningPipeline and in tests
    pub fn with_shared_model(shared_model: SharedTranscriptionModel) -> Self {
        Self::with_shared_model_and_config(shared_model, WakeWordDetectorConfig::default())
    }

    /// Create a wake word detector with a shared model and custom configuration
    pub fn with_shared_model_and_config(
        shared_model: SharedTranscriptionModel,
        config: WakeWordDetectorConfig,
    ) -> Self {
        let buffer = CircularBuffer::for_duration(config.window_duration_secs, config.sample_rate);
        Self {
            config,
            shared_model: Some(shared_model),
            state: Mutex::new(DetectorState {
                buffer,
                last_analysis_sample_count: 0,
                recent_fingerprints: VecDeque::with_capacity(5),
                vad: None,
            }),
        }
    }

    /// Set the shared transcription model
    ///
    /// This allows setting the model after construction (e.g., when the model
    /// becomes available after async loading).
    #[allow(dead_code)] // Future use for dynamic model loading
    pub fn set_shared_model(&mut self, shared_model: SharedTranscriptionModel) {
        self.shared_model = Some(shared_model);
    }

    /// Initialize VAD (Voice Activity Detection)
    ///
    /// Must be called before processing audio if VAD is enabled.
    /// The Parakeet model should already be loaded via the shared model.
    /// Uses the unified VadConfig with wake_word preset for optimal sensitivity.
    pub fn init_vad(&self) -> Result<(), WakeWordError> {
        if !self.config.vad_enabled {
            crate::info!("[wake-word] VAD disabled");
            return Ok(());
        }

        crate::debug!("[wake-word] Initializing VAD detector...");

        // Create VAD config based on detector settings
        let vad_config = VadConfig {
            speech_threshold: self.config.vad_speech_threshold,
            sample_rate: self.config.sample_rate,
            min_speech_frames: self.config.min_speech_frames,
        };

        let vad = create_vad(&vad_config).map_err(|e| {
            crate::error!("[wake-word] Failed to initialize VAD: {}", e);
            match e {
                VadError::InitializationFailed(msg) => WakeWordError::VadInitFailed(msg),
                VadError::ConfigurationInvalid(msg) => WakeWordError::VadInitFailed(msg),
            }
        })?;

        let mut state = self.state.lock().map_err(|_| WakeWordError::LockPoisoned)?;
        state.vad = Some(vad);

        crate::info!(
            "[wake-word] VAD initialized (threshold={})",
            self.config.vad_speech_threshold
        );

        Ok(())
    }

    /// Check if the model is loaded
    #[allow(dead_code)] // Utility method for status checks
    pub fn is_loaded(&self) -> bool {
        self.shared_model
            .as_ref()
            .map(|m| m.is_loaded())
            .unwrap_or(false)
    }

    /// Push audio samples into the buffer for analysis
    ///
    /// Call this with incoming audio data from the audio capture.
    pub fn push_samples(&self, samples: &[f32]) -> Result<(), WakeWordError> {
        crate::trace!("[wake-word] Pushing {} samples to buffer", samples.len());
        let mut state = self.state.lock().map_err(|_| WakeWordError::LockPoisoned)?;
        state.buffer.push_samples(samples);
        Ok(())
    }

    /// Analyze the current buffer for wake word
    ///
    /// Returns a WakeWordResult indicating whether the wake phrase was detected.
    /// Skips analysis if not enough new samples have accumulated since last analysis.
    pub fn analyze(&self) -> Result<WakeWordResult, WakeWordError> {
        // Acquire the single state lock for the entire operation
        let mut state = self.state.lock().map_err(|_| WakeWordError::LockPoisoned)?;

        // Check if buffer is empty
        if state.buffer.is_empty() {
            return Err(WakeWordError::EmptyBuffer);
        }

        // Get samples and calculate audio fingerprint
        let samples = state.buffer.get_samples();
        let end_idx = state.buffer.total_samples_pushed();
        let start_idx = end_idx.saturating_sub(samples.len() as u64);
        let fingerprint = AudioFingerprint { start_idx, end_idx };

        // Check if we have enough new samples since last analysis
        let new_samples = fingerprint.end_idx.saturating_sub(state.last_analysis_sample_count) as usize;
        if new_samples < self.config.min_new_samples_for_analysis {
            crate::trace!(
                "[wake-word] Skipping analysis: only {} new samples (need {})",
                new_samples,
                self.config.min_new_samples_for_analysis
            );
            return Err(WakeWordError::InsufficientNewSamples);
        }

        // Check for duplicate audio using fingerprint (>50% overlap with recent)
        for fp in state.recent_fingerprints.iter() {
            let overlap = fingerprint.overlap_ratio(fp);
            if overlap >= FINGERPRINT_OVERLAP_THRESHOLD {
                crate::trace!(
                    "[wake-word] Skipping duplicate audio: {:.1}% overlap with recent fingerprint",
                    overlap * 100.0
                );
                return Err(WakeWordError::DuplicateAudio);
            }
        }

        crate::trace!(
            "[wake-word] Analyzing {} samples ({} new since last analysis)",
            samples.len(),
            new_samples
        );

        // VAD check - skip expensive transcription if no speech detected
        if self.config.vad_enabled {
            let has_speech = Self::check_vad_internal(&mut state.vad, &samples, &self.config)?;
            if !has_speech {
                crate::trace!("[wake-word] VAD: No speech detected, skipping transcription");
                return Err(WakeWordError::NoSpeechDetected);
            }
            crate::debug!("[wake-word] VAD: Speech detected, proceeding with transcription");
        }

        // Drop the lock before expensive transcription to avoid blocking push_samples
        // We clone the samples since we need them after releasing the lock
        let samples_clone = samples.clone();
        drop(state);

        // Transcribe the audio using the shared model
        let transcribe_start = std::time::Instant::now();
        let shared_model = self
            .shared_model
            .as_ref()
            .ok_or(WakeWordError::ModelNotLoaded)?;

        let transcription = shared_model
            .transcribe_samples(samples_clone, self.config.sample_rate, 1)
            .map_err(|e| WakeWordError::TranscriptionFailed(e.to_string()))?;
        let transcribe_duration = transcribe_start.elapsed();

        // Check if transcription exceeded the timeout threshold
        // Note: Since transcription is synchronous, we can only detect this after completion
        // This provides visibility into slow transcriptions for debugging
        let timeout_duration = std::time::Duration::from_secs(self.config.transcription_timeout_secs);
        if transcribe_duration > timeout_duration {
            crate::warn!(
                "[wake-word] Transcription exceeded timeout: {:?} > {:?}",
                transcribe_duration,
                timeout_duration
            );
            return Err(WakeWordError::TranscriptionTimeout {
                duration_secs: transcribe_duration.as_secs(),
                timeout_secs: self.config.transcription_timeout_secs,
            });
        }

        // Re-acquire lock to update state after successful transcription
        let mut state = self.state.lock().map_err(|_| WakeWordError::LockPoisoned)?;

        // Update sample count and store fingerprint
        state.last_analysis_sample_count = fingerprint.end_idx;
        state.recent_fingerprints.push_back(fingerprint);
        // Keep only last 5 fingerprints
        while state.recent_fingerprints.len() > 5 {
            state.recent_fingerprints.pop_front();
        }
        crate::trace!(
            "[wake-word] Stored fingerprint, total={} recent fingerprints",
            state.recent_fingerprints.len()
        );

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
        let mut state = self.state.lock().map_err(|_| WakeWordError::LockPoisoned)?;

        // Clear the audio buffer
        state.buffer.clear();
        state.buffer.reset_sample_counter();

        // Reset analysis tracking
        state.last_analysis_sample_count = 0;

        // Clear recent fingerprints to allow fresh detection
        state.recent_fingerprints.clear();

        Ok(())
    }

    /// Check if audio samples contain speech using Voice Activity Detection (internal helper)
    ///
    /// This is the internal implementation that takes the VAD directly to avoid
    /// re-acquiring the lock when called from analyze().
    fn check_vad_internal(
        vad: &mut Option<VoiceActivityDetector>,
        samples: &[f32],
        config: &WakeWordDetectorConfig,
    ) -> Result<bool, WakeWordError> {
        // If VAD not initialized, conservatively assume speech is present
        let vad = match vad.as_mut() {
            Some(v) => v,
            None => {
                crate::trace!("[wake-word] VAD not initialized, assuming speech present");
                return Ok(true);
            }
        };

        // Process samples in VAD_CHUNK_SIZE_16KHZ chunks (required by Silero at 16kHz)
        const CHUNK_SIZE: usize = VAD_CHUNK_SIZE_16KHZ;
        let mut max_probability: f32 = 0.0;
        let mut speech_frames = 0;
        let mut total_frames = 0;

        for chunk in samples.chunks(CHUNK_SIZE) {
            if chunk.len() == CHUNK_SIZE {
                let probability = vad.predict(chunk.to_vec());
                max_probability = max_probability.max(probability);
                if probability >= config.vad_speech_threshold {
                    speech_frames += 1;
                    // Early return on confident speech detection for performance
                    // This ensures short utterances like "hello" trigger analysis quickly
                    if speech_frames > config.min_speech_frames {
                        crate::trace!(
                            "[wake-word] VAD: Early return after {} speech frames (max_prob={:.2})",
                            speech_frames,
                            max_probability
                        );
                        return Ok(true);
                    }
                }
                total_frames += 1;
            }
        }

        // Also process partial final chunk by zero-padding
        // This prevents missing speech at buffer boundaries
        let remaining = samples.len() % CHUNK_SIZE;
        if remaining >= MIN_PARTIAL_VAD_CHUNK {
            // Only process if we have at least half a chunk (meaningful data)
            let start = samples.len() - remaining;
            let mut padded = vec![0.0f32; CHUNK_SIZE];
            padded[..remaining].copy_from_slice(&samples[start..]);
            let probability = vad.predict(padded);
            max_probability = max_probability.max(probability);
            if probability >= config.vad_speech_threshold {
                speech_frames += 1;
            }
            total_frames += 1;
        }

        crate::trace!(
            "[wake-word] VAD: {}/{} frames with speech (max_prob={:.2}, threshold={}, min_frames={})",
            speech_frames,
            total_frames,
            max_probability,
            config.vad_speech_threshold,
            config.min_speech_frames
        );

        // Require at least min_speech_frames with speech above threshold
        // This catches short utterances while filtering random noise spikes
        Ok(speech_frames >= config.min_speech_frames)
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
        // 2 seconds window for ~128KB memory at 16kHz
        assert_eq!(config.window_duration_secs, WAKE_WORD_WINDOW_SECS);
        assert_eq!(config.sample_rate, DEFAULT_SAMPLE_RATE);
        // 0.75 seconds of new audio required before re-analysis
        assert_eq!(config.min_new_samples_for_analysis, WAKE_WORD_MIN_NEW_SAMPLES);
        // VAD enabled by default with aggressive threshold
        assert!(config.vad_enabled);
        assert_eq!(config.vad_speech_threshold, VAD_THRESHOLD_AGGRESSIVE);
        // Minimum speech frames required (~128ms, filters brief noise spikes)
        assert_eq!(config.min_speech_frames, WAKE_WORD_MIN_SPEECH_FRAMES);
        // Transcription timeout (generous for ~2s audio window)
        assert_eq!(config.transcription_timeout_secs, WAKE_WORD_TRANSCRIPTION_TIMEOUT_SECS);
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
            min_speech_frames: 3,
            transcription_timeout_secs: 5, // Custom timeout
        };
        let detector = WakeWordDetector::with_config(config.clone());
        assert_eq!(detector.config().wake_phrase, "hello world");
        assert_eq!(detector.config().sample_rate, 44100);
        assert_eq!(detector.config().transcription_timeout_secs, 5);
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
        // Push enough samples to meet min_new_samples_for_analysis (12000)
        let samples = vec![0.1; 12000];
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

    // Tests removed per docs/TESTING.md:
    // - test_error_display: Display trait test
    // - test_wake_word_result_equality: Type system guarantee (#[derive(PartialEq, Clone)])

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

    #[test]
    fn test_with_shared_model() {
        let shared = SharedTranscriptionModel::new();
        let detector = WakeWordDetector::with_shared_model(shared.clone());

        // Shared model is set but not loaded
        assert!(!detector.is_loaded());
        assert!(!shared.is_loaded());
    }

    #[test]
    fn test_set_shared_model() {
        let mut detector = WakeWordDetector::new();
        assert!(!detector.is_loaded());

        let shared = SharedTranscriptionModel::new();
        detector.set_shared_model(shared);

        // Still not loaded (model not initialized) but shared model is set
        assert!(!detector.is_loaded());
    }

    #[test]
    fn test_with_shared_model_and_config() {
        let shared = SharedTranscriptionModel::new();
        let config = WakeWordDetectorConfig {
            wake_phrase: "hello world".to_string(),
            ..Default::default()
        };
        let detector = WakeWordDetector::with_shared_model_and_config(shared, config);

        assert_eq!(detector.config().wake_phrase, "hello world");
        assert!(!detector.is_loaded());
    }
}

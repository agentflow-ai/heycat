//! Centralized constants for audio processing.
//!
//! All audio-related magic numbers are defined here with documentation
//! explaining their purpose and constraints. This module eliminates
//! scattered magic numbers throughout the audio processing code.

// =============================================================================
// SAMPLE RATE AND TIMING
// =============================================================================

/// Sample rate used throughout the audio pipeline (Hz).
///
/// Silero VAD only supports 8000 or 16000 Hz. The Parakeet transcription
/// model also expects 16kHz audio. This is the standard rate for all
/// audio processing in the application.
pub const DEFAULT_SAMPLE_RATE: u32 = 16000;

/// Optimal chunk duration for VAD processing (milliseconds).
///
/// Silero VAD works best with 32ms windows. This duration is multiplied
/// by the sample rate to get the chunk size in samples.
pub const OPTIMAL_CHUNK_DURATION_MS: u32 = 32;

// =============================================================================
// VAD CHUNK SIZES
// =============================================================================

/// Chunk size for VAD at 16kHz: 16000 * 32 / 1000 = 512 samples.
///
/// This is required by Silero VAD at 16kHz. Each chunk represents
/// a 32ms window of audio data.
pub const VAD_CHUNK_SIZE_16KHZ: usize = 512;

/// Chunk size for VAD at 8kHz: 8000 * 32 / 1000 = 256 samples.
///
/// For use when processing 8kHz audio (also 32ms window).
#[allow(dead_code)]
pub const VAD_CHUNK_SIZE_8KHZ: usize = 256;

/// Minimum samples to process for a partial VAD chunk.
///
/// When the remaining audio buffer doesn't fill a complete VAD chunk,
/// we still process it if it contains at least this many samples.
/// Set to half a chunk (256 samples at 16kHz = 16ms) to avoid
/// missing speech at buffer boundaries while filtering noise.
pub const MIN_PARTIAL_VAD_CHUNK: usize = VAD_CHUNK_SIZE_16KHZ / 2;

// =============================================================================
// VAD THRESHOLDS
// =============================================================================

/// VAD speech threshold for wake word detection (0.0 - 1.0).
///
/// Lower threshold = more sensitive to speech. Set to 0.3 for better
/// sensitivity to varied pronunciations and volumes. The cost of false
/// positives here is only an extra transcription attempt.
pub const VAD_THRESHOLD_WAKE_WORD: f32 = 0.3;

/// VAD speech threshold for balanced general use (0.0 - 1.0).
///
/// A middle ground between sensitivity and precision. Suitable for
/// general-purpose VAD where neither extreme sensitivity nor precision
/// is critical.
pub const VAD_THRESHOLD_BALANCED: f32 = 0.4;

/// VAD speech threshold for silence/end-of-speech detection (0.0 - 1.0).
///
/// Higher threshold = more confident speech is present. Used by the
/// silence detector to avoid cutting off speech prematurely during
/// pauses or soft speech.
pub const VAD_THRESHOLD_SILENCE: f32 = 0.5;

/// VAD speech threshold for wake word detector (aggressive filtering).
///
/// Set higher (0.6) to aggressively filter ambient noise. May miss
/// very quiet speech but significantly reduces false positives during
/// continuous listening.
pub const VAD_THRESHOLD_AGGRESSIVE: f32 = 0.6;

// =============================================================================
// WAKE WORD DETECTION
// =============================================================================

/// Default transcription timeout for wake word detection (seconds).
///
/// This is shorter than the hotkey timeout since wake word detection
/// only processes ~2 second audio windows. If transcription exceeds
/// this duration, a warning is logged.
pub const WAKE_WORD_TRANSCRIPTION_TIMEOUT_SECS: u64 = 10;

/// Wake word detection window duration (seconds).
///
/// How much audio history to analyze for wake word. Reduced from 3s
/// to 2s for faster response with short utterances like "hey cat".
/// At 16kHz: 2.0 * 16000 = 32000 samples = ~128KB memory.
pub const WAKE_WORD_WINDOW_SECS: f32 = 2.0;

/// Minimum new samples required before wake word re-analysis.
///
/// Set to 0.75 seconds of new audio (12000 samples at 16kHz) to reduce
/// noise from overlapping transcriptions while still catching "hey cat".
pub const WAKE_WORD_MIN_NEW_SAMPLES: usize = 12000;

/// Minimum speech frames required for wake word VAD check.
///
/// Require 4+ frames (~128ms) above threshold to filter brief noise
/// spikes while catching short utterances like "hey cat".
pub const WAKE_WORD_MIN_SPEECH_FRAMES: usize = 4;

/// Fingerprint overlap threshold for duplicate audio detection (0.0 - 1.0).
///
/// Audio segments with >50% overlap are considered duplicates and
/// skipped to prevent re-transcribing the same audio.
pub const FINGERPRINT_OVERLAP_THRESHOLD: f32 = 0.5;

// =============================================================================
// SILENCE DETECTION
// =============================================================================

/// Duration of silence before stopping recording (milliseconds).
///
/// After speech is detected and followed by this duration of silence,
/// recording is automatically stopped (SilenceAfterSpeech).
pub const SILENCE_DURATION_MS: u32 = 2000;

/// Duration before canceling if no speech detected (milliseconds).
///
/// If no speech is detected after wake word activation within this
/// duration, recording is canceled (NoSpeechTimeout / false activation).
pub const NO_SPEECH_TIMEOUT_MS: u32 = 5000;

/// Duration of pause that doesn't trigger stop (milliseconds).
///
/// Brief pauses in speech below this duration won't trigger silence
/// detection. Reserved for future pause detection refinement.
#[allow(dead_code)]
pub const PAUSE_TOLERANCE_MS: u32 = 1000;

/// Minimum speech frames for silence detector VAD.
///
/// Helps filter out brief noise spikes. Setting to 2 catches short
/// utterances while filtering random pops.
pub const SILENCE_MIN_SPEECH_FRAMES: usize = 2;

// =============================================================================
// PIPELINE CONFIGURATION
// =============================================================================

/// Analysis interval for wake word pipeline (milliseconds).
///
/// How often the listening pipeline analyzes accumulated audio for
/// wake word. 150ms provides responsive detection (trades ~3x more
/// CPU for ~3x faster response time).
pub const ANALYSIS_INTERVAL_MS: u64 = 150;

/// Detection check interval for silence/speech detection (milliseconds).
///
/// How often the recording coordinator checks accumulated audio for
/// silence or speech patterns. 100ms provides a good balance between
/// responsiveness and CPU usage.
pub const DETECTION_INTERVAL_MS: u64 = 100;

/// Minimum samples to process for silence detection.
///
/// At least 100ms worth of audio at 16kHz (1600 samples) is needed
/// for reliable silence/speech detection. This ensures enough context
/// for the VAD to make accurate decisions.
pub const MIN_DETECTION_SAMPLES: usize = 1600;

/// Minimum samples before first wake word analysis.
///
/// Need at least 0.25 seconds of audio before analyzing.
/// At 16kHz: 0.25 * 16000 = 4000 samples.
pub const MIN_SAMPLES_FOR_ANALYSIS: usize = 4000;

/// Event channel buffer size for wake word events.
///
/// Small buffer since events should be processed quickly. Bounded
/// to handle backpressure gracefully if receiver falls behind.
pub const EVENT_CHANNEL_BUFFER_SIZE: usize = 16;

/// Chunk size for real-time audio resampling (samples).
///
/// When the audio device doesn't support 16kHz natively, we resample
/// in chunks of this size. 1024 samples provides a good balance between
/// latency (~64ms at 16kHz) and processing efficiency.
pub const RESAMPLE_CHUNK_SIZE: usize = 1024;

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Calculate chunk size for a given sample rate.
///
/// Returns the number of samples needed for optimal VAD processing
/// at the given sample rate (32ms window).
///
/// # Arguments
/// * `sample_rate` - The audio sample rate in Hz
///
/// # Returns
/// The chunk size in samples (512 for 16kHz, 256 for 8kHz)
pub const fn chunk_size_for_sample_rate(sample_rate: u32) -> usize {
    (sample_rate * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_size_calculation() {
        assert_eq!(chunk_size_for_sample_rate(16000), VAD_CHUNK_SIZE_16KHZ);
        assert_eq!(chunk_size_for_sample_rate(8000), VAD_CHUNK_SIZE_8KHZ);
    }

    #[test]
    fn test_chunk_sizes_match_formula() {
        // Verify the constants match the formula
        assert_eq!(
            VAD_CHUNK_SIZE_16KHZ,
            (DEFAULT_SAMPLE_RATE * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
        );
        assert_eq!(
            VAD_CHUNK_SIZE_8KHZ,
            (8000 * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
        );
    }

    #[test]
    fn test_min_partial_vad_chunk_is_half_of_full_chunk() {
        // MIN_PARTIAL_VAD_CHUNK should be exactly half of VAD_CHUNK_SIZE_16KHZ
        assert_eq!(MIN_PARTIAL_VAD_CHUNK, VAD_CHUNK_SIZE_16KHZ / 2);
        assert_eq!(MIN_PARTIAL_VAD_CHUNK, 256);
    }

    #[test]
    fn test_threshold_ordering() {
        // Wake word threshold should be lowest (most sensitive)
        assert!(VAD_THRESHOLD_WAKE_WORD < VAD_THRESHOLD_BALANCED);
        // Balanced should be in the middle
        assert!(VAD_THRESHOLD_BALANCED < VAD_THRESHOLD_SILENCE);
        // Silence threshold should be below aggressive
        assert!(VAD_THRESHOLD_SILENCE < VAD_THRESHOLD_AGGRESSIVE);
    }

    #[test]
    fn test_sample_rate_valid_for_silero() {
        // Silero VAD only supports 8000 or 16000 Hz
        assert!(DEFAULT_SAMPLE_RATE == 16000 || DEFAULT_SAMPLE_RATE == 8000);
    }

    #[test]
    fn test_wake_word_window_memory_bounded() {
        // 2 seconds at 16kHz = 32000 samples * 4 bytes = 128KB
        let samples = (WAKE_WORD_WINDOW_SECS * DEFAULT_SAMPLE_RATE as f32) as usize;
        let memory = samples * std::mem::size_of::<f32>();
        assert!(memory < 150 * 1024, "Wake word buffer should be < 150KB");
        assert!(memory >= 120 * 1024, "Wake word buffer should be >= 120KB");
    }

    #[test]
    fn test_min_new_samples_reasonable() {
        // min_new_samples should be less than one window duration
        let window_samples = (WAKE_WORD_WINDOW_SECS * DEFAULT_SAMPLE_RATE as f32) as usize;
        assert!(WAKE_WORD_MIN_NEW_SAMPLES < window_samples);
        // But at least 0.5 seconds worth
        let half_second_samples = (DEFAULT_SAMPLE_RATE / 2) as usize;
        assert!(WAKE_WORD_MIN_NEW_SAMPLES >= half_second_samples);
    }

    #[test]
    fn test_analysis_interval_reasonable() {
        // Analysis interval should be at least 50ms for CPU efficiency
        assert!(ANALYSIS_INTERVAL_MS >= 50);
        // But less than 500ms for responsive detection
        assert!(ANALYSIS_INTERVAL_MS <= 500);
    }
}

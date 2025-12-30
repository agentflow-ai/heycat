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

/// Chunk size for real-time audio resampling (samples).
///
/// When the audio device doesn't support 16kHz natively, we resample
/// in chunks of this size. 1024 samples provides a good balance between
/// latency (~64ms at 16kHz) and processing efficiency.
#[allow(dead_code)]
pub const RESAMPLE_CHUNK_SIZE: usize = 1024;

// =============================================================================
// BUFFER SIZE CONFIGURATION
// =============================================================================

/// Preferred audio buffer size for consistent timing (samples).
///
/// 256 samples = ~16ms at 16kHz, ~5ms at 48kHz.
/// Smaller values reduce latency but increase CPU usage.
/// This is used to configure the audio buffer size for processing,
/// reducing glitches caused by variable platform defaults.
///
/// | Buffer Size | Latency @ 16kHz | Latency @ 48kHz | Notes |
/// |-------------|-----------------|-----------------|-------|
/// | 128         | 8ms             | 2.7ms           | Very low latency, higher CPU |
/// | 256         | 16ms            | 5.3ms           | Good balance (recommended) |
/// | 512         | 32ms            | 10.7ms          | Lower CPU, higher latency |
#[allow(dead_code)]
pub const PREFERRED_BUFFER_SIZE: u32 = 256;

// =============================================================================
// AUDIO PREPROCESSING
// =============================================================================

/// Highpass filter cutoff frequency (Hz).
///
/// Removes low-frequency rumble (HVAC, traffic, handling noise) below this
/// frequency. 80Hz is above typical voice fundamentals (85-255Hz) but removes
/// room noise effectively.
#[allow(dead_code)]
pub const HIGHPASS_CUTOFF_HZ: f32 = 80.0;

/// Pre-emphasis filter coefficient.
///
/// Standard ASR coefficient that boosts frequencies above ~300Hz to improve
/// speech intelligibility. The filter is: y[n] = x[n] - alpha * x[n-1]
/// Higher values (closer to 1.0) = stronger high-frequency boost.
#[allow(dead_code)]
pub const PRE_EMPHASIS_ALPHA: f32 = 0.97;

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
#[path = "audio_constants_test.rs"]
mod tests;

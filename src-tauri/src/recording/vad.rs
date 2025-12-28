// Voice Activity Detection (VAD) configuration
// Used by silence detection during recording

use crate::audio_constants::{
    chunk_size_for_sample_rate, DEFAULT_SAMPLE_RATE, VAD_THRESHOLD_SILENCE,
};
use voice_activity_detector::VoiceActivityDetector;

/// Error type for VAD operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VadError {
    /// VAD initialization failed
    #[error("VAD initialization failed: {0}")]
    InitializationFailed(String),
    /// Invalid configuration (e.g., unsupported sample rate)
    #[error("VAD configuration invalid: {0}")]
    ConfigurationInvalid(String),
}

/// VAD configuration for silence detection.
///
/// # Threshold Rationale
///
/// Silence detection uses a threshold of 0.5 for precision.
/// The silence detector must avoid cutting off speech prematurely.
/// A higher threshold ensures we only stop recording when true silence
/// is detected, not during brief pauses or soft speech.
///
/// The Silero VAD model outputs speech probability 0.0-1.0:
/// - Values below 0.3 are typically background noise
/// - Values 0.3-0.5 may be soft speech or ambiguous audio
/// - Values above 0.5 are confident speech detection
///
/// Note: `speech_threshold` and `min_speech_frames` are not used by `create_vad()`
/// (Silero VAD doesn't accept thresholds at init time). They exist for documentation
/// and future use if consumers want to extract threshold config from a unified source.
#[derive(Debug, Clone)]
pub struct VadConfig {
    /// Speech probability threshold (0.0-1.0)
    ///
    /// Audio frames with probability above this value are considered speech.
    /// See struct docs for threshold rationale.
    #[allow(dead_code)]
    pub speech_threshold: f32,

    /// Audio sample rate in Hz
    ///
    /// Must match the audio input source. The Silero VAD model only supports
    /// 8000 or 16000 Hz. The chunk size is automatically derived from this
    /// value (32ms window = 256 samples at 8kHz, 512 at 16kHz).
    pub sample_rate: u32,

    /// Minimum speech frames before considering speech detected
    ///
    /// Helps filter out brief noise spikes. Setting to 2 catches
    /// short utterances like "hello" while filtering random pops.
    #[allow(dead_code)]
    pub min_speech_frames: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            // Silence detection threshold for precision
            speech_threshold: VAD_THRESHOLD_SILENCE,
            sample_rate: DEFAULT_SAMPLE_RATE,
            min_speech_frames: 2,
        }
    }
}

impl VadConfig {
    /// Configuration preset for silence detection
    ///
    /// Uses a threshold of 0.5 to avoid cutting off speech
    /// during pauses. Precision is more important than sensitivity
    /// when deciding to stop recording.
    #[allow(dead_code)]
    pub fn silence() -> Self {
        Self {
            speech_threshold: VAD_THRESHOLD_SILENCE,
            ..Default::default()
        }
    }

    /// Create config with custom threshold
    #[allow(dead_code)]
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            speech_threshold: threshold,
            ..Default::default()
        }
    }
}

/// Factory function for creating VAD detector
///
/// Initializes a Silero VAD model with the given configuration.
///
/// # Errors
///
/// Returns `VadError::ConfigurationInvalid` if the sample rate is not 8000 or 16000 Hz.
/// Returns `VadError::InitializationFailed` if the VAD model fails to load.
pub fn create_vad(config: &VadConfig) -> Result<VoiceActivityDetector, VadError> {
    // Validate sample rate - Silero VAD only supports 8kHz and 16kHz
    match config.sample_rate {
        8000 | 16000 => {}
        other => {
            return Err(VadError::ConfigurationInvalid(format!(
                "Unsupported sample rate: {} Hz. Must be 8000 or 16000 Hz.",
                other
            )))
        }
    }

    // Calculate chunk size from sample rate (32ms window)
    let chunk_size = chunk_size_for_sample_rate(config.sample_rate);

    VoiceActivityDetector::builder()
        .sample_rate(config.sample_rate as i32)
        .chunk_size(chunk_size)
        .build()
        .map_err(|e| VadError::InitializationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio_constants::{VAD_CHUNK_SIZE_16KHZ, VAD_CHUNK_SIZE_8KHZ};

    #[test]
    fn test_config_presets_have_distinct_thresholds() {
        let silence = VadConfig::silence();
        let custom = VadConfig::with_threshold(0.6);

        // Silence detection threshold
        assert_eq!(silence.speech_threshold, VAD_THRESHOLD_SILENCE);
        // Custom threshold works
        assert_eq!(custom.speech_threshold, 0.6);
    }

    #[test]
    fn test_create_vad_with_valid_sample_rates() {
        // 8kHz and 16kHz are the only supported rates
        let config_8k = VadConfig { sample_rate: 8000, ..Default::default() };
        let config_16k = VadConfig { sample_rate: 16000, ..Default::default() };

        assert!(create_vad(&config_8k).is_ok());
        assert!(create_vad(&config_16k).is_ok());

        // Verify chunk sizes match expected values
        assert_eq!(chunk_size_for_sample_rate(8000), VAD_CHUNK_SIZE_8KHZ);
        assert_eq!(chunk_size_for_sample_rate(16000), VAD_CHUNK_SIZE_16KHZ);
    }

    #[test]
    fn test_create_vad_rejects_unsupported_sample_rates() {
        let invalid_rates = [0, 22050, 44100, 48000];

        for rate in invalid_rates {
            let config = VadConfig { sample_rate: rate, ..Default::default() };
            let result = create_vad(&config);
            assert!(result.is_err(), "Should reject {} Hz", rate);
            assert!(matches!(result.unwrap_err(), VadError::ConfigurationInvalid(_)));
        }
    }

    #[test]
    fn test_sample_rate_error_message_mentions_supported_rates() {
        let config = VadConfig { sample_rate: 22050, ..Default::default() };
        let err = create_vad(&config).unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("8000") && msg.contains("16000"));
    }
}

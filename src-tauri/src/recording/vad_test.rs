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

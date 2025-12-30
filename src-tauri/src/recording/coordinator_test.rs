use super::*;

#[test]
fn test_recording_detectors_with_config() {
    let silence_config = SilenceConfig {
        silence_duration_ms: 1000,
        ..Default::default()
    };
    let detectors = RecordingDetectors::with_config(silence_config);
    assert!(!detectors.is_running());
}

#[test]
fn test_stop_without_start() {
    let mut detectors = RecordingDetectors::new();
    // Should not panic
    detectors.stop_monitoring();
    assert!(!detectors.is_running());
}

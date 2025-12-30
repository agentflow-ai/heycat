use super::*;
use std::thread;
use std::time::Duration;

#[test]
fn test_reset_clears_state() {
    let mut detector = SilenceDetector::new();
    detector.has_detected_speech = true;
    detector.silence_start = Some(Instant::now());

    detector.reset();
    assert!(!detector.has_detected_speech());
    assert!(detector.silence_start.is_none());
}

#[test]
fn test_no_speech_timeout() {
    let config = SilenceConfig {
        no_speech_timeout_ms: 50,
        ..Default::default()
    };
    let mut detector = SilenceDetector::with_config(config);
    let silent_samples = vec![0.0; 512];

    thread::sleep(Duration::from_millis(60));
    let result = detector.process_samples(&silent_samples);

    assert_eq!(result, SilenceDetectionResult::Stop(SilenceStopReason::NoSpeechTimeout));
}

#[test]
fn test_silence_after_speech_state_machine() {
    let config = SilenceConfig {
        silence_duration_ms: 50,
        ..Default::default()
    };
    let mut detector = SilenceDetector::with_config(config);
    let silent_samples = vec![0.0; 512];

    // Simulate speech was detected
    detector.has_detected_speech = true;

    // Start silence tracking
    let _ = detector.process_samples(&silent_samples);

    // Wait and verify silence detected
    thread::sleep(Duration::from_millis(60));
    let result = detector.process_samples(&silent_samples);

    assert_eq!(result, SilenceDetectionResult::Stop(SilenceStopReason::SilenceAfterSpeech));
}

#[test]
fn test_continues_while_waiting() {
    let mut detector = SilenceDetector::new();
    let silent_samples = vec![0.0; 512];

    let result = detector.process_samples(&silent_samples);
    assert_eq!(result, SilenceDetectionResult::Continue);
    assert!(!detector.has_detected_speech());
}

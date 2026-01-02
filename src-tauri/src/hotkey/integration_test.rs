// Silence detection integration tests for hotkey-to-recording
//
// Other tests have been split into focused test files:
// - toggle_handler_test.rs: Toggle mode (press once to start, again to stop)
// - ptt_handler_test.rs: Push-to-talk mode (hold to record, release to stop)
// - cancel_handler_test.rs: Recording cancellation
// - escape/listener_test.rs: Escape key listener registration/unregistration

use super::integration::HotkeyIntegration;
use crate::recording::{RecordingManager, RecordingState};
use crate::test_utils::{ensure_test_model_files, MockEmitter};
use std::sync::{Arc, Mutex};

/// Type alias for HotkeyIntegration with MockEmitter for all parameters
type TestIntegration = HotkeyIntegration<MockEmitter, MockEmitter, MockEmitter>;

// === Silence Detection Integration Tests ===

#[test]
fn test_with_recording_detectors_builder() {
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));

    let integration: TestIntegration =
        HotkeyIntegration::new(emitter).with_recording_detectors(detectors.clone());

    // Just verify the builder works and doesn't panic
    assert!(integration.silence.enabled);
}

#[test]
fn test_silence_detection_can_be_disabled() {
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));

    let integration: TestIntegration = HotkeyIntegration::new(emitter)
        .with_recording_detectors(detectors)
        .with_silence_detection_enabled(false);

    assert!(!integration.silence.enabled);
}

#[test]
fn test_custom_silence_config() {
    use crate::recording::{RecordingDetectors, SilenceConfig};

    let emitter = MockEmitter::new();
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));
    let config = SilenceConfig {
        silence_duration_ms: 3000,
        no_speech_timeout_ms: 10000,
        ..Default::default()
    };

    let integration: TestIntegration = HotkeyIntegration::new(emitter)
        .with_recording_detectors(detectors)
        .with_silence_config(config);

    // Custom config should be stored
    assert!(integration.silence.config.is_some());
    assert_eq!(
        integration.silence.config.as_ref().unwrap().silence_duration_ms,
        3000
    );
}

/// Test that manual stop takes precedence over silence detection
/// Ignored by default as it requires microphone permissions
/// Run manually with: cargo test test_manual_stop_takes_precedence_over_silence_detection -- --ignored
#[test]
#[ignore]
fn test_manual_stop_takes_precedence_over_silence_detection() {
    // When user manually stops via hotkey, silence detection should be stopped
    // and not interfere with the manual stop
    ensure_test_model_files();
    use crate::audio::AudioThreadHandle;
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let audio_thread = Arc::new(AudioThreadHandle::spawn());
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));
    let recording_state = Arc::new(Mutex::new(RecordingManager::new()));

    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_audio_thread(audio_thread)
        .with_recording_detectors(detectors.clone())
        .with_recording_state(recording_state.clone());

    // Start recording
    integration.handle_toggle(&recording_state);
    assert_eq!(
        recording_state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Note: Silence detection won't actually start because we don't have
    // transcription_emitter or app_handle configured - that's fine for this test

    // Manual stop
    integration.handle_toggle(&recording_state);
    assert_eq!(
        recording_state.lock().unwrap().get_state(),
        RecordingState::Idle
    );

    // Detectors should not be running after manual stop
    let det = detectors.lock().unwrap();
    assert!(
        !det.is_running(),
        "Detectors should be stopped after manual stop"
    );
}

#[test]
fn test_silence_detection_not_started_without_detectors() {
    // Without recording detectors configured, silence detection should be skipped
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording - should work fine without detectors
    let accepted = integration.handle_toggle(&state);
    assert!(accepted);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Stop - should also work
    let accepted = integration.handle_toggle(&state);
    assert!(accepted);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
}

#[test]
fn test_silence_detection_respects_enabled_flag() {
    // When silence_detection_enabled is false, detectors should not be started
    ensure_test_model_files();
    use crate::audio::AudioThreadHandle;
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let audio_thread = Arc::new(AudioThreadHandle::spawn());
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));
    let recording_state = Arc::new(Mutex::new(RecordingManager::new()));

    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_audio_thread(audio_thread)
        .with_recording_detectors(detectors.clone())
        .with_recording_state(recording_state.clone())
        .with_silence_detection_enabled(false);

    // Start recording
    integration.handle_toggle(&recording_state);

    // Detectors should NOT be running because feature is disabled
    let det = detectors.lock().unwrap();
    assert!(
        !det.is_running(),
        "Detectors should not start when disabled"
    );
}

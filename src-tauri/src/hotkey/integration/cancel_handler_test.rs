//! Cancel recording tests for hotkey-to-recording integration.

use super::HotkeyIntegration;
use crate::recording::{RecordingManager, RecordingState};
use crate::test_utils::{ensure_test_model_files, MockEmitter, MockShortcutBackend};
use std::sync::{Arc, Mutex};

/// Type alias for HotkeyIntegration with MockEmitter for all parameters
type TestIntegration = HotkeyIntegration<MockEmitter, MockEmitter, MockEmitter>;

#[test]
fn test_cancel_recording_during_recording_clears_buffer() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);

    // Cancel recording
    let cancelled = integration.cancel_recording(&state, "double-tap-escape");

    assert!(cancelled, "Cancel should succeed");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Idle,
        "State should be Idle after cancel"
    );

    // Buffer should be cleared (no audio data retained)
    let buffer_result = state.lock().unwrap().get_audio_buffer();
    assert!(buffer_result.is_err(), "Buffer should be cleared after cancel");
}

#[test]
fn test_cancel_recording_does_not_emit_stopped_event() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(emitter.started_count(), 1);
    assert_eq!(emitter.stopped_count(), 0);
    assert_eq!(emitter.cancelled_count(), 0);

    // Cancel recording
    integration.cancel_recording(&state, "double-tap-escape");

    // Should emit cancelled, NOT stopped
    assert_eq!(
        emitter.stopped_count(),
        0,
        "Should not emit stopped event on cancel"
    );
    assert_eq!(emitter.cancelled_count(), 1, "Should emit cancelled event");
}

#[test]
fn test_cancel_recording_emits_correct_payload() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);

    // Cancel with specific reason
    integration.cancel_recording(&state, "double-tap-escape");

    let cancelled = emitter.last_cancelled().expect("Should have cancelled event");
    assert_eq!(cancelled.reason, "double-tap-escape", "Reason should match");
    assert!(!cancelled.timestamp.is_empty(), "Timestamp should be present");
    assert!(
        cancelled.timestamp.contains('T'),
        "Timestamp should be ISO 8601 format"
    );
}

#[test]
fn test_cancel_recording_ignored_when_not_recording() {
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Try to cancel from Idle state
    let cancelled = integration.cancel_recording(&state, "double-tap-escape");

    assert!(!cancelled, "Cancel should be ignored when not recording");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(
        emitter.cancelled_count(),
        0,
        "Should not emit cancelled event when not recording"
    );
}

#[test]
fn test_cancel_recording_ignored_when_processing() {
    use crate::audio::TARGET_SAMPLE_RATE;

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Manually put into Processing state
    {
        let mut manager = state.lock().unwrap();
        manager.start_recording(TARGET_SAMPLE_RATE).unwrap();
        manager.transition_to(RecordingState::Processing).unwrap();
    }

    // Try to cancel from Processing state
    let cancelled = integration.cancel_recording(&state, "double-tap-escape");

    assert!(!cancelled, "Cancel should be ignored when processing");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Processing
    );
    assert_eq!(emitter.cancelled_count(), 0);
}

#[test]
fn test_cancel_recording_unregisters_escape_listener() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_shortcut_backend(backend.clone())
        .with_escape_callback(Arc::new(move || {
            *callback_count_clone.lock().unwrap() += 1;
        }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert!(
        backend.is_registered("Escape"),
        "Escape should be registered during recording"
    );

    // Cancel recording
    integration.cancel_recording(&state, "double-tap-escape");

    // Escape should be unregistered after cancel
    assert!(
        !backend.is_registered("Escape"),
        "Escape should be unregistered after cancel"
    );
}

#[test]
fn test_cancel_recording_stops_silence_detection() {
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

    // Cancel recording
    integration.cancel_recording(&recording_state, "double-tap-escape");

    // Detectors should not be running
    let det = detectors.lock().unwrap();
    assert!(
        !det.is_running(),
        "Detectors should be stopped after cancel"
    );
}

#[test]
fn test_cancel_recording_can_restart_after_cancel() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(emitter.started_count(), 1);

    // Cancel recording
    integration.cancel_recording(&state, "double-tap-escape");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);

    // Start new recording - should work
    let accepted = integration.handle_toggle(&state);
    assert!(accepted, "Should be able to start recording after cancel");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 2);
}

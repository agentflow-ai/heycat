//! Toggle mode tests for hotkey-to-recording integration.

use super::HotkeyIntegration;
use crate::recording::{RecordingManager, RecordingState};
use crate::test_utils::{ensure_test_model_files, MockEmitter};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/// Type alias for HotkeyIntegration with MockEmitter for all parameters
type TestIntegration = HotkeyIntegration<MockEmitter, MockEmitter, MockEmitter>;

#[test]
fn test_toggle_from_idle_starts_recording() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::new(emitter.clone());
    let state = Mutex::new(RecordingManager::new());

    let accepted = integration.handle_toggle(&state);

    assert!(accepted, "Toggle should be accepted");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);
    assert_eq!(emitter.stopped_count(), 0);
}

#[test]
fn test_toggle_from_recording_stops() {
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

    // Stop recording
    let accepted = integration.handle_toggle(&state);

    assert!(accepted, "Toggle should be accepted");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.started_count(), 1);
    assert_eq!(emitter.stopped_count(), 1);
}

#[test]
fn test_rapid_toggle_debounced() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 100);
    let state = Mutex::new(RecordingManager::new());

    // First toggle accepted
    let first = integration.handle_toggle(&state);
    assert!(first, "First toggle should be accepted");

    // Immediate second toggle should be debounced
    let second = integration.handle_toggle(&state);
    assert!(!second, "Second toggle should be debounced");

    // State should still be Recording (not toggled back)
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);
    assert_eq!(emitter.stopped_count(), 0);
}

#[test]
fn test_toggle_after_debounce_window() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 50);
    let state = Mutex::new(RecordingManager::new());

    // First toggle
    integration.handle_toggle(&state);

    // Wait for debounce to expire
    thread::sleep(Duration::from_millis(60));

    // Second toggle should work
    let accepted = integration.handle_toggle(&state);
    assert!(accepted, "Toggle after debounce should be accepted");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
}

#[test]
fn test_events_emitted_on_each_toggle() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Toggle to Recording
    integration.handle_toggle(&state);
    assert_eq!(emitter.started_count(), 1);

    // Toggle to Idle
    integration.handle_toggle(&state);
    assert_eq!(emitter.stopped_count(), 1);

    // Toggle back to Recording
    integration.handle_toggle(&state);
    assert_eq!(emitter.started_count(), 2);
}

#[test]
fn test_toggle_from_processing_ignored() {
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

    // Toggle should be ignored
    let accepted = integration.handle_toggle(&state);
    assert!(!accepted, "Toggle from Processing should be ignored");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Processing
    );
}

#[test]
fn test_is_debouncing() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter, 100);
    let state = Mutex::new(RecordingManager::new());

    assert!(!integration.is_debouncing(), "Should not be debouncing initially");

    integration.handle_toggle(&state);
    assert!(integration.is_debouncing(), "Should be debouncing after toggle");

    thread::sleep(Duration::from_millis(110));
    assert!(
        !integration.is_debouncing(),
        "Should not be debouncing after window expires"
    );
}

#[test]
fn test_multiple_rapid_toggles_only_first_accepted() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 50);
    let state = Mutex::new(RecordingManager::new());

    // Rapid fire toggles
    let results: Vec<bool> = (0..5).map(|_| integration.handle_toggle(&state)).collect();

    // Only first should be accepted
    assert_eq!(results, vec![true, false, false, false, false]);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);
}

#[test]
fn test_started_payload_has_timestamp() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::new(emitter.clone());
    let state = Mutex::new(RecordingManager::new());

    integration.handle_toggle(&state);

    let started = emitter.started.lock().unwrap();
    assert!(!started[0].timestamp.is_empty());
    // Should be ISO 8601 format with T separator
    assert!(started[0].timestamp.contains('T'));
}

#[test]
fn test_stopped_payload_has_metadata() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start then stop
    integration.handle_toggle(&state);
    integration.handle_toggle(&state);

    let stopped = emitter.stopped.lock().unwrap();
    assert_eq!(stopped.len(), 1);
    // Metadata should be present (even if empty for now)
    assert!(stopped[0].metadata.duration_secs >= 0.0);
}

#[test]
fn test_toggle_without_audio_thread_still_works() {
    // Regression test: integration without audio thread should still manage state
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

    // Stop recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.stopped_count(), 1);
}

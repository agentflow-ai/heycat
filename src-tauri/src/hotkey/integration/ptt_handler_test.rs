//! Push-to-talk mode tests for hotkey-to-recording integration.

use super::HotkeyIntegration;
use crate::recording::{RecordingManager, RecordingState};
use crate::test_utils::{ensure_test_model_files, MockEmitter};
use std::sync::Mutex;

/// Type alias for HotkeyIntegration with MockEmitter for all parameters
type TestIntegration = HotkeyIntegration<MockEmitter, MockEmitter, MockEmitter>;

#[test]
fn test_recording_mode_default() {
    let emitter = MockEmitter::new();
    let integration: TestIntegration = HotkeyIntegration::new(emitter);
    assert_eq!(
        integration.recording_mode(),
        crate::hotkey::RecordingMode::Toggle
    );
}

#[test]
fn test_set_recording_mode() {
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::new(emitter);
    assert_eq!(
        integration.recording_mode(),
        crate::hotkey::RecordingMode::Toggle
    );

    integration.set_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    assert_eq!(
        integration.recording_mode(),
        crate::hotkey::RecordingMode::PushToTalk
    );

    integration.set_recording_mode(crate::hotkey::RecordingMode::Toggle);
    assert_eq!(
        integration.recording_mode(),
        crate::hotkey::RecordingMode::Toggle
    );
}

#[test]
fn test_ptt_press_starts_recording() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    integration.set_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    let state = Mutex::new(RecordingManager::new());

    // Initially Idle
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);

    // Press should start recording
    let started = integration.handle_hotkey_press(&state);
    assert!(started);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);
}

#[test]
fn test_ptt_release_stops_recording() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    integration.set_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    let state = Mutex::new(RecordingManager::new());

    // Start recording via press
    integration.handle_hotkey_press(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Release should stop recording
    let stopped = integration.handle_hotkey_release(&state);
    assert!(stopped);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.stopped_count(), 1);
}

#[test]
fn test_ptt_press_ignored_when_already_recording() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    integration.set_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    let state = Mutex::new(RecordingManager::new());

    // First press starts recording
    integration.handle_hotkey_press(&state);
    assert_eq!(emitter.started_count(), 1);

    // Second press should be ignored (already recording)
    let result = integration.handle_hotkey_press(&state);
    assert!(!result);
    assert_eq!(emitter.started_count(), 1); // Still just 1
}

#[test]
fn test_ptt_release_ignored_when_not_recording() {
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    integration.set_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    let state = Mutex::new(RecordingManager::new());

    // Release without prior press should be ignored
    let result = integration.handle_hotkey_release(&state);
    assert!(!result);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.stopped_count(), 0);
}

//! Escape key listener tests for hotkey-to-recording integration.

use crate::hotkey::integration::HotkeyIntegration;
use crate::recording::{RecordingManager, RecordingState};
use crate::test_utils::{
    ensure_test_model_files, FailingShortcutBackend, MockEmitter, MockShortcutBackend,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Type alias for HotkeyIntegration with MockEmitter for all parameters
type TestIntegration = HotkeyIntegration<MockEmitter, MockEmitter, MockEmitter>;

#[test]
fn test_escape_listener_registered_when_recording_starts() {
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

    // Before recording, Escape should not be registered
    assert!(
        !backend.is_registered("Escape"),
        "Escape should not be registered before recording"
    );

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Escape should now be registered
    assert!(
        backend.is_registered("Escape"),
        "Escape should be registered after recording starts"
    );
}

#[test]
fn test_escape_callback_fires_during_recording() {
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

    // Simulate double-tap Escape (two presses within window)
    backend.simulate_press("Escape"); // First tap
    backend.simulate_press("Escape"); // Second tap - triggers callback

    // Callback should have been invoked
    assert_eq!(
        *callback_count.lock().unwrap(),
        1,
        "Callback should fire on double-tap Escape during recording"
    );
}

#[test]
fn test_escape_listener_unregistered_when_recording_stops() {
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

    // Stop recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);

    // Escape should be unregistered
    assert!(
        !backend.is_registered("Escape"),
        "Escape should be unregistered after recording stops"
    );
}

#[test]
fn test_single_escape_tap_does_not_trigger_cancel() {
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

    // Single Escape press - should NOT trigger callback
    backend.simulate_press("Escape");

    // Callback should NOT have been invoked (single tap ignored)
    assert_eq!(
        *callback_count.lock().unwrap(),
        0,
        "Single tap should not trigger cancel"
    );
}

#[test]
fn test_escape_callback_does_not_fire_when_not_recording() {
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

    // Start and stop recording
    integration.handle_toggle(&state);
    integration.handle_toggle(&state);

    // Try to simulate Escape press (but listener is gone)
    backend.simulate_press("Escape");

    // Callback should NOT have been invoked (listener was unregistered)
    assert_eq!(
        *callback_count.lock().unwrap(),
        0,
        "Callback should not fire when not recording"
    );
}

#[test]
fn test_escape_listener_multiple_cycles() {
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

    // Cycle 1: Start -> Double-tap Escape fires -> Stop
    integration.handle_toggle(&state);
    assert!(backend.is_registered("Escape"));
    backend.simulate_press("Escape"); // First tap
    backend.simulate_press("Escape"); // Second tap - triggers
    assert_eq!(*callback_count.lock().unwrap(), 1);
    integration.handle_toggle(&state);
    assert!(!backend.is_registered("Escape"));

    // Cycle 2: Start -> Double-tap Escape fires -> Stop
    integration.handle_toggle(&state);
    assert!(backend.is_registered("Escape"));
    backend.simulate_press("Escape"); // First tap
    backend.simulate_press("Escape"); // Second tap - triggers
    assert_eq!(*callback_count.lock().unwrap(), 2);
    integration.handle_toggle(&state);
    assert!(!backend.is_registered("Escape"));

    // Cycle 3: Start -> Stop (no Escape press)
    integration.handle_toggle(&state);
    assert!(backend.is_registered("Escape"));
    integration.handle_toggle(&state);
    assert!(!backend.is_registered("Escape"));

    // Total callback count should be 2 (from the two double-taps)
    assert_eq!(*callback_count.lock().unwrap(), 2);
}

#[test]
fn test_three_rapid_escape_taps_triggers_cancel_once() {
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

    // Triple-tap Escape
    backend.simulate_press("Escape"); // First tap - records time
    backend.simulate_press("Escape"); // Second tap - triggers, resets
    backend.simulate_press("Escape"); // Third tap - starts new cycle

    // Should trigger exactly once
    assert_eq!(
        *callback_count.lock().unwrap(),
        1,
        "Triple tap should trigger cancel only once"
    );
}

#[test]
fn test_escape_double_tap_window_is_configurable() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    // Configure with a very short window (10ms)
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_shortcut_backend(backend.clone())
        .with_escape_callback(Arc::new(move || {
            *callback_count_clone.lock().unwrap() += 1;
        }))
        .with_double_tap_window(10); // 10ms window
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);

    // First tap
    backend.simulate_press("Escape");

    // Wait longer than window
    thread::sleep(Duration::from_millis(20));

    // Second tap - should NOT trigger (outside window)
    backend.simulate_press("Escape");

    // Callback should NOT have been invoked
    assert_eq!(
        *callback_count.lock().unwrap(),
        0,
        "Taps outside window should not trigger"
    );
}

#[test]
fn test_escape_listener_without_backend_gracefully_skips() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    // Only callback configured, no backend
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_escape_callback(Arc::new(move || {
            *callback_count_clone.lock().unwrap() += 1;
        }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording - should work without backend
    let accepted = integration.handle_toggle(&state);
    assert!(accepted, "Recording should start even without escape backend");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Stop recording - should work
    let accepted = integration.handle_toggle(&state);
    assert!(accepted, "Recording should stop");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
}

#[test]
fn test_escape_listener_without_callback_gracefully_skips() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());

    // Only backend configured, no callback
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_shortcut_backend(backend.clone());
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Escape should NOT be registered (no callback)
    assert!(
        !backend.is_registered("Escape"),
        "Escape should not register without callback"
    );

    // Stop recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
}

// === Escape Registration Failure Tests ===

#[test]
fn test_escape_registered_false_when_registration_fails() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(FailingShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_shortcut_backend(backend.clone())
        .with_escape_callback(Arc::new(move || {
            *callback_count_clone.lock().unwrap() += 1;
        }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording - this triggers register_escape_listener which will fail
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Recording should still work even though Escape registration failed
    assert_eq!(emitter.started_count(), 1);
}

#[test]
fn test_unregister_not_called_when_registration_failed() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(FailingShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_shortcut_backend(backend.clone())
        .with_escape_callback(Arc::new(move || {
            *callback_count_clone.lock().unwrap() += 1;
        }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording - registration will fail
    integration.handle_toggle(&state);

    // Stop recording - unregister should NOT be called since registration failed
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);

    // Verify unregister was not attempted (escape_registered stayed false)
    assert_eq!(
        backend.unregister_attempt_count(),
        0,
        "Unregister should not be called when registration failed"
    );
}

#[test]
fn test_key_blocking_unavailable_event_emitted_on_registration_failure() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(FailingShortcutBackend::new());
    let hotkey_emitter = Arc::new(emitter.clone());

    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_shortcut_backend(backend)
        .with_escape_callback(Arc::new(|| {}))
        .with_hotkey_emitter(hotkey_emitter);
    let state = Mutex::new(RecordingManager::new());

    // Start recording - this triggers register_escape_listener which will fail
    integration.handle_toggle(&state);

    // Verify the key_blocking_unavailable event was emitted
    assert_eq!(
        emitter.key_blocking_unavailable_count(),
        1,
        "key_blocking_unavailable event should be emitted when registration fails"
    );

    // Recording should still work (graceful degradation)
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);
}

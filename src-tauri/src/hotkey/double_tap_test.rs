use super::*;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn test_two_taps_within_window_triggers_cancel() {
    let triggered = Arc::new(Mutex::new(false));
    let triggered_clone = triggered.clone();

    let mut detector = DoubleTapDetector::with_window(
        move || {
            *triggered_clone.lock().unwrap() = true;
        },
        DEFAULT_DOUBLE_TAP_WINDOW_MS,
    );

    // First tap
    let result1 = detector.on_tap();
    assert!(!result1, "First tap should not trigger");
    assert!(!*triggered.lock().unwrap(), "Callback should not fire on first tap");

    // Second tap immediately (within window)
    let result2 = detector.on_tap();
    assert!(result2, "Second tap should trigger");
    assert!(*triggered.lock().unwrap(), "Callback should fire on double-tap");
}

#[test]
fn test_two_taps_outside_window_does_not_trigger() {
    let triggered = Arc::new(Mutex::new(false));
    let triggered_clone = triggered.clone();

    // Use a very short window for testing
    let mut detector = DoubleTapDetector::with_window(
        move || {
            *triggered_clone.lock().unwrap() = true;
        },
        50, // 50ms window
    );

    // First tap
    detector.on_tap();

    // Wait longer than the window
    thread::sleep(Duration::from_millis(60));

    // Second tap - should NOT trigger (outside window)
    let result = detector.on_tap();
    assert!(!result, "Tap outside window should not trigger");
    assert!(!*triggered.lock().unwrap(), "Callback should not fire");
}

#[test]
fn test_single_tap_does_not_trigger() {
    let triggered = Arc::new(Mutex::new(false));
    let triggered_clone = triggered.clone();

    let mut detector = DoubleTapDetector::with_window(
        move || {
            *triggered_clone.lock().unwrap() = true;
        },
        DEFAULT_DOUBLE_TAP_WINDOW_MS,
    );

    // Single tap only
    let result = detector.on_tap();
    assert!(!result, "Single tap should not trigger");
    assert!(!*triggered.lock().unwrap(), "Callback should not fire on single tap");
}

#[test]
fn test_three_rapid_taps_triggers_only_once() {
    let count = Arc::new(Mutex::new(0));
    let count_clone = count.clone();

    let mut detector = DoubleTapDetector::with_window(
        move || {
            *count_clone.lock().unwrap() += 1;
        },
        DEFAULT_DOUBLE_TAP_WINDOW_MS,
    );

    // Three rapid taps
    detector.on_tap(); // First tap - records time
    detector.on_tap(); // Second tap - triggers, resets
    detector.on_tap(); // Third tap - records time (starts fresh cycle)

    assert_eq!(*count.lock().unwrap(), 1, "Should trigger exactly once");
}

#[test]
fn test_time_window_is_configurable() {
    let triggered = Arc::new(Mutex::new(false));
    let triggered_clone = triggered.clone();

    // Use a longer window (500ms)
    let mut detector = DoubleTapDetector::with_window(
        move || {
            *triggered_clone.lock().unwrap() = true;
        },
        500,
    );

    assert_eq!(detector.window_ms(), 500);

    // First tap
    detector.on_tap();

    // Wait 400ms (within 500ms window)
    thread::sleep(Duration::from_millis(400));

    // Second tap - should trigger (still within window)
    let result = detector.on_tap();
    assert!(result, "Should trigger within configured window");
    assert!(*triggered.lock().unwrap(), "Callback should fire");
}

#[test]
fn test_default_window_is_300ms() {
    let detector = DoubleTapDetector::with_window(|| {}, DEFAULT_DOUBLE_TAP_WINDOW_MS);
    assert_eq!(detector.window_ms(), DEFAULT_DOUBLE_TAP_WINDOW_MS);
    assert_eq!(DEFAULT_DOUBLE_TAP_WINDOW_MS, 300);
}

#[test]
fn test_reset_clears_state() {
    let count = Arc::new(Mutex::new(0));
    let count_clone = count.clone();

    let mut detector = DoubleTapDetector::with_window(
        move || {
            *count_clone.lock().unwrap() += 1;
        },
        DEFAULT_DOUBLE_TAP_WINDOW_MS,
    );

    // First tap
    detector.on_tap();

    // Reset
    detector.reset();

    // Second tap - should NOT trigger (state was reset)
    let result = detector.on_tap();
    assert!(!result, "Tap after reset should not trigger");
    assert_eq!(*count.lock().unwrap(), 0, "Callback should not fire after reset");
}

#[test]
fn test_multiple_double_tap_cycles() {
    let count = Arc::new(Mutex::new(0));
    let count_clone = count.clone();

    let mut detector = DoubleTapDetector::with_window(
        move || {
            *count_clone.lock().unwrap() += 1;
        },
        100,
    );

    // Cycle 1: double-tap
    detector.on_tap();
    detector.on_tap();
    assert_eq!(*count.lock().unwrap(), 1);

    // Wait for window to expire
    thread::sleep(Duration::from_millis(110));

    // Cycle 2: double-tap
    detector.on_tap();
    detector.on_tap();
    assert_eq!(*count.lock().unwrap(), 2);
}

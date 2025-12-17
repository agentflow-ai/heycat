// Double-tap detection for Escape key cancellation
//
// Detects double-tap patterns within a configurable time window.
// Single taps are ignored; only double-taps trigger the cancel callback.

use std::time::{Duration, Instant};

/// Default time window for double-tap detection (300ms)
pub const DEFAULT_DOUBLE_TAP_WINDOW_MS: u64 = 300;

/// Detects double-tap patterns within a configurable time window
///
/// Usage:
/// ```ignore
/// let mut detector = DoubleTapDetector::new(|| println!("Double-tap detected!"));
/// detector.on_tap(); // First tap - nothing happens
/// detector.on_tap(); // Second tap within window - callback fires
/// ```
pub struct DoubleTapDetector<F: Fn() + Send + Sync> {
    /// Time of the last tap (None if no tap recorded)
    last_tap_time: Option<Instant>,
    /// Time window for double-tap detection
    window: Duration,
    /// Callback to invoke on double-tap
    callback: F,
}

impl<F: Fn() + Send + Sync> DoubleTapDetector<F> {
    /// Create a new DoubleTapDetector with custom window duration
    pub fn with_window(callback: F, window_ms: u64) -> Self {
        Self {
            last_tap_time: None,
            window: Duration::from_millis(window_ms),
            callback,
        }
    }

    /// Handle a tap event
    ///
    /// If this tap is within the time window of the previous tap,
    /// the callback is invoked and the state is reset.
    /// Otherwise, the tap time is recorded for the next comparison.
    ///
    /// Returns true if a double-tap was detected (callback invoked).
    pub fn on_tap(&mut self) -> bool {
        let now = Instant::now();

        if let Some(last) = self.last_tap_time {
            if now.duration_since(last) <= self.window {
                // Double-tap detected!
                (self.callback)();
                // Reset state after triggering
                self.last_tap_time = None;
                return true;
            }
        }

        // Record this tap for next comparison
        self.last_tap_time = Some(now);
        false
    }

    /// Reset the detector state
    ///
    /// Call this when the context changes (e.g., recording stopped)
    /// to prevent stale taps from affecting future detection.
    pub fn reset(&mut self) {
        self.last_tap_time = None;
    }

    /// Get the configured time window
    #[cfg(test)]
    pub fn window_ms(&self) -> u64 {
        self.window.as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
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
}

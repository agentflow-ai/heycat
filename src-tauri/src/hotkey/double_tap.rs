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
#[path = "double_tap_test.rs"]
mod tests;

// Keyboard capture module using CGEventTap for capturing all keyboard events
// This module provides low-level keyboard event capture that can detect keys that
// JavaScript's KeyboardEvent API cannot, such as the fn key on Mac keyboards,
// media keys (volume, brightness, playback), and left/right modifier distinction.
//
// Requires Accessibility permission (System Settings > Privacy & Security > Accessibility)

pub mod cgeventtap;
pub mod permissions;

// Re-export the CapturedKeyEvent from cgeventtap for backwards compatibility
// The new version has expanded fields for left/right modifiers and media keys
pub use cgeventtap::CapturedKeyEvent;

use cgeventtap::CGEventTapCapture;

/// Handle to the keyboard capture system
///
/// This is a wrapper around CGEventTapCapture that maintains the same public API
/// as the previous IOKit HID implementation.
pub struct KeyboardCapture {
    /// The underlying CGEventTap capture instance
    capture: CGEventTapCapture,
}

impl KeyboardCapture {
    /// Create a new keyboard capture instance
    pub fn new() -> Self {
        Self {
            capture: CGEventTapCapture::new(),
        }
    }

    /// Start capturing keyboard events
    ///
    /// The callback will be invoked for each key event captured.
    /// Returns an error if capture is already running or if Accessibility permission is not granted.
    ///
    /// Note: Requires Accessibility permission (not Input Monitoring).
    pub fn start<F>(&mut self, callback: F) -> Result<(), String>
    where
        F: Fn(CapturedKeyEvent) + Send + 'static,
    {
        self.capture.start(callback)
    }

    /// Stop capturing keyboard events
    pub fn stop(&mut self) -> Result<(), String> {
        self.capture.stop()
    }

    /// Check if capture is currently running
    pub fn is_running(&self) -> bool {
        self.capture.is_running()
    }
}

impl Default for KeyboardCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for KeyboardCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;

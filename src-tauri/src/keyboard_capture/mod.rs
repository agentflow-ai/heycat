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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_capture_new_not_running() {
        let capture = KeyboardCapture::new();
        assert!(!capture.is_running());
    }

    #[test]
    fn test_keyboard_capture_stop_when_not_running() {
        let mut capture = KeyboardCapture::new();
        // Stopping when not running should be a no-op
        assert!(capture.stop().is_ok());
    }

    #[test]
    fn test_captured_key_event_has_expanded_fields() {
        // Verify the expanded CapturedKeyEvent has the new fields
        let event = CapturedKeyEvent {
            key_code: 0,
            key_name: "A".to_string(),
            fn_key: false,
            command: true,
            command_left: true,
            command_right: false,
            control: false,
            control_left: false,
            control_right: false,
            alt: false,
            alt_left: false,
            alt_right: false,
            shift: true,
            shift_left: true,
            shift_right: false,
            pressed: true,
            is_media_key: false,
        };

        assert_eq!(event.key_name, "A");
        assert!(event.command);
        assert!(event.command_left);
        assert!(!event.command_right);
        assert!(event.shift_left);
        assert!(!event.is_media_key);
    }

    #[test]
    fn test_captured_key_event_serialization() {
        let event = CapturedKeyEvent {
            key_code: 0,
            key_name: "A".to_string(),
            fn_key: true,
            command: false,
            command_left: false,
            command_right: false,
            control: false,
            control_left: false,
            control_right: false,
            alt: false,
            alt_left: false,
            alt_right: false,
            shift: false,
            shift_left: false,
            shift_right: false,
            pressed: true,
            is_media_key: false,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"fn_key\":true"));
        assert!(json.contains("\"key_name\":\"A\""));
        assert!(json.contains("\"command_left\":false"));
        assert!(json.contains("\"is_media_key\":false"));
    }

    #[test]
    fn test_captured_key_event_media_key() {
        let event = CapturedKeyEvent {
            key_code: 0, // NX_KEYTYPE_SOUND_UP
            key_name: "VolumeUp".to_string(),
            fn_key: false,
            command: false,
            command_left: false,
            command_right: false,
            control: false,
            control_left: false,
            control_right: false,
            alt: false,
            alt_left: false,
            alt_right: false,
            shift: false,
            shift_left: false,
            shift_right: false,
            pressed: true,
            is_media_key: true,
        };

        assert_eq!(event.key_name, "VolumeUp");
        assert!(event.is_media_key);
        assert!(event.pressed);
    }
}

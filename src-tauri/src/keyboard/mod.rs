// Keyboard simulation module - provides cross-platform keyboard simulation
// Uses Core Graphics on macOS (for consistency with paste simulation)
// Uses enigo crate for Windows and Linux support

pub mod synth;

#[cfg(not(target_os = "macos"))]
use enigo::{Enigo, Key, Keyboard, Settings};

/// Keyboard simulator for sending key events
pub struct KeyboardSimulator {
    #[cfg(not(target_os = "macos"))]
    enigo: Enigo,
}

impl KeyboardSimulator {
    /// Create a new KeyboardSimulator
    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Self, String> {
        Ok(Self {})
    }

    #[cfg(not(target_os = "macos"))]
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to create keyboard simulator: {}", e))?;
        Ok(Self { enigo })
    }

    /// Simulate an Enter/Return keypress
    ///
    /// On macOS, uses Core Graphics API to ensure no modifier keys are included.
    /// This is necessary because the paste simulation uses Core Graphics with Command modifier,
    /// and mixing APIs can cause residual modifier state issues.
    ///
    /// Includes a small delay to ensure previous typing is complete before sending the key.
    /// Returns Ok(()) on success, Err with message on failure.
    #[cfg(target_os = "macos")]
    pub fn simulate_enter_keypress(&mut self) -> Result<(), String> {
        use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        // Small delay to ensure previous events are processed
        std::thread::sleep(std::time::Duration::from_millis(50));

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| "Failed to create event source")?;

        // Return/Enter key = keycode 36
        let key_return: CGKeyCode = 36;

        // Key down with NO modifiers
        let event_down = CGEvent::new_keyboard_event(source.clone(), key_return, true)
            .map_err(|_| "Failed to create key down event")?;
        event_down.set_flags(CGEventFlags::empty()); // Explicitly clear all modifiers
        event_down.post(CGEventTapLocation::HID);

        // Small delay for event processing
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Key up with NO modifiers
        let event_up = CGEvent::new_keyboard_event(source, key_return, false)
            .map_err(|_| "Failed to create key up event")?;
        event_up.set_flags(CGEventFlags::empty()); // Explicitly clear all modifiers
        event_up.post(CGEventTapLocation::HID);

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn simulate_enter_keypress(&mut self) -> Result<(), String> {
        // Small delay to ensure previous typing is complete
        std::thread::sleep(std::time::Duration::from_millis(50));

        self.enigo
            .key(Key::Return, enigo::Direction::Click)
            .map_err(|e| format!("Failed to simulate enter keypress: {}", e))
    }
}

impl Default for KeyboardSimulator {
    fn default() -> Self {
        Self::new().expect("Failed to create default KeyboardSimulator")
    }
}

#[cfg(test)]
#[path = "keyboard_test.rs"]
mod tests;

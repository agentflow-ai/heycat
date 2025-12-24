#[cfg(target_os = "macos")]
mod macos {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use std::sync::{Mutex, MutexGuard};
    use std::time::Duration;

    /// Global lock to prevent interleaving multiple synthetic keyboard sequences.
    static KEYBOARD_SYNTH_MUTEX: Mutex<()> = Mutex::new(());

    fn lock_synth() -> MutexGuard<'static, ()> {
        KEYBOARD_SYNTH_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Simulate Cmd+V paste keystroke on macOS.
    ///
    /// Important: this function only checks shutdown *before* starting. Once it begins
    /// posting events, it will always post the matching key-up event.
    pub fn simulate_cmd_v_paste() -> Result<(), String> {
        // Don't start new synthesis during shutdown.
        if crate::shutdown::is_shutting_down() {
            return Ok(());
        }

        // Serialize synthesis so key sequences can't interleave across tasks.
        let _guard = lock_synth();

        // Re-check after acquiring lock to avoid starting after shutdown is signaled.
        if crate::shutdown::is_shutting_down() {
            return Ok(());
        }

        // V key = keycode 9
        let key_v: CGKeyCode = 9;

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| "Failed to create event source")?;

        // Create BOTH events before posting any, so we never post key-down without also
        // being able to post a key-up.
        let event_down = CGEvent::new_keyboard_event(source.clone(), key_v, true)
            .map_err(|_| "Failed to create key down event")?;
        let event_up = CGEvent::new_keyboard_event(source, key_v, false)
            .map_err(|_| "Failed to create key up event")?;

        let flags = CGEventFlags::CGEventFlagCommand;
        event_down.set_flags(flags);
        event_up.set_flags(flags);

        event_down.post(CGEventTapLocation::HID);

        // Keep delay minimal to reduce the window where shutdown/interruptions could occur.
        std::thread::sleep(Duration::from_millis(1));

        event_up.post(CGEventTapLocation::HID);

        Ok(())
    }

    /// Type Unicode text into the currently focused application.
    ///
    /// We post both key-down and key-up events to avoid leaving the system with a key held down.
    /// If shutdown is signaled mid-typing, we stop *between characters* (never between down/up).
    pub fn type_unicode_text(text: &str, delay_ms: u64) -> Result<(), String> {
        if crate::shutdown::is_shutting_down() {
            return Ok(());
        }

        let _guard = lock_synth();

        if crate::shutdown::is_shutting_down() {
            return Ok(());
        }

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| "Failed to create event source")?;

        for character in text.chars() {
            // Allow shutdown to stop further typing between characters.
            if crate::shutdown::is_shutting_down() {
                break;
            }

            // Encode the character to UTF-16
            let mut buf = [0u16; 2];
            let slice = character.encode_utf16(&mut buf);
            let chars: Vec<u16> = slice.to_vec();

            if chars.is_empty() {
                continue;
            }

            // Key down with unicode string (dummy keycode 0)
            let event_down = CGEvent::new_keyboard_event(source.clone(), 0, true)
                .map_err(|_| "Failed to create key down event")?;
            event_down.set_string_from_utf16_unchecked(&chars);
            event_down.post(CGEventTapLocation::HID);

            std::thread::sleep(Duration::from_millis(1));

            // Key up (no unicode string to avoid any risk of double-typing)
            let event_up = CGEvent::new_keyboard_event(source.clone(), 0, false)
                .map_err(|_| "Failed to create key up event")?;
            event_up.post(CGEventTapLocation::HID);

            if delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(delay_ms));
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub use macos::{simulate_cmd_v_paste, type_unicode_text};

#[cfg(not(target_os = "macos"))]
pub fn simulate_cmd_v_paste() -> Result<(), String> {
    Err("Paste simulation only supported on macOS".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn type_unicode_text(_text: &str, _delay_ms: u64) -> Result<(), String> {
    Err("Text input is only supported on macOS".to_string())
}



//! CGEventTap-based keyboard event capture for macOS
//!
//! This module uses CGEventTap to capture ALL keyboard events including:
//! - Regular keys (letters, numbers, symbols, special keys)
//! - Modifier keys with left/right distinction
//! - fn/Globe key via FlagsChanged
//! - Media keys (volume, brightness, play/pause) via NSSystemDefined
//! - Full modifier state tracking
//!
//! CGEventTap requires Accessibility permission (System Settings > Privacy & Security > Accessibility)
//!
//! ## Module Organization
//!
//! - `keycodes`: Keycode to human-readable name conversion
//! - `modifiers`: Modifier key flag extraction and state determination
//! - `types`: Type definitions (CapturedKeyEvent)
//! - `capture`: CGEventTapCapture struct and lifecycle management
//! - `callback`: Event callback handling

mod callback;
mod capture;
mod keycodes;
mod modifiers;
mod types;

pub use capture::CGEventTapCapture;
pub use types::CapturedKeyEvent;

use std::sync::atomic::{AtomicBool, Ordering};

/// Escape key code on macOS
pub const ESCAPE_KEY_CODE: u16 = 53;

/// Global flag to control Escape key consumption during recording.
/// When true, Escape key events are blocked from reaching other applications.
/// This flag is thread-safe and can be set from the HotkeyIntegration layer.
pub static CONSUME_ESCAPE: AtomicBool = AtomicBool::new(false);

/// Set whether Escape key events should be consumed (blocked from other apps).
/// Call with `true` when recording starts, `false` when recording stops/cancels.
pub fn set_consume_escape(consume: bool) {
    CONSUME_ESCAPE.store(consume, Ordering::SeqCst);
    crate::debug!("Escape key consume mode: {}", consume);
}

/// Get the current state of the Escape key consumption flag.
/// Returns true if Escape events are being blocked.
#[cfg(test)]
pub fn get_consume_escape() -> bool {
    CONSUME_ESCAPE.load(Ordering::SeqCst)
}

#[cfg(test)]
#[path = "cgeventtap_test.rs"]
mod tests;

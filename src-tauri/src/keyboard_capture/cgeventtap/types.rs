//! Type definitions for CGEventTap keyboard capture.

/// Captured key event with full modifier information including left/right distinction
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CapturedKeyEvent {
    /// The key code (CGKeyCode)
    pub key_code: u32,
    /// Human-readable key name
    pub key_name: String,
    /// Whether fn key is pressed
    pub fn_key: bool,
    /// Whether any command key is pressed
    pub command: bool,
    /// Whether left command is pressed
    pub command_left: bool,
    /// Whether right command is pressed
    pub command_right: bool,
    /// Whether any control key is pressed
    pub control: bool,
    /// Whether left control is pressed
    pub control_left: bool,
    /// Whether right control is pressed
    pub control_right: bool,
    /// Whether any alt/option key is pressed
    pub alt: bool,
    /// Whether left alt/option is pressed
    pub alt_left: bool,
    /// Whether right alt/option is pressed
    pub alt_right: bool,
    /// Whether any shift key is pressed
    pub shift: bool,
    /// Whether left shift is pressed
    pub shift_left: bool,
    /// Whether right shift is pressed
    pub shift_right: bool,
    /// Whether this is a key press (true) or release (false)
    pub pressed: bool,
    /// Whether this is a media key (volume, brightness, etc.)
    pub is_media_key: bool,
}

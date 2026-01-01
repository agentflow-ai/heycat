//! Modifier key flag extraction for macOS.
//!
//! Provides constants and utilities for extracting modifier key states from CGEventFlags.

use core_graphics::event::CGEventFlags;

// Standard modifier flags from CGEvent
pub const CG_EVENT_FLAG_MASK_SHIFT: u64 = 0x00020000;
pub const CG_EVENT_FLAG_MASK_CONTROL: u64 = 0x00040000;
pub const CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x00080000;
pub const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x00100000;
pub const CG_EVENT_FLAG_MASK_SECONDARY_FN: u64 = 0x00800000;

// Left/Right device flags (from IOKit NX_DEVICE*KEYMASK constants)
pub const NX_DEVICELSHIFTKEYMASK: u64 = 0x00000002;
pub const NX_DEVICERSHIFTKEYMASK: u64 = 0x00000004;
pub const NX_DEVICELCTLKEYMASK: u64 = 0x00000001;
pub const NX_DEVICERCTLKEYMASK: u64 = 0x00002000;
pub const NX_DEVICELALTKEYMASK: u64 = 0x00000020;
pub const NX_DEVICERALTKEYMASK: u64 = 0x00000040;
pub const NX_DEVICELCMDKEYMASK: u64 = 0x00000008;
pub const NX_DEVICERCMDKEYMASK: u64 = 0x00000010;

/// Modifier flags extracted from CGEventFlags
#[derive(Debug, Clone, Default)]
pub struct ModifierFlags {
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
}

impl ModifierFlags {
    /// Extract modifier flags from raw CGEventFlags bits
    pub fn from_cg_flags(flags_raw: u64) -> Self {
        Self {
            fn_key: (flags_raw & CG_EVENT_FLAG_MASK_SECONDARY_FN) != 0,
            command: (flags_raw & CG_EVENT_FLAG_MASK_COMMAND) != 0,
            control: (flags_raw & CG_EVENT_FLAG_MASK_CONTROL) != 0,
            alt: (flags_raw & CG_EVENT_FLAG_MASK_ALTERNATE) != 0,
            shift: (flags_raw & CG_EVENT_FLAG_MASK_SHIFT) != 0,
            command_left: (flags_raw & NX_DEVICELCMDKEYMASK) != 0,
            command_right: (flags_raw & NX_DEVICERCMDKEYMASK) != 0,
            control_left: (flags_raw & NX_DEVICELCTLKEYMASK) != 0,
            control_right: (flags_raw & NX_DEVICERCTLKEYMASK) != 0,
            alt_left: (flags_raw & NX_DEVICELALTKEYMASK) != 0,
            alt_right: (flags_raw & NX_DEVICERALTKEYMASK) != 0,
            shift_left: (flags_raw & NX_DEVICELSHIFTKEYMASK) != 0,
            shift_right: (flags_raw & NX_DEVICERSHIFTKEYMASK) != 0,
        }
    }
}

/// Determine which modifier key changed and whether it was pressed or released
pub fn determine_modifier_key_state(key_code: u32, flags: u64) -> (String, bool) {
    match key_code {
        // Shift keys
        56 => (
            "Shift".to_string(),
            (flags & NX_DEVICELSHIFTKEYMASK) != 0,
        ), // Left Shift
        60 => (
            "Shift".to_string(),
            (flags & NX_DEVICERSHIFTKEYMASK) != 0,
        ), // Right Shift
        // Control keys
        59 => (
            "Control".to_string(),
            (flags & NX_DEVICELCTLKEYMASK) != 0,
        ), // Left Control
        62 => (
            "Control".to_string(),
            (flags & NX_DEVICERCTLKEYMASK) != 0,
        ), // Right Control
        // Alt/Option keys
        58 => (
            "Alt".to_string(),
            (flags & NX_DEVICELALTKEYMASK) != 0,
        ), // Left Alt
        61 => (
            "Alt".to_string(),
            (flags & NX_DEVICERALTKEYMASK) != 0,
        ), // Right Alt
        // Command keys
        55 => (
            "Command".to_string(),
            (flags & NX_DEVICELCMDKEYMASK) != 0,
        ), // Left Command
        54 => (
            "Command".to_string(),
            (flags & NX_DEVICERCMDKEYMASK) != 0,
        ), // Right Command
        // Caps Lock
        57 => (
            "CapsLock".to_string(),
            (flags & CGEventFlags::CGEventFlagAlphaShift.bits()) != 0,
        ),
        // fn key - detected via the secondary fn flag
        // Key code 63 is traditional fn, 179 is Globe key on newer Macs
        63 | 179 => (
            "fn".to_string(),
            (flags & CG_EVENT_FLAG_MASK_SECONDARY_FN) != 0,
        ),
        _ => (format!("Modifier({})", key_code), true),
    }
}

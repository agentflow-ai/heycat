//! Keycode to name conversion for macOS.
//!
//! Provides functions to convert macOS key codes to human-readable key names.

/// Media key codes from IOKit/hidsystem/ev_keymap.h (NX_KEYTYPE_*)
pub const NX_KEYTYPE_SOUND_UP: u32 = 0;
pub const NX_KEYTYPE_SOUND_DOWN: u32 = 1;
pub const NX_KEYTYPE_BRIGHTNESS_UP: u32 = 2;
pub const NX_KEYTYPE_BRIGHTNESS_DOWN: u32 = 3;
pub const NX_KEYTYPE_MUTE: u32 = 7;
pub const NX_KEYTYPE_PLAY: u32 = 16;
pub const NX_KEYTYPE_NEXT: u32 = 17;
pub const NX_KEYTYPE_PREVIOUS: u32 = 18;
pub const NX_KEYTYPE_FAST: u32 = 19;
pub const NX_KEYTYPE_REWIND: u32 = 20;
pub const NX_KEYTYPE_ILLUMINATION_UP: u32 = 21;
pub const NX_KEYTYPE_ILLUMINATION_DOWN: u32 = 22;

/// Convert media key code to human-readable name
/// Media key codes are from IOKit/hidsystem/ev_keymap.h (NX_KEYTYPE_*)
pub fn media_keycode_to_name(key_code: u32) -> String {
    match key_code {
        NX_KEYTYPE_SOUND_UP => "VolumeUp".to_string(),
        NX_KEYTYPE_SOUND_DOWN => "VolumeDown".to_string(),
        NX_KEYTYPE_MUTE => "Mute".to_string(),
        NX_KEYTYPE_BRIGHTNESS_UP => "BrightnessUp".to_string(),
        NX_KEYTYPE_BRIGHTNESS_DOWN => "BrightnessDown".to_string(),
        NX_KEYTYPE_PLAY => "PlayPause".to_string(),
        NX_KEYTYPE_NEXT => "NextTrack".to_string(),
        NX_KEYTYPE_PREVIOUS => "PreviousTrack".to_string(),
        NX_KEYTYPE_FAST => "FastForward".to_string(),
        NX_KEYTYPE_REWIND => "Rewind".to_string(),
        NX_KEYTYPE_ILLUMINATION_UP => "KeyboardBrightnessUp".to_string(),
        NX_KEYTYPE_ILLUMINATION_DOWN => "KeyboardBrightnessDown".to_string(),
        _ => format!("MediaKey({})", key_code),
    }
}

/// Convert macOS key code to human-readable key name
pub fn keycode_to_name(key_code: u32) -> String {
    match key_code {
        // Letters (A-Z)
        0 => "A".to_string(),
        1 => "S".to_string(),
        2 => "D".to_string(),
        3 => "F".to_string(),
        4 => "H".to_string(),
        5 => "G".to_string(),
        6 => "Z".to_string(),
        7 => "X".to_string(),
        8 => "C".to_string(),
        9 => "V".to_string(),
        11 => "B".to_string(),
        12 => "Q".to_string(),
        13 => "W".to_string(),
        14 => "E".to_string(),
        15 => "R".to_string(),
        16 => "Y".to_string(),
        17 => "T".to_string(),
        31 => "O".to_string(),
        32 => "U".to_string(),
        34 => "I".to_string(),
        35 => "P".to_string(),
        37 => "L".to_string(),
        38 => "J".to_string(),
        40 => "K".to_string(),
        45 => "N".to_string(),
        46 => "M".to_string(),

        // Numbers (top row)
        18 => "1".to_string(),
        19 => "2".to_string(),
        20 => "3".to_string(),
        21 => "4".to_string(),
        22 => "6".to_string(),
        23 => "5".to_string(),
        24 => "=".to_string(),
        25 => "9".to_string(),
        26 => "7".to_string(),
        27 => "-".to_string(),
        28 => "8".to_string(),
        29 => "0".to_string(),

        // Punctuation and symbols
        30 => "]".to_string(),
        33 => "[".to_string(),
        39 => "'".to_string(),
        41 => ";".to_string(),
        42 => "\\".to_string(),
        43 => ",".to_string(),
        44 => "/".to_string(),
        47 => ".".to_string(),
        50 => "`".to_string(),

        // Special keys
        36 => "Enter".to_string(),
        48 => "Tab".to_string(),
        49 => "Space".to_string(),
        51 => "Backspace".to_string(),
        53 => "Escape".to_string(),

        // Modifier keys
        54 => "Command".to_string(),  // Right Command
        55 => "Command".to_string(),  // Left Command
        56 => "Shift".to_string(),    // Left Shift
        57 => "CapsLock".to_string(),
        58 => "Alt".to_string(),      // Left Alt/Option
        59 => "Control".to_string(),  // Left Control
        60 => "Shift".to_string(),    // Right Shift
        61 => "Alt".to_string(),      // Right Alt/Option
        62 => "Control".to_string(),  // Right Control
        63 | 179 => "fn".to_string(), // fn/Globe key (179 on newer Macs)

        // Function keys
        122 => "F1".to_string(),
        120 => "F2".to_string(),
        99 => "F3".to_string(),
        118 => "F4".to_string(),
        96 => "F5".to_string(),
        97 => "F6".to_string(),
        98 => "F7".to_string(),
        100 => "F8".to_string(),
        101 => "F9".to_string(),
        109 => "F10".to_string(),
        103 => "F11".to_string(),
        111 => "F12".to_string(),
        105 => "F13".to_string(),
        107 => "F14".to_string(),
        113 => "F15".to_string(),
        106 => "F16".to_string(),
        64 => "F17".to_string(),
        79 => "F18".to_string(),
        80 => "F19".to_string(),

        // Navigation keys
        123 => "Left".to_string(),
        124 => "Right".to_string(),
        125 => "Down".to_string(),
        126 => "Up".to_string(),
        115 => "Home".to_string(),
        116 => "PageUp".to_string(),
        117 => "Delete".to_string(), // Forward Delete
        119 => "End".to_string(),
        121 => "PageDown".to_string(),

        // Numpad keys
        65 => "Numpad.".to_string(),
        67 => "Numpad*".to_string(),
        69 => "Numpad+".to_string(),
        71 => "NumpadClear".to_string(),
        75 => "Numpad/".to_string(),
        76 => "NumpadEnter".to_string(),
        78 => "Numpad-".to_string(),
        81 => "Numpad=".to_string(),
        82 => "Numpad0".to_string(),
        83 => "Numpad1".to_string(),
        84 => "Numpad2".to_string(),
        85 => "Numpad3".to_string(),
        86 => "Numpad4".to_string(),
        87 => "Numpad5".to_string(),
        88 => "Numpad6".to_string(),
        89 => "Numpad7".to_string(),
        91 => "Numpad8".to_string(),
        92 => "Numpad9".to_string(),

        // Other
        10 => "Section".to_string(),       // § key (ISO keyboards)
        52 => "International".to_string(), // International key
        102 => "Help".to_string(),         // Help key (older keyboards)
        110 => "ContextMenu".to_string(),

        _ => format!("Key({})", key_code),
    }
}

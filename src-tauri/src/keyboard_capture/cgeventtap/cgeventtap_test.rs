use super::keycodes::{
    keycode_to_name, media_keycode_to_name, NX_KEYTYPE_BRIGHTNESS_DOWN, NX_KEYTYPE_BRIGHTNESS_UP,
    NX_KEYTYPE_FAST, NX_KEYTYPE_ILLUMINATION_DOWN, NX_KEYTYPE_ILLUMINATION_UP, NX_KEYTYPE_MUTE,
    NX_KEYTYPE_NEXT, NX_KEYTYPE_PLAY, NX_KEYTYPE_PREVIOUS, NX_KEYTYPE_REWIND,
    NX_KEYTYPE_SOUND_DOWN, NX_KEYTYPE_SOUND_UP,
};
use super::modifiers::{
    determine_modifier_key_state, CG_EVENT_FLAG_MASK_SECONDARY_FN, NX_DEVICELCMDKEYMASK,
    NX_DEVICELSHIFTKEYMASK, NX_DEVICERCMDKEYMASK, NX_DEVICERSHIFTKEYMASK,
};
use super::types::CapturedKeyEvent;
use super::{get_consume_escape, set_consume_escape, CGEventTapCapture, ESCAPE_KEY_CODE};

#[test]
fn test_keycode_to_name_letters() {
    assert_eq!(keycode_to_name(0), "A");
    assert_eq!(keycode_to_name(1), "S");
    assert_eq!(keycode_to_name(6), "Z");
    assert_eq!(keycode_to_name(12), "Q");
}

#[test]
fn test_keycode_to_name_numbers() {
    assert_eq!(keycode_to_name(18), "1");
    assert_eq!(keycode_to_name(29), "0");
    assert_eq!(keycode_to_name(23), "5");
}

#[test]
fn test_keycode_to_name_function_keys() {
    assert_eq!(keycode_to_name(122), "F1");
    assert_eq!(keycode_to_name(96), "F5");
    assert_eq!(keycode_to_name(111), "F12");
    assert_eq!(keycode_to_name(80), "F19");
}

#[test]
fn test_keycode_to_name_special_keys() {
    assert_eq!(keycode_to_name(36), "Enter");
    assert_eq!(keycode_to_name(49), "Space");
    assert_eq!(keycode_to_name(48), "Tab");
    assert_eq!(keycode_to_name(53), "Escape");
    assert_eq!(keycode_to_name(51), "Backspace");
}

#[test]
fn test_keycode_to_name_modifiers() {
    assert_eq!(keycode_to_name(55), "Command");  // Left
    assert_eq!(keycode_to_name(54), "Command");  // Right
    assert_eq!(keycode_to_name(56), "Shift");    // Left
    assert_eq!(keycode_to_name(60), "Shift");    // Right
    assert_eq!(keycode_to_name(58), "Alt");      // Left
    assert_eq!(keycode_to_name(61), "Alt");      // Right
    assert_eq!(keycode_to_name(59), "Control");  // Left
    assert_eq!(keycode_to_name(62), "Control");  // Right
    assert_eq!(keycode_to_name(63), "fn");
}

#[test]
fn test_keycode_to_name_navigation() {
    assert_eq!(keycode_to_name(123), "Left");
    assert_eq!(keycode_to_name(124), "Right");
    assert_eq!(keycode_to_name(125), "Down");
    assert_eq!(keycode_to_name(126), "Up");
}

#[test]
fn test_keycode_to_name_numpad() {
    assert_eq!(keycode_to_name(82), "Numpad0");
    assert_eq!(keycode_to_name(83), "Numpad1");
    assert_eq!(keycode_to_name(92), "Numpad9");
    assert_eq!(keycode_to_name(76), "NumpadEnter");
}

#[test]
fn test_keycode_to_name_unknown() {
    assert_eq!(keycode_to_name(255), "Key(255)");
}

#[test]
fn test_captured_key_event_default() {
    let event = CapturedKeyEvent::default();
    assert_eq!(event.key_code, 0);
    assert_eq!(event.key_name, "");
    assert!(!event.fn_key);
    assert!(!event.command);
    assert!(!event.command_left);
    assert!(!event.command_right);
    assert!(!event.pressed);
    assert!(!event.is_media_key);
}

#[test]
fn test_captured_key_event_serialization() {
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
        shift_left: false,
        shift_right: true,
        pressed: true,
        is_media_key: false,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"key_name\":\"A\""));
    assert!(json.contains("\"command\":true"));
    assert!(json.contains("\"command_left\":true"));
    assert!(json.contains("\"command_right\":false"));
    assert!(json.contains("\"shift_right\":true"));
}

#[test]
fn test_determine_modifier_key_state_left_shift() {
    let (name, pressed) = determine_modifier_key_state(56, NX_DEVICELSHIFTKEYMASK);
    assert_eq!(name, "Shift");
    assert!(pressed);

    let (_, released) = determine_modifier_key_state(56, 0);
    assert!(!released);
}

#[test]
fn test_determine_modifier_key_state_right_shift() {
    let (name, pressed) = determine_modifier_key_state(60, NX_DEVICERSHIFTKEYMASK);
    assert_eq!(name, "Shift");
    assert!(pressed);
}

#[test]
fn test_determine_modifier_key_state_left_command() {
    let (name, pressed) = determine_modifier_key_state(55, NX_DEVICELCMDKEYMASK);
    assert_eq!(name, "Command");
    assert!(pressed);
}

#[test]
fn test_determine_modifier_key_state_right_command() {
    let (name, pressed) = determine_modifier_key_state(54, NX_DEVICERCMDKEYMASK);
    assert_eq!(name, "Command");
    assert!(pressed);
}

#[test]
fn test_determine_modifier_key_state_fn() {
    let (name, pressed) = determine_modifier_key_state(63, CG_EVENT_FLAG_MASK_SECONDARY_FN);
    assert_eq!(name, "fn");
    assert!(pressed);
}

#[test]
fn test_cgeventtap_capture_new_not_running() {
    let capture = CGEventTapCapture::new();
    assert!(!capture.is_running());
}

#[test]
fn test_cgeventtap_capture_stop_when_not_running() {
    let mut capture = CGEventTapCapture::new();
    // Stopping when not running should be a no-op
    assert!(capture.stop().is_ok());
}

#[test]
fn test_media_keycode_to_name_volume() {
    assert_eq!(media_keycode_to_name(NX_KEYTYPE_SOUND_UP), "VolumeUp");
    assert_eq!(media_keycode_to_name(NX_KEYTYPE_SOUND_DOWN), "VolumeDown");
    assert_eq!(media_keycode_to_name(NX_KEYTYPE_MUTE), "Mute");
}

#[test]
fn test_media_keycode_to_name_brightness() {
    assert_eq!(
        media_keycode_to_name(NX_KEYTYPE_BRIGHTNESS_UP),
        "BrightnessUp"
    );
    assert_eq!(
        media_keycode_to_name(NX_KEYTYPE_BRIGHTNESS_DOWN),
        "BrightnessDown"
    );
}

#[test]
fn test_media_keycode_to_name_playback() {
    assert_eq!(media_keycode_to_name(NX_KEYTYPE_PLAY), "PlayPause");
    assert_eq!(media_keycode_to_name(NX_KEYTYPE_NEXT), "NextTrack");
    assert_eq!(media_keycode_to_name(NX_KEYTYPE_PREVIOUS), "PreviousTrack");
    assert_eq!(media_keycode_to_name(NX_KEYTYPE_FAST), "FastForward");
    assert_eq!(media_keycode_to_name(NX_KEYTYPE_REWIND), "Rewind");
}

#[test]
fn test_media_keycode_to_name_keyboard_backlight() {
    assert_eq!(
        media_keycode_to_name(NX_KEYTYPE_ILLUMINATION_UP),
        "KeyboardBrightnessUp"
    );
    assert_eq!(
        media_keycode_to_name(NX_KEYTYPE_ILLUMINATION_DOWN),
        "KeyboardBrightnessDown"
    );
}

#[test]
fn test_media_keycode_to_name_unknown() {
    assert_eq!(media_keycode_to_name(255), "MediaKey(255)");
}

#[test]
fn test_captured_key_event_media_key() {
    let event = CapturedKeyEvent {
        key_code: NX_KEYTYPE_SOUND_UP,
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

    assert_eq!(event.key_code, 0);
    assert_eq!(event.key_name, "VolumeUp");
    assert!(event.pressed);
    assert!(event.is_media_key);
}

// === Escape key consumption tests ===

#[test]
fn test_consume_escape_default_false() {
    // Reset to known state first
    set_consume_escape(false);
    // Default state should be false (Escape passes through)
    assert!(!get_consume_escape());
}

#[test]
fn test_consume_escape_set_true() {
    // Set consume mode to true
    set_consume_escape(true);
    assert!(get_consume_escape());
    // Clean up
    set_consume_escape(false);
}

#[test]
fn test_consume_escape_set_false() {
    // First set to true
    set_consume_escape(true);
    assert!(get_consume_escape());
    // Then set back to false
    set_consume_escape(false);
    assert!(!get_consume_escape());
}

#[test]
fn test_escape_key_code_constant() {
    // Verify the ESCAPE_KEY_CODE constant is correct (53 on macOS)
    assert_eq!(ESCAPE_KEY_CODE, 53);
}

#[test]
fn test_keycode_to_name_escape() {
    // Verify Escape key is properly mapped
    assert_eq!(keycode_to_name(ESCAPE_KEY_CODE as u32), "Escape");
}

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

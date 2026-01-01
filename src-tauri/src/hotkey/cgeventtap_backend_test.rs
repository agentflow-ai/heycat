use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn test_parse_shortcut_standard() {
    let spec = parse_shortcut("CmdOrControl+Shift+R").unwrap();
    assert!(spec.command);
    assert!(spec.shift);
    assert!(!spec.fn_key);
    assert!(!spec.control);
    assert!(!spec.alt);
    assert_eq!(spec.key_name, Some("R".to_string()));
    assert!(!spec.is_media_key);
}

#[test]
fn test_parse_shortcut_with_fn() {
    let spec = parse_shortcut("fn+Command+R").unwrap();
    assert!(spec.fn_key);
    assert!(spec.command);
    assert_eq!(spec.key_name, Some("R".to_string()));
}

#[test]
fn test_parse_shortcut_fn_only() {
    let spec = parse_shortcut("fn").unwrap();
    assert!(spec.fn_key);
    assert!(!spec.command);
    assert!(!spec.shift);
    assert_eq!(spec.key_name, None);
}

#[test]
fn test_parse_shortcut_media_key() {
    let spec = parse_shortcut("PlayPause").unwrap();
    assert!(spec.is_media_key);
    assert_eq!(spec.key_name, Some("PlayPause".to_string()));
    assert!(!spec.fn_key);
}

#[test]
fn test_parse_shortcut_media_key_with_modifier() {
    let spec = parse_shortcut("Command+PlayPause").unwrap();
    assert!(spec.is_media_key);
    assert!(spec.command);
    assert_eq!(spec.key_name, Some("PlayPause".to_string()));
}

#[test]
fn test_parse_shortcut_escape() {
    let spec = parse_shortcut("Escape").unwrap();
    assert_eq!(spec.key_name, Some("Escape".to_string()));
    assert!(!spec.is_media_key);
}

#[test]
fn test_parse_shortcut_function_key() {
    let spec = parse_shortcut("Command+F5").unwrap();
    assert!(spec.command);
    assert_eq!(spec.key_name, Some("F5".to_string()));
}

#[test]
fn test_matches_shortcut_basic() {
    let spec = parse_shortcut("Command+Shift+R").unwrap();
    let event = CapturedKeyEvent {
        key_code: 15, // R
        key_name: "R".to_string(),
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
    assert!(matches_shortcut(&event, &spec));
}

#[test]
fn test_matches_shortcut_wrong_key() {
    let spec = parse_shortcut("Command+Shift+R").unwrap();
    let event = CapturedKeyEvent {
        key_code: 0, // A
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
    assert!(!matches_shortcut(&event, &spec));
}

#[test]
fn test_matches_shortcut_release_ignored() {
    let spec = parse_shortcut("Command+Shift+R").unwrap();
    let event = CapturedKeyEvent {
        key_code: 15,
        key_name: "R".to_string(),
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
        pressed: false, // Release
        is_media_key: false,
    };
    assert!(!matches_shortcut(&event, &spec));
}

#[test]
fn test_matches_shortcut_with_fn() {
    let spec = parse_shortcut("fn+Command+R").unwrap();
    let event = CapturedKeyEvent {
        key_code: 15,
        key_name: "R".to_string(),
        fn_key: true,
        command: true,
        command_left: true,
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
    assert!(matches_shortcut(&event, &spec));
}

#[test]
fn test_matches_shortcut_media_key() {
    let spec = parse_shortcut("PlayPause").unwrap();
    let event = CapturedKeyEvent {
        key_code: 16, // NX_KEYTYPE_PLAY
        key_name: "PlayPause".to_string(),
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
    assert!(matches_shortcut(&event, &spec));
}

#[test]
fn test_normalize_key_name() {
    assert_eq!(normalize_key_name("r"), "R");
    assert_eq!(normalize_key_name("R"), "R");
    assert_eq!(normalize_key_name("escape"), "Escape");
    assert_eq!(normalize_key_name("Esc"), "Escape");
    assert_eq!(normalize_key_name("F5"), "F5");
    assert_eq!(normalize_key_name("f12"), "F12");
}

#[test]
fn test_matches_shortcut_fn_only_with_fn_key() {
    // fn-only shortcut should match when fn key itself is pressed
    let spec = parse_shortcut("fn").unwrap();
    let event = CapturedKeyEvent {
        key_code: 63, // fn key
        key_name: "fn".to_string(),
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
    assert!(matches_shortcut(&event, &spec));
}

#[test]
fn test_matches_shortcut_fn_only_rejects_arrow_keys() {
    // fn-only shortcut should NOT match arrow keys even though they have fn flag
    let spec = parse_shortcut("fn").unwrap();

    // Test all arrow keys - they all have fn_key=true on macOS but should not trigger
    for (key_code, key_name) in [(123, "Left"), (124, "Right"), (125, "Down"), (126, "Up")] {
        let event = CapturedKeyEvent {
            key_code,
            key_name: key_name.to_string(),
            fn_key: true, // macOS sets this flag for arrow keys!
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
        assert!(
            !matches_shortcut(&event, &spec),
            "Arrow key '{}' should not trigger fn-only shortcut",
            key_name
        );
    }
}

#[test]
fn test_matches_shortcut_command_only() {
    // Command-only shortcut should match when Command key is pressed
    let spec = parse_shortcut("Command").unwrap();
    let event = CapturedKeyEvent {
        key_code: 55, // Left Command
        key_name: "Command".to_string(),
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
        shift: false,
        shift_left: false,
        shift_right: false,
        pressed: true,
        is_media_key: false,
    };
    assert!(matches_shortcut(&event, &spec));
}

#[test]
fn test_matches_shortcut_modifier_only_rejects_regular_keys() {
    // Modifier-only shortcuts should not match regular keys
    let spec = parse_shortcut("Command").unwrap();
    let event = CapturedKeyEvent {
        key_code: 0, // A key
        key_name: "A".to_string(),
        fn_key: false,
        command: true, // Command is held
        command_left: true,
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
    // Should NOT match - we want Command key itself, not Command+A
    assert!(!matches_shortcut(&event, &spec));
}

#[test]
fn test_backend_new() {
    let backend = CGEventTapHotkeyBackend::new();
    assert!(!backend.running.load(Ordering::SeqCst));
}

#[test]
fn test_matches_shortcut_release() {
    let spec = parse_shortcut("Command+Shift+R").unwrap();
    let event = CapturedKeyEvent {
        key_code: 15,
        key_name: "R".to_string(),
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
        pressed: false, // Release event
        is_media_key: false,
    };
    // matches_shortcut should NOT match release events
    assert!(!matches_shortcut(&event, &spec));
    // matches_shortcut_release SHOULD match release events
    assert!(matches_shortcut_release(&event, &spec));
}

#[test]
fn test_matches_shortcut_release_rejects_press() {
    let spec = parse_shortcut("Command+Shift+R").unwrap();
    let event = CapturedKeyEvent {
        key_code: 15,
        key_name: "R".to_string(),
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
        pressed: true, // Press event
        is_media_key: false,
    };
    // matches_shortcut SHOULD match press events
    assert!(matches_shortcut(&event, &spec));
    // matches_shortcut_release should NOT match press events
    assert!(!matches_shortcut_release(&event, &spec));
}

#[test]
fn test_backend_has_release_callbacks() {
    let backend = CGEventTapHotkeyBackend::new();
    // Verify the backend has release_callbacks field initialized
    let release_callbacks = backend.release_callbacks.lock().unwrap();
    assert!(release_callbacks.is_empty());
}

#[test]
fn test_handle_key_event_calls_press_callback() {
    let shortcuts = Arc::new(Mutex::new(HashMap::new()));
    let callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));
    let release_callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));

    let spec = parse_shortcut("R").unwrap();
    shortcuts.lock().unwrap().insert("R".to_string(), spec);

    let press_called = Arc::new(AtomicBool::new(false));
    let press_flag = Arc::clone(&press_called);
    callbacks.lock().unwrap().insert(
        "R".to_string(),
        Arc::new(move || press_flag.store(true, Ordering::SeqCst)),
    );

    let event = CapturedKeyEvent {
        key_code: 15,
        key_name: "R".to_string(),
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
        is_media_key: false,
    };

    CGEventTapHotkeyBackend::handle_key_event(&event, &shortcuts, &callbacks, &release_callbacks);
    assert!(press_called.load(Ordering::SeqCst));
}

#[test]
fn test_handle_key_event_calls_release_callback() {
    let shortcuts = Arc::new(Mutex::new(HashMap::new()));
    let callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));
    let release_callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));

    let spec = parse_shortcut("R").unwrap();
    shortcuts.lock().unwrap().insert("R".to_string(), spec);

    let release_called = Arc::new(AtomicBool::new(false));
    let release_flag = Arc::clone(&release_called);
    release_callbacks.lock().unwrap().insert(
        "R".to_string(),
        Arc::new(move || release_flag.store(true, Ordering::SeqCst)),
    );

    let event = CapturedKeyEvent {
        key_code: 15,
        key_name: "R".to_string(),
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
        pressed: false, // Release
        is_media_key: false,
    };

    CGEventTapHotkeyBackend::handle_key_event(&event, &shortcuts, &callbacks, &release_callbacks);
    assert!(release_called.load(Ordering::SeqCst));
}

// === PTT Release Matching Tests ===
// These tests verify the fix for the PTT bug where key release wasn't detected
// when modifier flags were already cleared.

#[test]
fn test_matches_shortcut_release_fn_only_flag_cleared() {
    // When fn key is released, the fn flag is immediately cleared.
    // The release matcher should still match based on key_name, not flag state.
    let spec = parse_shortcut("fn").unwrap();
    let event = CapturedKeyEvent {
        key_code: 63, // fn key
        key_name: "fn".to_string(),
        fn_key: false, // Flag is CLEARED when fn is released!
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
        pressed: false, // Release event
        is_media_key: false,
    };
    // This is the critical test - release should match even though fn_key flag is false
    assert!(
        matches_shortcut_release(&event, &spec),
        "fn release should match fn-only shortcut even when fn flag is cleared"
    );
}

#[test]
fn test_matches_shortcut_release_regular_key_modifiers_already_released() {
    // When using ⌘⇧R shortcut and user releases ⌘ first, then R:
    // The R release event will have command=false (⌘ already released).
    // The release matcher should still match based on key_name.
    let spec = parse_shortcut("Command+Shift+R").unwrap();
    let event = CapturedKeyEvent {
        key_code: 15,
        key_name: "R".to_string(),
        fn_key: false,
        command: false, // Command was released BEFORE R
        command_left: false,
        command_right: false,
        control: false,
        control_left: false,
        control_right: false,
        alt: false,
        alt_left: false,
        alt_right: false,
        shift: false, // Shift was also released before R
        shift_left: false,
        shift_right: false,
        pressed: false, // Release event
        is_media_key: false,
    };
    // This is the critical test - release should match even though modifier flags are false
    assert!(
        matches_shortcut_release(&event, &spec),
        "R release should match ⌘⇧R shortcut even when modifiers already released"
    );
}

#[test]
fn test_matches_shortcut_release_command_only_flag_cleared() {
    // When Command-only shortcut is released, the command flag is cleared.
    let spec = parse_shortcut("Command").unwrap();
    let event = CapturedKeyEvent {
        key_code: 55, // Left Command
        key_name: "Command".to_string(),
        fn_key: false,
        command: false, // Flag is CLEARED when Command is released!
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
        pressed: false, // Release event
        is_media_key: false,
    };
    assert!(
        matches_shortcut_release(&event, &spec),
        "Command release should match Command-only shortcut even when command flag is cleared"
    );
}

#[test]
fn test_matches_shortcut_release_wrong_key_does_not_match() {
    // Release of a different key should NOT match
    let spec = parse_shortcut("Command+Shift+R").unwrap();
    let event = CapturedKeyEvent {
        key_code: 0,
        key_name: "A".to_string(), // Wrong key!
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
        pressed: false, // Release event
        is_media_key: false,
    };
    assert!(
        !matches_shortcut_release(&event, &spec),
        "A release should NOT match ⌘⇧R shortcut"
    );
}

#[test]
fn test_matches_shortcut_release_media_key() {
    // Media key release should match based on key name
    let spec = parse_shortcut("PlayPause").unwrap();
    let event = CapturedKeyEvent {
        key_code: 16,
        key_name: "PlayPause".to_string(),
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
        pressed: false, // Release event
        is_media_key: true,
    };
    assert!(
        matches_shortcut_release(&event, &spec),
        "PlayPause release should match PlayPause shortcut"
    );
}

#[test]
fn test_matches_shortcut_release_fn_only_rejects_other_keys() {
    // When fn-only shortcut, release of other keys should NOT match
    let spec = parse_shortcut("fn").unwrap();
    let event = CapturedKeyEvent {
        key_code: 15,
        key_name: "R".to_string(), // Not fn key!
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
        pressed: false, // Release event
        is_media_key: false,
    };
    assert!(
        !matches_shortcut_release(&event, &spec),
        "R release should NOT match fn-only shortcut"
    );
}

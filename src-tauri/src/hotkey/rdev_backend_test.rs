// Tests for rdev_backend module
// Test code is excluded from coverage since we measure production code coverage
// Note: These tests only run on Windows/Linux (not macOS)
#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;

#[test]
fn test_parse_shortcut_key_escape() {
    let key = parse_shortcut_key("Escape").unwrap();
    assert_eq!(key, Key::Escape);
}

#[test]
fn test_parse_shortcut_key_letter() {
    let key = parse_shortcut_key("R").unwrap();
    assert_eq!(key, Key::KeyR);

    let key_lower = parse_shortcut_key("r").unwrap();
    assert_eq!(key_lower, Key::KeyR);
}

#[test]
fn test_parse_shortcut_key_with_modifier() {
    // Should extract just the key part
    let key = parse_shortcut_key("Command+R").unwrap();
    assert_eq!(key, Key::KeyR);

    let key_multi = parse_shortcut_key("Command+Shift+R").unwrap();
    assert_eq!(key_multi, Key::KeyR);
}

#[test]
fn test_parse_shortcut_key_function_keys() {
    assert_eq!(parse_shortcut_key("F1").unwrap(), Key::F1);
    assert_eq!(parse_shortcut_key("F12").unwrap(), Key::F12);
}

#[test]
fn test_parse_shortcut_key_navigation() {
    assert_eq!(parse_shortcut_key("Enter").unwrap(), Key::Return);
    assert_eq!(parse_shortcut_key("Space").unwrap(), Key::Space);
    assert_eq!(parse_shortcut_key("Tab").unwrap(), Key::Tab);
    assert_eq!(parse_shortcut_key("Backspace").unwrap(), Key::Backspace);
}

#[test]
fn test_parse_shortcut_key_unknown() {
    let result = parse_shortcut_key("UnknownKey");
    assert!(result.is_err());
}

#[test]
fn test_backend_new() {
    let backend = RdevShortcutBackend::new();
    assert!(!backend.running.load(Ordering::SeqCst));
}

#[test]
fn test_backend_default() {
    let backend = RdevShortcutBackend::default();
    assert!(!backend.running.load(Ordering::SeqCst));
}

#[test]
fn test_handle_event_press_callback() {
    use std::sync::atomic::AtomicBool;

    let shortcuts = Arc::new(Mutex::new(HashMap::new()));
    let press_callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));
    let release_callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));

    shortcuts.lock().unwrap().insert("R".to_string(), Key::KeyR);

    let press_called = Arc::new(AtomicBool::new(false));
    let press_flag = Arc::clone(&press_called);
    press_callbacks.lock().unwrap().insert(
        "R".to_string(),
        Arc::new(move || press_flag.store(true, Ordering::SeqCst)),
    );

    let event = Event {
        time: std::time::SystemTime::now(),
        name: None,
        event_type: EventType::KeyPress(Key::KeyR),
    };

    RdevShortcutBackend::handle_event(&event, &shortcuts, &press_callbacks, &release_callbacks);
    assert!(press_called.load(Ordering::SeqCst));
}

#[test]
fn test_handle_event_release_callback() {
    use std::sync::atomic::AtomicBool;

    let shortcuts = Arc::new(Mutex::new(HashMap::new()));
    let press_callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));
    let release_callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));

    shortcuts.lock().unwrap().insert("R".to_string(), Key::KeyR);

    let release_called = Arc::new(AtomicBool::new(false));
    let release_flag = Arc::clone(&release_called);
    release_callbacks.lock().unwrap().insert(
        "R".to_string(),
        Arc::new(move || release_flag.store(true, Ordering::SeqCst)),
    );

    let event = Event {
        time: std::time::SystemTime::now(),
        name: None,
        event_type: EventType::KeyRelease(Key::KeyR),
    };

    RdevShortcutBackend::handle_event(&event, &shortcuts, &press_callbacks, &release_callbacks);
    assert!(release_called.load(Ordering::SeqCst));
}

#[test]
fn test_handle_event_ignores_mouse_events() {
    use std::sync::atomic::AtomicBool;

    let shortcuts = Arc::new(Mutex::new(HashMap::new()));
    let press_callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));
    let release_callbacks: CallbackMap = Arc::new(Mutex::new(HashMap::new()));

    shortcuts.lock().unwrap().insert("R".to_string(), Key::KeyR);

    let press_called = Arc::new(AtomicBool::new(false));
    let press_flag = Arc::clone(&press_called);
    press_callbacks.lock().unwrap().insert(
        "R".to_string(),
        Arc::new(move || press_flag.store(true, Ordering::SeqCst)),
    );

    // Mouse event should be ignored
    let event = Event {
        time: std::time::SystemTime::now(),
        name: None,
        event_type: EventType::MouseMove { x: 0.0, y: 0.0 },
    };

    RdevShortcutBackend::handle_event(&event, &shortcuts, &press_callbacks, &release_callbacks);
    assert!(!press_called.load(Ordering::SeqCst));
}

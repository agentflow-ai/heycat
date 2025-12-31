// Tests for hotkey module
// Test code is excluded from coverage since we measure production code coverage
#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct MockBackend {
    should_fail: bool,
    error_msg: String,
}

struct MockBackendExt {
    should_fail: bool,
    error_msg: String,
}

impl ShortcutBackend for MockBackendExt {
    fn register(&self, _: &str, callback: Box<dyn Fn() + Send + Sync>) -> Result<(), String> {
        if self.should_fail {
            Err(self.error_msg.clone())
        } else {
            callback();
            Ok(())
        }
    }

    fn unregister(&self, _: &str) -> Result<(), String> {
        if self.should_fail {
            Err(self.error_msg.clone())
        } else {
            Ok(())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ShortcutBackendExt for MockBackendExt {
    fn register_with_release(
        &self,
        _shortcut: &str,
        on_press: Box<dyn Fn() + Send + Sync>,
        on_release: Box<dyn Fn() + Send + Sync>,
    ) -> Result<(), String> {
        if self.should_fail {
            Err(self.error_msg.clone())
        } else {
            on_press();
            on_release();
            Ok(())
        }
    }
}

impl ShortcutBackend for MockBackend {
    fn register(&self, _: &str, callback: Box<dyn Fn() + Send + Sync>) -> Result<(), String> {
        if self.should_fail {
            Err(self.error_msg.clone())
        } else {
            callback(); // Actually invoke the callback
            Ok(())
        }
    }

    fn unregister(&self, _: &str) -> Result<(), String> {
        if self.should_fail {
            Err(self.error_msg.clone())
        } else {
            Ok(())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_map_already_registered() {
    assert_eq!(
        map_backend_error("already registered"),
        HotkeyError::AlreadyRegistered
    );
}

#[test]
fn test_map_conflict() {
    assert!(matches!(
        map_backend_error("conflict"),
        HotkeyError::Conflict(_)
    ));
}

#[test]
fn test_map_in_use() {
    assert!(matches!(
        map_backend_error("shortcut in use"),
        HotkeyError::Conflict(_)
    ));
}

#[test]
fn test_map_unknown_error() {
    assert!(matches!(
        map_backend_error("unknown"),
        HotkeyError::RegistrationFailed(_)
    ));
}

#[test]
fn test_backend_register_success() {
    let backend = MockBackend {
        should_fail: false,
        error_msg: String::new(),
    };
    let result = backend.register("Command+Shift+R", Box::new(|| {}));
    assert!(result.is_ok());
}

#[test]
fn test_backend_register_failure() {
    let backend = MockBackend {
        should_fail: true,
        error_msg: "conflict".into(),
    };
    let result = backend.register("Command+Shift+R", Box::new(|| {}));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "conflict");
}

#[test]
fn test_backend_unregister_success() {
    let backend = MockBackend {
        should_fail: false,
        error_msg: String::new(),
    };
    let result = backend.unregister("Command+Shift+R");
    assert!(result.is_ok());
}

#[test]
fn test_backend_unregister_failure() {
    let backend = MockBackend {
        should_fail: true,
        error_msg: "not registered".into(),
    };
    let result = backend.unregister("Command+Shift+R");
    assert!(result.is_err());
}

#[test]
fn test_recording_mode_default_is_toggle() {
    assert_eq!(RecordingMode::default(), RecordingMode::Toggle);
}

#[test]
fn test_recording_mode_serialization() {
    // Test kebab-case serialization
    let toggle_json = serde_json::to_string(&RecordingMode::Toggle).unwrap();
    assert_eq!(toggle_json, "\"toggle\"");

    let ptt_json = serde_json::to_string(&RecordingMode::PushToTalk).unwrap();
    assert_eq!(ptt_json, "\"push-to-talk\"");

    // Test deserialization
    let toggle: RecordingMode = serde_json::from_str("\"toggle\"").unwrap();
    assert_eq!(toggle, RecordingMode::Toggle);

    let ptt: RecordingMode = serde_json::from_str("\"push-to-talk\"").unwrap();
    assert_eq!(ptt, RecordingMode::PushToTalk);
}

#[test]
fn test_register_with_release_calls_both_callbacks() {
    let press_called = Arc::new(AtomicBool::new(false));
    let release_called = Arc::new(AtomicBool::new(false));

    let press_flag = Arc::clone(&press_called);
    let release_flag = Arc::clone(&release_called);

    let backend = MockBackendExt {
        should_fail: false,
        error_msg: String::new(),
    };

    let result = backend.register_with_release(
        "Command+Shift+R",
        Box::new(move || press_flag.store(true, Ordering::SeqCst)),
        Box::new(move || release_flag.store(true, Ordering::SeqCst)),
    );

    assert!(result.is_ok());
    assert!(press_called.load(Ordering::SeqCst));
    assert!(release_called.load(Ordering::SeqCst));
}

#[test]
fn test_register_with_release_failure() {
    let backend = MockBackendExt {
        should_fail: true,
        error_msg: "registration failed".into(),
    };

    let result = backend.register_with_release(
        "Command+Shift+R",
        Box::new(|| {}),
        Box::new(|| {}),
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "registration failed");
}

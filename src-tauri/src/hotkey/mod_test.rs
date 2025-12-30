// Tests for hotkey module
// Test code is excluded from coverage since we measure production code coverage
#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;

struct MockBackend {
    should_fail: bool,
    error_msg: String,
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

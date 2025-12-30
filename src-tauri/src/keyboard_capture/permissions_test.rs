use super::*;

#[test]
fn test_check_accessibility_permission_returns_bool() {
    // This test verifies that the function can be called and returns a boolean
    // The actual return value depends on system state
    let result = check_accessibility_permission();
    // Just verify it's a valid boolean (not a crash or undefined behavior)
    assert!(result == true || result == false);
}

#[test]
#[ignore] // Opens System Settings - skip during automated test runs
fn test_open_accessibility_settings_succeeds() {
    // This test verifies that open_accessibility_settings can spawn the open command
    // Note: This will actually open System Settings on the test machine
    // The function returns Ok if spawn succeeds, regardless of whether Settings opens
    let result = open_accessibility_settings();
    assert!(result.is_ok(), "open_accessibility_settings should succeed: {:?}", result);
}

#[test]
fn test_accessibility_permission_error_display() {
    let error = AccessibilityPermissionError::new();
    let display = format!("{}", error);
    assert!(display.contains("Accessibility permission required"));
    assert!(display.contains("System Settings"));
}

#[test]
fn test_accessibility_permission_error_default() {
    let error = AccessibilityPermissionError::default();
    assert!(error.message.contains("Accessibility permission required"));
}

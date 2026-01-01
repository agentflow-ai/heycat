// Tests for the recording storage module

use super::*;

#[test]
fn test_window_context_default_values() {
    // WindowContext::capture requires an active window, so we test
    // that the struct can be constructed with empty values
    let ctx = WindowContext {
        app_name: None,
        bundle_id: None,
        title: None,
    };

    assert!(ctx.app_name.is_none());
    assert!(ctx.bundle_id.is_none());
    assert!(ctx.title.is_none());
}

#[test]
fn test_window_context_with_values() {
    let ctx = WindowContext {
        app_name: Some("Test App".to_string()),
        bundle_id: Some("com.test.app".to_string()),
        title: Some("Test Window".to_string()),
    };

    assert_eq!(ctx.app_name, Some("Test App".to_string()));
    assert_eq!(ctx.bundle_id, Some("com.test.app".to_string()));
    assert_eq!(ctx.title, Some("Test Window".to_string()));
}

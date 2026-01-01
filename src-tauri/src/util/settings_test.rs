// Tests for the settings module

use super::*;

/// Test helper struct that implements SettingsAccess without an AppHandle
struct NoAppHandle;

impl SettingsAccess for NoAppHandle {
    fn app_handle(&self) -> Option<&AppHandle> {
        None
    }
}

#[test]
fn test_settings_access_returns_none_without_app_handle() {
    let accessor = NoAppHandle;

    // Without an app handle, get_setting should return None
    assert!(accessor.get_setting("any.key").is_none());
}

#[test]
fn test_settings_access_returns_default_settings_file_without_app_handle() {
    let accessor = NoAppHandle;

    // Without an app handle, should fall back to default
    assert_eq!(accessor.settings_file_name(), crate::worktree::DEFAULT_SETTINGS_FILE);
}

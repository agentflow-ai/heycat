// Tests for the util module's public API

use super::*;

#[test]
fn test_run_async_exported() {
    // Verify run_async is accessible through the module
    let result = run_async(async { 42 });
    assert_eq!(result, 42);
}

#[test]
fn test_settings_access_trait_exported() {
    // Verify SettingsAccess trait is accessible through the module
    // We can't instantiate it directly but can reference it in a generic context
    fn takes_settings_access<T: SettingsAccess>(_: &T) {}

    // This just proves the trait is exported and usable
    // Actual functionality is tested in settings_test.rs
}

// Tests for the state_access module

// SettingsHelper and AppHandleSettingsExt require an AppHandle which requires
// a full Tauri runtime. The behavior is tested through integration tests.
// Here we just verify the module compiles correctly and types are accessible.

#[test]
fn test_settings_helper_is_sized() {
    // Verify SettingsHelper can be used with sized bounds
    fn _requires_sized<T: Sized>() {}
    _requires_sized::<super::SettingsHelper>();
}

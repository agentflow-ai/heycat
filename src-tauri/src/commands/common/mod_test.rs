// Tests for the commands/common module's public API

use super::*;

#[test]
fn test_tauri_event_emitter_exported() {
    // Verify TauriEventEmitter is exported through the module
    // This is a compile-time check - we can't instantiate without an AppHandle
    fn _takes_emitter(_: &TauriEventEmitter) {}
}

#[test]
fn test_settings_helper_exported() {
    // Verify SettingsHelper is exported through the module
    fn _takes_helper(_: &SettingsHelper) {}
}

#[test]
fn test_get_settings_file_exported() {
    // Verify get_settings_file is exported
    // This is a compile-time check - we can't call without an AppHandle
    fn _takes_fn(_: fn(&tauri::AppHandle) -> String) {}
    _takes_fn(get_settings_file);
}

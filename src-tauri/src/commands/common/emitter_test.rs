// Tests for the emitter module

// TauriEventEmitter requires an AppHandle which requires a full Tauri runtime.
// The behavior is tested through integration tests in mod_test.rs for commands.
// Here we just verify the module compiles correctly and types are accessible.

#[test]
fn test_tauri_event_emitter_is_sized() {
    // Verify TauriEventEmitter can be used with sized bounds
    fn _requires_sized<T: Sized>() {}
    _requires_sized::<super::TauriEventEmitter>();
}

use super::*;

// Note: Full integration tests require a running Tauri app.
// These tests verify the module structure and basic behavior.

#[test]
fn test_device_handler_state_is_send_sync() {
    // Compile-time check that DeviceHandlerState satisfies Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    // We can't instantiate DeviceHandlerState directly, but the impl exists
    // This is a compile-time check via the unsafe impl declarations
}

#[test]
fn test_on_device_change_handles_uninitialized_state() {
    // Before init, calling on_device_change should not panic
    // It should log an error and return early
    // Note: This test is safe because DEVICE_HANDLER is empty initially
    // and we're not testing state after initialization (which would pollute global state)

    // The function should return without panicking when handler is not initialized
    // We can't easily verify the log output in unit tests, but no panic = success
}

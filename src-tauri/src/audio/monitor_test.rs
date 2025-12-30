use super::*;

#[test]
fn test_audio_monitor_handle_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AudioMonitorHandle>();
}

#[test]
fn test_spawn_and_drop() {
    let handle = AudioMonitorHandle::spawn();
    drop(handle);
    // If we get here without hanging, the Drop impl worked correctly
}

#[test]
fn test_stop_without_start() {
    let handle = AudioMonitorHandle::spawn();
    // Stop when not started should be fine
    assert!(handle.stop().is_ok());
}

#[test]
fn test_shutdown() {
    let handle = AudioMonitorHandle::spawn();
    assert!(handle.shutdown().is_ok());
}

#[test]
fn test_engine_running_query() {
    // Ensure clean state
    swift::audio_engine_stop();

    // Should not be running after stop
    assert!(!swift::audio_engine_is_running(), "Engine should not be running after stop");
}

#[test]
fn test_init_without_device() {
    let handle = AudioMonitorHandle::spawn();
    // Init without device should succeed (pre-warms the engine)
    assert!(handle.init(None).is_ok());
}

#[test]
fn test_init_is_idempotent() {
    let handle = AudioMonitorHandle::spawn();
    // First init
    assert!(handle.init(None).is_ok());
    // Second init should also succeed (engine already running)
    assert!(handle.init(None).is_ok());
}

#[test]
fn test_start_after_init_works() {
    let handle = AudioMonitorHandle::spawn();
    // Pre-warm the engine
    assert!(handle.init(None).is_ok());
    // Start monitoring - should attach to running engine instantly
    let result = handle.start(None);
    assert!(result.is_ok());
    // Clean up
    assert!(handle.stop().is_ok());
}

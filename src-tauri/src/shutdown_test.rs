use super::*;

#[test]
fn test_shutdown_flag_transitions() {
    // Reset for test isolation (note: tests run in parallel, so this is imperfect)
    APP_SHUTTING_DOWN.store(false, Ordering::SeqCst);

    // Initially not shutting down
    assert!(!is_shutting_down());

    // After signal, should be shutting down
    signal_shutdown();
    assert!(is_shutting_down());

    // Should remain true
    assert!(is_shutting_down());
}

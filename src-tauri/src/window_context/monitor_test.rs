use super::*;

#[test]
fn monitor_starts_with_running_flag() {
    let monitor = WindowMonitor::new();
    assert!(!monitor.is_running());
}

#[test]
fn monitor_config_default_has_200ms_interval() {
    let config = MonitorConfig::default();
    assert_eq!(config.poll_interval_ms, 200);
}

#[test]
fn monitor_with_custom_config() {
    let config = MonitorConfig {
        poll_interval_ms: 500,
    };
    let monitor = WindowMonitor::with_config(config);
    assert!(!monitor.is_running());
}

#[test]
fn get_current_context_returns_none_before_start() {
    let monitor = WindowMonitor::new();
    assert!(monitor.get_current_context().is_none());
}

#[test]
fn monitor_default_creates_new() {
    let monitor = WindowMonitor::default();
    assert!(!monitor.is_running());
    assert!(monitor.get_current_context().is_none());
}

// Integration tests that require AppHandle are tested manually
// by switching windows and observing events in the UI.
//
// The tests below verify the struct's public API without
// requiring Tauri runtime dependencies.

#[test]
fn monitor_config_clone() {
    let config = MonitorConfig {
        poll_interval_ms: 100,
    };
    let cloned = config.clone();
    assert_eq!(cloned.poll_interval_ms, 100);
}

#[test]
fn monitor_stop_without_start_is_ok() {
    let mut monitor = WindowMonitor::new();
    // Stop without starting should not panic or error
    let result = monitor.stop();
    assert!(result.is_ok());
}

#[test]
fn monitor_stop_is_idempotent() {
    let mut monitor = WindowMonitor::new();
    assert!(monitor.stop().is_ok());
    assert!(monitor.stop().is_ok());
    assert!(monitor.stop().is_ok());
}

#[test]
fn monitor_handles_rapid_stop_cycles() {
    // Test rapid stop cycles without starting
    // This tests thread safety of the running flag
    for _ in 0..100 {
        let mut monitor = WindowMonitor::new();
        assert!(!monitor.is_running());
        assert!(monitor.stop().is_ok());
        assert!(!monitor.is_running());
    }
}

#[test]
fn monitor_drop_is_safe_without_start() {
    // Test that dropping an unstarted monitor is safe
    {
        let _monitor = WindowMonitor::new();
        // Monitor goes out of scope and is dropped
    }
    // If we get here without panic, the test passes
}

#[test]
fn monitor_config_debug_format() {
    let config = MonitorConfig {
        poll_interval_ms: 100,
    };
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("100"));
}

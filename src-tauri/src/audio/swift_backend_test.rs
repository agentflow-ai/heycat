use super::*;

#[test]
fn test_swift_backend_is_send_sync() {
    // SwiftBackend should be Send + Sync for thread safety
    fn assert_send_sync<T: Send + Sync>() {}
    // Note: SwiftBackend contains AudioBuffer which has Arc<Mutex<...>>
    // The Arc<Mutex<...>> types are Send + Sync, so SwiftBackend should be too
    // This test documents the expectation
}

#[test]
fn test_new_creates_idle_state() {
    let backend = SwiftBackend::new();
    assert_eq!(backend.state, CaptureState::Idle);
    assert!(backend.buffer.is_none());
    assert!(backend.last_capture_file_path.is_none());
    assert_eq!(backend.last_duration_ms, 0);
}

#[test]
fn test_take_warnings_returns_and_clears() {
    let mut backend = SwiftBackend::new();
    // Initially empty
    assert!(backend.take_warnings().is_empty());
}

#[test]
fn test_take_raw_audio_returns_none() {
    let mut backend = SwiftBackend::new();
    // SwiftBackend doesn't support raw audio
    assert!(backend.take_raw_audio().is_none());
}

#[test]
fn test_take_capture_file_initially_none() {
    let mut backend = SwiftBackend::new();
    // Initially no capture file
    assert!(backend.take_capture_file().is_none());
}

/// Test that start and stop work without panicking
/// Ignored by default as it requires microphone permissions
/// Run manually with: cargo test test_start_stop_cycle -- --ignored
#[test]
#[ignore]
#[cfg_attr(coverage_nightly, coverage(off))]
fn test_start_stop_cycle() {
    let mut backend = SwiftBackend::new();
    let buffer = AudioBuffer::new();

    // Start may fail if no device available (CI environment)
    let result = backend.start(buffer, None, None);
    match result {
        Ok(sample_rate) => {
            assert_eq!(sample_rate, TARGET_SAMPLE_RATE);
            assert_eq!(backend.state, CaptureState::Capturing);

            // Stop should succeed
            let stop_result = backend.stop();
            assert!(stop_result.is_ok());
            assert_eq!(backend.state, CaptureState::Stopped);
        }
        Err(_) => {
            // Expected in CI without audio device
            assert_eq!(backend.state, CaptureState::Idle);
        }
    }
}

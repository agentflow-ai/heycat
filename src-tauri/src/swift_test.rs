use super::*;

#[test]
fn test_swift_hello() {
    let result = hello();
    assert_eq!(result, "Hello from Swift!");
}

#[test]
fn test_list_audio_devices_returns_vec() {
    let devices = list_audio_devices();
    // Should return a vector (may be empty if no devices)
    // If devices exist, default should be first
    if !devices.is_empty() && devices.iter().any(|d| d.is_default) {
        assert!(devices[0].is_default, "Default device should be first");
    }
}

/// Test audio engine start/capture/stop cycle
/// Ignored by default as it requires:
/// - Microphone permissions granted to the test process
/// - An actual audio input device
/// Run manually with: cargo test test_audio_engine_capture -- --ignored
#[test]
#[ignore]
#[cfg_attr(coverage_nightly, coverage(off))]
fn test_audio_engine_capture() {
    // Start engine (may fail if no audio device available)
    let result = audio_engine_start(None);

    match result {
        AudioEngineResult::Ok => {
            // Start capture
            match audio_engine_start_capture() {
                AudioEngineResult::Ok => {
                    // Capture a tiny bit
                    std::thread::sleep(std::time::Duration::from_millis(100));

                    // Stop capture and get file path
                    let stop_result = audio_engine_stop_capture();

                    // Duration should be approximately 100ms
                    assert!(stop_result.duration_ms >= 50, "Duration should be at least 50ms");
                    assert!(stop_result.duration_ms <= 500, "Duration should be at most 500ms");

                    // File path should be non-empty
                    assert!(!stop_result.file_path.is_empty(), "File path should be non-empty");

                    // Clean up temp file
                    let _ = std::fs::remove_file(&stop_result.file_path);
                }
                AudioEngineResult::Failed(_) => {
                    // Expected in CI without audio device
                }
            }

            // Stop engine
            audio_engine_stop();
        }
        AudioEngineResult::Failed(_) => {
            // Expected in CI without audio device
        }
    }
}

#[test]
fn test_audio_engine_is_running_query() {
    // Ensure clean state by stopping any running engine
    audio_engine_stop();

    // After stopping, should not be running
    assert!(!audio_engine_is_running(), "Engine should not be running after stop");
}

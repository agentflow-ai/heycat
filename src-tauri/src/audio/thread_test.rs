use super::*;

#[test]
fn test_audio_thread_handle_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AudioThreadHandle>();
}

#[test]
fn test_spawn_and_shutdown() {
    let handle = AudioThreadHandle::spawn();
    assert!(handle.shutdown().is_ok());
}

#[test]
fn test_drop_shuts_down_thread() {
    // Spawn a thread and immediately drop it
    let handle = AudioThreadHandle::spawn();
    drop(handle);
    // If we get here without hanging, the Drop impl worked correctly
}

/// Test that start and stop commands work
/// Excluded from coverage because hardware availability varies
#[test]
#[cfg_attr(coverage_nightly, coverage(off))]
fn test_start_stop_commands() {
    let handle = AudioThreadHandle::spawn();
    let buffer = AudioBuffer::new();

    // Start returns sample rate on success (or CaptureError if no device)
    let result = handle.start_with_device(buffer, None);
    // Either succeeds with sample rate or fails with CaptureError (no device in CI)
    match result {
        Ok(sample_rate) => assert!(sample_rate > 0),
        Err(AudioThreadError::CaptureError(_)) => {} // Expected in CI without audio device
        Err(e) => panic!("Unexpected error: {:?}", e),
    }

    // Stop should succeed
    assert!(handle.stop().is_ok());

    // Shutdown
    assert!(handle.shutdown().is_ok());
}

/// Test AudioCommand::Start includes device_name field
#[test]
fn test_audio_command_start_includes_device() {
    let buffer = AudioBuffer::new();
    let (response_tx, _response_rx) = mpsc::channel::<StartResponse>();

    // Test with Some device name
    let cmd_with_device = AudioCommand::Start {
        buffer: buffer.clone(),
        response_tx: response_tx.clone(),
        device_name: Some("Test Microphone".to_string()),
    };

    // Verify the command can hold device_name (compile-time check)
    match cmd_with_device {
        AudioCommand::Start { device_name, .. } => {
            assert_eq!(device_name, Some("Test Microphone".to_string()));
        }
        _ => panic!("Expected Start command"),
    }

    // Test with None device name
    let (response_tx2, _) = mpsc::channel::<StartResponse>();
    let cmd_without_device = AudioCommand::Start {
        buffer,
        response_tx: response_tx2,
        device_name: None,
    };

    match cmd_without_device {
        AudioCommand::Start { device_name, .. } => {
            assert!(device_name.is_none());
        }
        _ => panic!("Expected Start command"),
    }
}

/// Test start_with_device sends correct command
#[test]
#[cfg_attr(coverage_nightly, coverage(off))]
fn test_start_with_device_passes_device_name() {
    let handle = AudioThreadHandle::spawn();
    let buffer = AudioBuffer::new();

    // Start with a non-existent device - should fall back to default
    let result = handle.start_with_device(buffer, Some("NonExistent Device".to_string()));

    // Either succeeds with sample rate (fallback to default) or fails with CaptureError
    match result {
        Ok(sample_rate) => assert!(sample_rate > 0),
        Err(AudioThreadError::CaptureError(_)) => {} // Expected in CI without audio device
        Err(e) => panic!("Unexpected error: {:?}", e),
    }

    // Stop and shutdown
    let _ = handle.stop();
    assert!(handle.shutdown().is_ok());
}

// test_start_uses_default_device removed: start() method removed (unused convenience wrapper)

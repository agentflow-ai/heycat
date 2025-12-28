use super::*;
use crate::audio::TARGET_SAMPLE_RATE;

/// Test complete recording flow: Idle -> Recording -> Processing -> Idle
/// Verifies buffer creation, data capture, and proper cleanup
#[test]
fn test_complete_recording_flow() {
    let mut manager = RecordingManager::new();
    assert_eq!(manager.get_state(), RecordingState::Idle);

    // Start recording - should create buffer and store sample rate
    let buffer = manager.start_recording(48000).unwrap();
    assert_eq!(manager.get_state(), RecordingState::Recording);
    assert_eq!(manager.get_sample_rate(), Some(48000));

    // Write audio data to buffer
    {
        let mut data = buffer.lock().unwrap();
        data.extend_from_slice(&[0.1, 0.2, 0.3]);
    }

    // Verify data is accessible via manager
    let manager_buffer = manager.get_audio_buffer().unwrap();
    assert_eq!(manager_buffer.lock().unwrap().len(), 3);

    // Transition to processing
    manager.transition_to(RecordingState::Processing).unwrap();
    assert_eq!(manager.get_state(), RecordingState::Processing);

    // Complete processing - return to idle
    manager.transition_to(RecordingState::Idle).unwrap();
    assert_eq!(manager.get_state(), RecordingState::Idle);

    // Active buffer should be cleared but last recording retained
    assert!(manager.get_audio_buffer().is_err());
    let last = manager.get_last_recording_buffer().unwrap();
    assert_eq!(last.samples, vec![0.1, 0.2, 0.3]);
    assert_eq!(last.sample_rate, 48000);
}

/// Test abort discards recording without saving
/// User cancels recording - data should not be retained
#[test]
fn test_abort_discards_recording() {
    let mut manager = RecordingManager::new();
    manager.start_recording(TARGET_SAMPLE_RATE).unwrap();

    // Add audio data
    {
        let buffer = manager.get_audio_buffer().unwrap();
        buffer.lock().unwrap().extend_from_slice(&[0.1, 0.2, 0.3]);
    }

    // Abort - should discard everything
    manager.abort_recording(RecordingState::Idle).unwrap();
    assert_eq!(manager.get_state(), RecordingState::Idle);

    // No data should be retained
    assert!(manager.get_audio_buffer().is_err());
    assert!(manager.get_last_recording_buffer().is_err());
}

/// Test that invalid operations don't corrupt state
/// After invalid transitions, manager remains in valid state
#[test]
fn test_error_recovery() {
    let mut manager = RecordingManager::new();

    // Cannot start processing from idle
    let err = manager.transition_to(RecordingState::Processing).unwrap_err();
    assert!(matches!(err, RecordingStateError::InvalidTransition { .. }));
    assert_eq!(manager.get_state(), RecordingState::Idle);

    // Cannot abort from idle (nothing to abort)
    assert!(manager.abort_recording(RecordingState::Idle).is_err());
    assert_eq!(manager.get_state(), RecordingState::Idle);

    // Start recording
    manager.start_recording(TARGET_SAMPLE_RATE).unwrap();

    // Cannot start another recording while recording
    assert!(manager.start_recording(TARGET_SAMPLE_RATE).is_err());
    assert_eq!(manager.get_state(), RecordingState::Recording);

    // Cannot skip to idle from recording
    assert!(manager.transition_to(RecordingState::Idle).is_err());
    assert_eq!(manager.get_state(), RecordingState::Recording);

    // Valid transition still works after errors
    manager.transition_to(RecordingState::Processing).unwrap();
    assert_eq!(manager.get_state(), RecordingState::Processing);

    // reset_to_idle always works as escape hatch
    manager.reset_to_idle();
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

/// Test that last recording persists across multiple recording sessions
/// New recording replaces previous one
#[test]
fn test_last_recording_persists() {
    let mut manager = RecordingManager::new();

    // No last recording initially
    assert!(manager.get_last_recording_buffer().is_err());

    // First recording
    manager.start_recording(44100).unwrap();
    {
        let buffer = manager.get_audio_buffer().unwrap();
        buffer.lock().unwrap().push(0.1);
    }
    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    // First recording accessible
    let first = manager.get_last_recording_buffer().unwrap();
    assert_eq!(first.samples, vec![0.1]);
    assert_eq!(first.sample_rate, 44100);

    // Second recording replaces first
    manager.start_recording(48000).unwrap();
    {
        let buffer = manager.get_audio_buffer().unwrap();
        buffer.lock().unwrap().extend_from_slice(&[0.9, 0.8]);
    }
    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    // Second recording is now the last
    let second = manager.get_last_recording_buffer().unwrap();
    assert_eq!(second.samples, vec![0.9, 0.8]);
    assert_eq!(second.sample_rate, 48000);

    // Can clear last recording
    manager.clear_last_recording();
    assert!(manager.get_last_recording_buffer().is_err());
}

/// Test duration calculation for AudioData
#[test]
fn test_audio_duration_calculation() {
    let mut manager = RecordingManager::new();
    manager.start_recording(TARGET_SAMPLE_RATE).unwrap();

    // Add 1 second worth of samples (16000 samples at 16kHz)
    {
        let buffer = manager.get_audio_buffer().unwrap();
        buffer.lock().unwrap().extend(std::iter::repeat(0.5f32).take(16000));
    }

    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    let audio = manager.get_last_recording_buffer().unwrap();
    assert!((audio.duration_secs - 1.0).abs() < 0.001);
}

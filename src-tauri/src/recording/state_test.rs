use super::*;
use crate::audio::DEFAULT_SAMPLE_RATE;
use std::sync::Mutex;
use std::thread;

#[test]
fn test_new_manager_starts_idle() {
    let manager = RecordingManager::new();
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

#[test]
fn test_default_manager_starts_idle() {
    let manager = RecordingManager::default();
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

#[test]
fn test_default_state_is_idle() {
    assert_eq!(RecordingState::default(), RecordingState::Idle);
}

#[test]
fn test_start_recording_from_idle() {
    let mut manager = RecordingManager::new();
    let result = manager.start_recording(DEFAULT_SAMPLE_RATE);
    assert!(result.is_ok());
    assert_eq!(manager.get_state(), RecordingState::Recording);
}

#[test]
fn test_start_recording_returns_buffer() {
    let mut manager = RecordingManager::new();
    let buffer = manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    // Verify we can write to the buffer
    {
        let mut data = buffer.lock().unwrap();
        data.push(0.5);
    }
    // Buffer should be the same one in the manager
    let manager_buffer = manager.get_audio_buffer().unwrap();
    let data = manager_buffer.lock().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0], 0.5);
}

#[test]
fn test_start_recording_stores_sample_rate() {
    let mut manager = RecordingManager::new();
    let custom_rate = 48000u32;
    manager.start_recording(custom_rate).unwrap();
    assert_eq!(manager.get_sample_rate(), Some(custom_rate));
}

#[test]
fn test_start_recording_fails_when_already_recording() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();

    let result = manager.start_recording(DEFAULT_SAMPLE_RATE);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        RecordingStateError::InvalidTransition {
            from: RecordingState::Recording,
            to: RecordingState::Recording
        }
    );
}

#[test]
fn test_start_recording_fails_when_processing() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    manager.transition_to(RecordingState::Processing).unwrap();

    let result = manager.start_recording(DEFAULT_SAMPLE_RATE);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        RecordingStateError::InvalidTransition {
            from: RecordingState::Processing,
            to: RecordingState::Recording
        }
    );
}

#[test]
fn test_set_sample_rate_updates_active_recording() {
    let mut manager = RecordingManager::new();
    manager.start_recording(44100).unwrap();
    assert_eq!(manager.get_sample_rate(), Some(44100));

    manager.set_sample_rate(48000);
    assert_eq!(manager.get_sample_rate(), Some(48000));
}

#[test]
fn test_get_sample_rate_returns_none_when_idle() {
    let manager = RecordingManager::new();
    assert_eq!(manager.get_sample_rate(), None);
}

#[test]
fn test_set_sample_rate_does_nothing_when_idle() {
    let mut manager = RecordingManager::new();
    // Should not panic when called in idle state
    manager.set_sample_rate(48000);
    // Sample rate should still be None
    assert_eq!(manager.get_sample_rate(), None);
}

#[test]
fn test_transition_to_recording_is_invalid() {
    // transition_to(Recording) is no longer valid - use start_recording() instead
    let mut manager = RecordingManager::new();
    let result = manager.transition_to(RecordingState::Recording);
    assert!(result.is_err());
}

#[test]
fn test_valid_transition_recording_to_processing() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();

    let result = manager.transition_to(RecordingState::Processing);
    assert!(result.is_ok());
    assert_eq!(manager.get_state(), RecordingState::Processing);
}

#[test]
fn test_valid_transition_processing_to_idle() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    manager.transition_to(RecordingState::Processing).unwrap();

    let result = manager.transition_to(RecordingState::Idle);
    assert!(result.is_ok());
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

#[test]
fn test_full_cycle_idle_recording_processing_idle() {
    let mut manager = RecordingManager::new();

    // Idle -> Recording
    assert!(manager.start_recording(DEFAULT_SAMPLE_RATE).is_ok());
    assert_eq!(manager.get_state(), RecordingState::Recording);

    // Recording -> Processing
    assert!(manager.transition_to(RecordingState::Processing).is_ok());
    assert_eq!(manager.get_state(), RecordingState::Processing);

    // Processing -> Idle
    assert!(manager.transition_to(RecordingState::Idle).is_ok());
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

#[test]
fn test_invalid_transition_idle_to_processing() {
    let mut manager = RecordingManager::new();
    let result = manager.transition_to(RecordingState::Processing);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        RecordingStateError::InvalidTransition {
            from: RecordingState::Idle,
            to: RecordingState::Processing
        }
    );
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

#[test]
fn test_invalid_transition_idle_to_idle() {
    let mut manager = RecordingManager::new();
    let result = manager.transition_to(RecordingState::Idle);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        RecordingStateError::InvalidTransition {
            from: RecordingState::Idle,
            to: RecordingState::Idle
        }
    );
}

#[test]
fn test_invalid_transition_recording_to_idle() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();

    let result = manager.transition_to(RecordingState::Idle);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        RecordingStateError::InvalidTransition {
            from: RecordingState::Recording,
            to: RecordingState::Idle
        }
    );
}

#[test]
fn test_invalid_transition_processing_to_recording() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    manager.transition_to(RecordingState::Processing).unwrap();

    let result = manager.transition_to(RecordingState::Recording);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        RecordingStateError::InvalidTransition {
            from: RecordingState::Processing,
            to: RecordingState::Recording
        }
    );
}

#[test]
fn test_audio_buffer_not_available_in_idle() {
    let manager = RecordingManager::new();
    let result = manager.get_audio_buffer();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), RecordingStateError::NoAudioBuffer);
}

#[test]
fn test_audio_buffer_available_in_recording() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();

    let result = manager.get_audio_buffer();
    assert!(result.is_ok());
}

#[test]
fn test_audio_buffer_available_in_processing() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    manager.transition_to(RecordingState::Processing).unwrap();

    let result = manager.get_audio_buffer();
    assert!(result.is_ok());
}

#[test]
fn test_audio_buffer_cleared_after_processing_to_idle() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    let result = manager.get_audio_buffer();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), RecordingStateError::NoAudioBuffer);
}

#[test]
fn test_audio_buffer_can_be_written_to_during_recording() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();

    let buffer = manager.get_audio_buffer().unwrap();
    {
        let mut data = buffer.lock().unwrap();
        data.push(0.5);
        data.push(-0.5);
    }

    // Verify data was written
    let buffer_again = manager.get_audio_buffer().unwrap();
    let data = buffer_again.lock().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0], 0.5);
    assert_eq!(data[1], -0.5);
}

#[test]
fn test_concurrent_access_with_mutex() {
    let manager = std::sync::Arc::new(Mutex::new(RecordingManager::new()));
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let manager = manager.clone();
            thread::spawn(move || {
                let m = manager.lock().unwrap();
                m.get_state()
            })
        })
        .collect();

    for handle in handles {
        let state = handle.join().unwrap();
        assert_eq!(state, RecordingState::Idle);
    }
}

#[test]
fn test_concurrent_state_transitions() {
    let manager = std::sync::Arc::new(Mutex::new(RecordingManager::new()));

    // Transition to recording in one thread
    {
        let mut m = manager.lock().unwrap();
        m.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    }

    // Multiple threads reading state
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let manager = manager.clone();
            thread::spawn(move || {
                let m = manager.lock().unwrap();
                m.get_state()
            })
        })
        .collect();

    for handle in handles {
        let state = handle.join().unwrap();
        assert_eq!(state, RecordingState::Recording);
    }
}

#[test]
fn test_recording_state_error_display() {
    let error = RecordingStateError::InvalidTransition {
        from: RecordingState::Idle,
        to: RecordingState::Processing,
    };
    let display = format!("{}", error);
    assert!(display.contains("Invalid state transition"));
    assert!(display.contains("Idle"));
    assert!(display.contains("Processing"));

    let error = RecordingStateError::NoAudioBuffer;
    let display = format!("{}", error);
    assert!(display.contains("Audio buffer not available"));
}

#[test]
fn test_recording_state_serialization() {
    let idle = RecordingState::Idle;
    let serialized = serde_json::to_string(&idle).unwrap();
    assert_eq!(serialized, "\"Idle\"");

    let recording = RecordingState::Recording;
    let serialized = serde_json::to_string(&recording).unwrap();
    assert_eq!(serialized, "\"Recording\"");

    let processing = RecordingState::Processing;
    let serialized = serde_json::to_string(&processing).unwrap();
    assert_eq!(serialized, "\"Processing\"");
}

#[test]
fn test_recording_state_clone() {
    let state = RecordingState::Recording;
    let cloned = state.clone();
    assert_eq!(state, cloned);
}

#[test]
fn test_recording_state_copy() {
    let state = RecordingState::Processing;
    let copied: RecordingState = state;
    assert_eq!(state, copied);
}

#[test]
fn test_recording_state_debug() {
    let state = RecordingState::Idle;
    let debug = format!("{:?}", state);
    assert_eq!(debug, "Idle");
}

#[test]
fn test_error_is_std_error() {
    let error: Box<dyn std::error::Error> = Box::new(RecordingStateError::NoAudioBuffer);
    assert!(error.to_string().contains("Audio buffer"));
}

#[test]
fn test_reset_to_idle_from_recording() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    assert!(manager.get_audio_buffer().is_ok());

    manager.reset_to_idle();

    assert_eq!(manager.get_state(), RecordingState::Idle);
    assert!(manager.get_audio_buffer().is_err());
}

#[test]
fn test_reset_to_idle_from_processing() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    manager.transition_to(RecordingState::Processing).unwrap();

    manager.reset_to_idle();

    assert_eq!(manager.get_state(), RecordingState::Idle);
    assert!(manager.get_audio_buffer().is_err());
}

#[test]
fn test_reset_to_idle_from_idle_is_noop() {
    let mut manager = RecordingManager::new();

    manager.reset_to_idle();

    assert_eq!(manager.get_state(), RecordingState::Idle);
}

// =============================================================================
// Last Recording Buffer Tests
// =============================================================================

#[test]
fn test_get_last_recording_buffer_not_available_initially() {
    let manager = RecordingManager::new();
    let result = manager.get_last_recording_buffer();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), RecordingStateError::NoAudioBuffer);
}

#[test]
fn test_last_recording_buffer_retained_after_processing_to_idle() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();

    // Add samples
    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&[0.1, 0.2, 0.3]);
    }

    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    // Last recording buffer should be available
    let audio_data = manager.get_last_recording_buffer().unwrap();
    assert_eq!(audio_data.samples.len(), 3);
    assert_eq!(audio_data.samples[0], 0.1);
    assert_eq!(audio_data.samples[1], 0.2);
    assert_eq!(audio_data.samples[2], 0.3);
}

#[test]
fn test_last_recording_buffer_has_correct_sample_rate() {
    let mut manager = RecordingManager::new();
    let custom_rate = 48000u32;
    manager.start_recording(custom_rate).unwrap();

    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
    }

    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    let audio_data = manager.get_last_recording_buffer().unwrap();
    // Sample rate should be the one we passed to start_recording
    assert_eq!(audio_data.sample_rate, custom_rate);
}

#[test]
fn test_last_recording_buffer_calculates_duration() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();

    // Add 44100 samples (1 second at 44.1kHz)
    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend(std::iter::repeat(0.5f32).take(44100));
    }

    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    let audio_data = manager.get_last_recording_buffer().unwrap();
    assert!((audio_data.duration_secs - 1.0).abs() < 0.001);
}

#[test]
fn test_clear_last_recording_removes_buffer() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();

    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
    }

    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    // Buffer should be available
    assert!(manager.get_last_recording_buffer().is_ok());

    // Clear it
    manager.clear_last_recording();

    // Buffer should not be available
    assert!(manager.get_last_recording_buffer().is_err());
}

#[test]
fn test_new_recording_replaces_last_recording_buffer() {
    let mut manager = RecordingManager::new();

    // First recording
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.1);
    }
    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    // Second recording
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&[0.9, 0.8]);
    }
    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    // Should have the second recording's data
    let audio_data = manager.get_last_recording_buffer().unwrap();
    assert_eq!(audio_data.samples.len(), 2);
    assert_eq!(audio_data.samples[0], 0.9);
    assert_eq!(audio_data.samples[1], 0.8);
}

#[test]
fn test_audio_data_clone() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
    }
    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    let audio_data = manager.get_last_recording_buffer().unwrap();
    let cloned = audio_data.clone();
    assert_eq!(audio_data.samples, cloned.samples);
    assert_eq!(audio_data.sample_rate, cloned.sample_rate);
    assert_eq!(audio_data.duration_secs, cloned.duration_secs);
}

#[test]
fn test_audio_data_debug() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
    }
    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    let audio_data = manager.get_last_recording_buffer().unwrap();
    let debug = format!("{:?}", audio_data);
    assert!(debug.contains("AudioData"));
    assert!(debug.contains("samples"));
    assert!(debug.contains("sample_rate"));
    assert!(debug.contains("duration_secs"));
}

#[test]
fn test_audio_data_serialization() {
    let mut manager = RecordingManager::new();
    manager.start_recording(DEFAULT_SAMPLE_RATE).unwrap();
    {
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&[0.1, 0.2]);
    }
    manager.transition_to(RecordingState::Processing).unwrap();
    manager.transition_to(RecordingState::Idle).unwrap();

    let audio_data = manager.get_last_recording_buffer().unwrap();
    let json = serde_json::to_string(&audio_data).unwrap();
    assert!(json.contains("samples"));
    assert!(json.contains("sample_rate"));
    assert!(json.contains("duration_secs"));
}

// Tests for Tauri IPC commands
#![cfg_attr(coverage_nightly, coverage(off))]

use super::logic::{
    clear_last_recording_buffer_impl, get_last_recording_buffer_impl, get_recording_state_impl,
    start_recording_impl, stop_recording_impl, RecordingStateInfo,
};
use crate::audio::DEFAULT_SAMPLE_RATE;
use crate::recording::{RecordingManager, RecordingState};
use std::sync::Mutex;

// =============================================================================
// Helper to create test state
// =============================================================================

fn create_test_state() -> Mutex<RecordingManager> {
    Mutex::new(RecordingManager::new())
}

// =============================================================================
// get_recording_state_impl Tests
// =============================================================================

#[test]
fn test_get_recording_state_returns_idle_initially() {
    let state = create_test_state();
    let result = get_recording_state_impl(&state);

    assert!(result.is_ok());
    let state_info = result.unwrap();
    assert_eq!(state_info.state, RecordingState::Idle);
}

#[test]
fn test_get_recording_state_returns_recording_after_start() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    let result = get_recording_state_impl(&state);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().state, RecordingState::Recording);
}

#[test]
fn test_recording_state_info_serializes() {
    let state_info = RecordingStateInfo {
        state: RecordingState::Recording,
    };
    let json = serde_json::to_string(&state_info).unwrap();
    assert!(json.contains("Recording"));
}

// =============================================================================
// start_recording_impl Tests
// =============================================================================

#[test]
fn test_start_recording_returns_ok_from_idle() {
    let state = create_test_state();
    let result = start_recording_impl(&state, None);

    assert!(result.is_ok());
}

#[test]
fn test_start_recording_transitions_to_recording() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    let manager = state.lock().unwrap();
    assert_eq!(manager.get_state(), RecordingState::Recording);
}

#[test]
fn test_start_recording_returns_error_when_already_recording() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    let result = start_recording_impl(&state, None);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Already recording"));
}

#[test]
fn test_start_recording_creates_audio_buffer() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    let manager = state.lock().unwrap();
    let buffer_result = manager.get_audio_buffer();
    assert!(buffer_result.is_ok());
}

// =============================================================================
// stop_recording_impl Tests
// =============================================================================

#[test]
fn test_stop_recording_returns_error_when_not_recording() {
    let state = create_test_state();
    let result = stop_recording_impl(&state, None);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not currently recording"));
}

#[test]
fn test_stop_recording_transitions_to_idle() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();
    stop_recording_impl(&state, None).unwrap();

    let manager = state.lock().unwrap();
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

#[test]
fn test_stop_recording_returns_metadata_with_zero_samples() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    let result = stop_recording_impl(&state, None);

    assert!(result.is_ok());
    let metadata = result.unwrap();
    assert_eq!(metadata.sample_count, 0);
    assert_eq!(metadata.duration_secs, 0.0);
    assert!(metadata.file_path.is_empty()); // No file when no samples
}

#[test]
fn test_stop_recording_returns_metadata_with_samples() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    // Add samples to the buffer manually
    // DEFAULT_SAMPLE_RATE is 44100, so 44100 samples = 1 second
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&vec![0.5f32; 44100]); // 1 second at 44.1kHz
    }

    let result = stop_recording_impl(&state, None);

    // Result should be Ok - the actual file is created in the system recordings dir
    // We don't clean it up here to avoid parallel test conflicts
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let metadata = result.unwrap();
    assert_eq!(metadata.sample_count, 44100);
    assert!((metadata.duration_secs - 1.0).abs() < 0.001);
    assert!(metadata.file_path.contains(".wav"));
}

#[test]
fn test_stop_recording_returns_correct_duration() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    // Add 2 seconds of samples at 44.1kHz
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&vec![0.5f32; 88200]); // 2 seconds
    }

    let result = stop_recording_impl(&state, None);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let metadata = result.unwrap();

    assert_eq!(metadata.sample_count, 88200);
    assert!((metadata.duration_secs - 2.0).abs() < 0.001);
}

// =============================================================================
// Full Cycle Tests
// =============================================================================

#[test]
fn test_full_start_stop_cycle() {
    let state = create_test_state();

    // Start
    assert!(start_recording_impl(&state, None).is_ok());
    assert_eq!(
        get_recording_state_impl(&state).unwrap().state,
        RecordingState::Recording
    );

    // Stop
    assert!(stop_recording_impl(&state, None).is_ok());
    assert_eq!(
        get_recording_state_impl(&state).unwrap().state,
        RecordingState::Idle
    );
}

#[test]
fn test_multiple_start_stop_cycles() {
    let state = create_test_state();

    for _ in 0..3 {
        assert!(start_recording_impl(&state, None).is_ok());
        assert!(stop_recording_impl(&state, None).is_ok());
    }

    assert_eq!(
        get_recording_state_impl(&state).unwrap().state,
        RecordingState::Idle
    );
}

// =============================================================================
// RecordingStateInfo Tests
// =============================================================================

#[test]
fn test_recording_state_info_clone() {
    let state_info = RecordingStateInfo {
        state: RecordingState::Processing,
    };
    let cloned = state_info.clone();
    assert_eq!(cloned.state, RecordingState::Processing);
}

#[test]
fn test_recording_state_info_debug() {
    let state_info = RecordingStateInfo {
        state: RecordingState::Idle,
    };
    let debug = format!("{:?}", state_info);
    assert!(debug.contains("state"));
    assert!(debug.contains("Idle"));
}

#[test]
fn test_recording_state_info_all_states_serialize() {
    for state in [
        RecordingState::Idle,
        RecordingState::Recording,
        RecordingState::Processing,
    ] {
        let info = RecordingStateInfo { state };
        let json = serde_json::to_string(&info);
        assert!(json.is_ok());
    }
}

// =============================================================================
// get_last_recording_buffer_impl Tests
// =============================================================================

#[test]
fn test_get_last_recording_buffer_returns_error_when_no_recording() {
    let state = create_test_state();
    let result = get_last_recording_buffer_impl(&state);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Audio buffer not available"));
}

#[test]
fn test_get_last_recording_buffer_available_after_stop() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    // Add samples to the buffer
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&[0.5f32, -0.5f32, 0.25f32]);
    }

    stop_recording_impl(&state, None).unwrap();

    let result = get_last_recording_buffer_impl(&state);
    assert!(result.is_ok());

    let audio_data = result.unwrap();
    assert_eq!(audio_data.samples.len(), 3);
    assert_eq!(audio_data.samples[0], 0.5);
    assert_eq!(audio_data.samples[1], -0.5);
    assert_eq!(audio_data.samples[2], 0.25);
    assert_eq!(audio_data.sample_rate, DEFAULT_SAMPLE_RATE);
}

#[test]
fn test_get_last_recording_buffer_correct_duration() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    // Add 1 second of samples at 44.1kHz
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&vec![0.5f32; 44100]);
    }

    stop_recording_impl(&state, None).unwrap();

    let audio_data = get_last_recording_buffer_impl(&state).unwrap();
    assert!((audio_data.duration_secs - 1.0).abs() < 0.001);
}

#[test]
fn test_get_last_recording_buffer_persists_in_idle() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
    }

    stop_recording_impl(&state, None).unwrap();

    // Confirm state is Idle
    let state_info = get_recording_state_impl(&state).unwrap();
    assert_eq!(state_info.state, RecordingState::Idle);

    // Buffer should still be accessible
    let result = get_last_recording_buffer_impl(&state);
    assert!(result.is_ok());
}

#[test]
fn test_get_last_recording_buffer_updates_on_new_recording() {
    let state = create_test_state();

    // First recording
    start_recording_impl(&state, None).unwrap();
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.1);
    }
    stop_recording_impl(&state, None).unwrap();

    // Second recording with different data
    start_recording_impl(&state, None).unwrap();
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&[0.9, 0.8, 0.7]);
    }
    stop_recording_impl(&state, None).unwrap();

    // Should have the second recording's data
    let audio_data = get_last_recording_buffer_impl(&state).unwrap();
    assert_eq!(audio_data.samples.len(), 3);
    assert_eq!(audio_data.samples[0], 0.9);
}

// =============================================================================
// clear_last_recording_buffer_impl Tests
// =============================================================================

#[test]
fn test_clear_last_recording_buffer_succeeds_when_empty() {
    let state = create_test_state();
    let result = clear_last_recording_buffer_impl(&state);
    assert!(result.is_ok());
}

#[test]
fn test_clear_last_recording_buffer_clears_data() {
    let state = create_test_state();
    start_recording_impl(&state, None).unwrap();

    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
    }

    stop_recording_impl(&state, None).unwrap();

    // Buffer should be available
    assert!(get_last_recording_buffer_impl(&state).is_ok());

    // Clear it
    clear_last_recording_buffer_impl(&state).unwrap();

    // Buffer should no longer be available
    let result = get_last_recording_buffer_impl(&state);
    assert!(result.is_err());
}

#[test]
fn test_clear_last_recording_buffer_allows_new_recording() {
    let state = create_test_state();

    // Record and stop
    start_recording_impl(&state, None).unwrap();
    stop_recording_impl(&state, None).unwrap();

    // Clear
    clear_last_recording_buffer_impl(&state).unwrap();

    // Should be able to record again
    assert!(start_recording_impl(&state, None).is_ok());
    assert!(stop_recording_impl(&state, None).is_ok());
}

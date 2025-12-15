// Tests for Tauri IPC commands
#![cfg_attr(coverage_nightly, coverage(off))]

use super::logic::{
    clear_last_recording_buffer_impl, get_last_recording_buffer_impl, get_recording_state_impl,
    list_recordings_impl, start_recording_impl, stop_recording_impl, RecordingInfo,
    RecordingStateInfo,
};
use crate::audio::TARGET_SAMPLE_RATE;
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
    start_recording_impl(&state, None, true).unwrap();

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
    let result = start_recording_impl(&state, None, true);

    assert!(result.is_ok());
}

#[test]
fn test_start_recording_transitions_to_recording() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    let manager = state.lock().unwrap();
    assert_eq!(manager.get_state(), RecordingState::Recording);
}

#[test]
fn test_start_recording_returns_error_when_already_recording() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    let result = start_recording_impl(&state, None, true);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already recording"));
}

#[test]
fn test_start_recording_creates_audio_buffer() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

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
    let result = stop_recording_impl(&state, None, false);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No recording in progress"));
}

#[test]
fn test_stop_recording_transitions_to_idle() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();
    stop_recording_impl(&state, None, false).unwrap();

    let manager = state.lock().unwrap();
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

#[test]
fn test_stop_recording_transitions_to_listening_when_enabled() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();
    stop_recording_impl(&state, None, true).unwrap();

    let manager = state.lock().unwrap();
    assert_eq!(manager.get_state(), RecordingState::Listening);
}

#[test]
fn test_stop_recording_returns_metadata_with_zero_samples() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    let result = stop_recording_impl(&state, None, false);

    assert!(result.is_ok());
    let metadata = result.unwrap();
    assert_eq!(metadata.sample_count, 0);
    assert_eq!(metadata.duration_secs, 0.0);
    assert!(metadata.file_path.is_empty()); // No file when no samples
}

#[test]
fn test_stop_recording_returns_metadata_with_samples() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    // Add samples to the buffer manually
    // TARGET_SAMPLE_RATE is 16000, so 16000 samples = 1 second
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&vec![0.5f32; 16000]); // 1 second at 16kHz
    }

    let result = stop_recording_impl(&state, None, false);

    // Result should be Ok - the actual file is created in the system recordings dir
    // We don't clean it up here to avoid parallel test conflicts
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let metadata = result.unwrap();
    assert_eq!(metadata.sample_count, 16000);
    assert!((metadata.duration_secs - 1.0).abs() < 0.001);
    assert!(metadata.file_path.contains(".wav"));
}

#[test]
fn test_stop_recording_returns_correct_duration() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    // Add 2 seconds of samples at 16kHz
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&vec![0.5f32; 32000]); // 2 seconds at 16kHz
    }

    let result = stop_recording_impl(&state, None, false);
    assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    let metadata = result.unwrap();

    assert_eq!(metadata.sample_count, 32000);
    assert!((metadata.duration_secs - 2.0).abs() < 0.001);
}

// =============================================================================
// Full Cycle Tests
// =============================================================================

#[test]
fn test_full_start_stop_cycle() {
    let state = create_test_state();

    // Start
    assert!(start_recording_impl(&state, None, true).is_ok());
    assert_eq!(
        get_recording_state_impl(&state).unwrap().state,
        RecordingState::Recording
    );

    // Stop
    assert!(stop_recording_impl(&state, None, false).is_ok());
    assert_eq!(
        get_recording_state_impl(&state).unwrap().state,
        RecordingState::Idle
    );
}

#[test]
fn test_multiple_start_stop_cycles() {
    let state = create_test_state();

    for _ in 0..3 {
        assert!(start_recording_impl(&state, None, true).is_ok());
        assert!(stop_recording_impl(&state, None, false).is_ok());
    }

    assert_eq!(
        get_recording_state_impl(&state).unwrap().state,
        RecordingState::Idle
    );
}

// =============================================================================
// get_last_recording_buffer_impl Tests
// =============================================================================

#[test]
fn test_get_last_recording_buffer_returns_error_when_no_recording() {
    let state = create_test_state();
    let result = get_last_recording_buffer_impl(&state);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No recording available"));
}

#[test]
fn test_get_last_recording_buffer_available_after_stop() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    // Add samples to the buffer
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&[0.5f32, -0.5f32, 0.25f32]);
    }

    stop_recording_impl(&state, None, false).unwrap();

    let result = get_last_recording_buffer_impl(&state);
    assert!(result.is_ok());

    let audio_data = result.unwrap();
    assert_eq!(audio_data.samples.len(), 3);
    assert_eq!(audio_data.samples[0], 0.5);
    assert_eq!(audio_data.samples[1], -0.5);
    assert_eq!(audio_data.samples[2], 0.25);
    assert_eq!(audio_data.sample_rate, TARGET_SAMPLE_RATE);
}

#[test]
fn test_get_last_recording_buffer_correct_duration() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    // Add 1 second of samples at 16kHz
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&vec![0.5f32; 16000]);
    }

    stop_recording_impl(&state, None, false).unwrap();

    let audio_data = get_last_recording_buffer_impl(&state).unwrap();
    assert!((audio_data.duration_secs - 1.0).abs() < 0.001);
}

#[test]
fn test_get_last_recording_buffer_persists_in_idle() {
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
    }

    stop_recording_impl(&state, None, false).unwrap();

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
    start_recording_impl(&state, None, true).unwrap();
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.1);
    }
    stop_recording_impl(&state, None, false).unwrap();

    // Second recording with different data
    start_recording_impl(&state, None, true).unwrap();
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&[0.9, 0.8, 0.7]);
    }
    stop_recording_impl(&state, None, false).unwrap();

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
    start_recording_impl(&state, None, true).unwrap();

    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.push(0.5);
    }

    stop_recording_impl(&state, None, false).unwrap();

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
    start_recording_impl(&state, None, true).unwrap();
    stop_recording_impl(&state, None, false).unwrap();

    // Clear
    clear_last_recording_buffer_impl(&state).unwrap();

    // Should be able to record again
    assert!(start_recording_impl(&state, None, true).is_ok());
    assert!(stop_recording_impl(&state, None, false).is_ok());
}

// =============================================================================
// list_recordings_impl Tests
// =============================================================================

// Note: list_recordings_impl reads from the actual recordings directory.
// These tests verify the function works correctly with the system directory.
// Since tests may run in parallel, we focus on testing that the function
// doesn't error and returns a valid result.

#[test]
fn test_list_recordings_returns_ok() {
    // This test verifies list_recordings_impl doesn't panic or error
    // even if the directory doesn't exist yet
    let result = list_recordings_impl();
    assert!(result.is_ok());
}

#[test]
fn test_list_recordings_returns_vec() {
    let result = list_recordings_impl();
    assert!(result.is_ok());
    // Should return a Vec (empty or with recordings)
    let recordings = result.unwrap();
    // All items should have non-empty filenames
    for recording in &recordings {
        assert!(!recording.filename.is_empty());
        assert!(!recording.file_path.is_empty());
        assert!(recording.filename.ends_with(".wav"));
    }
}

#[test]
fn test_recording_info_struct_serializes() {
    let info = RecordingInfo {
        filename: "test.wav".to_string(),
        file_path: "/path/to/test.wav".to_string(),
        duration_secs: 1.5,
        created_at: "2025-01-01T00:00:00Z".to_string(),
        file_size_bytes: 1024,
        error: None,
    };
    let json = serde_json::to_string(&info);
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("test.wav"));
    assert!(json_str.contains("1.5"));
    assert!(json_str.contains("1024"));
}

#[test]
fn test_list_recordings_after_stop_recording() {
    // After creating a recording via stop_recording_impl, list_recordings should find it
    let state = create_test_state();
    start_recording_impl(&state, None, true).unwrap();

    // Add samples to create a valid recording
    {
        let manager = state.lock().unwrap();
        let buffer = manager.get_audio_buffer().unwrap();
        let mut guard = buffer.lock().unwrap();
        guard.extend_from_slice(&vec![0.5f32; 44100]); // 1 second
    }

    let metadata = stop_recording_impl(&state, None, false).unwrap();

    // Now list should include at least this recording
    let result = list_recordings_impl();
    assert!(result.is_ok());
    let recordings = result.unwrap();

    // Find our recording by path
    let found = recordings
        .iter()
        .any(|r| r.file_path == metadata.file_path);
    assert!(found, "Created recording should be in list");
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_recording_info_with_error_serializes() {
    let info = RecordingInfo {
        filename: "corrupt.wav".to_string(),
        file_path: "/path/to/corrupt.wav".to_string(),
        duration_secs: 0.0,
        created_at: String::new(),
        file_size_bytes: 0,
        error: Some("Corrupt audio file".to_string()),
    };
    let json = serde_json::to_string(&info);
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("corrupt.wav"));
    assert!(json_str.contains("Corrupt audio file"));
}

#[test]
fn test_recording_info_without_error_omits_field() {
    let info = RecordingInfo {
        filename: "test.wav".to_string(),
        file_path: "/path/to/test.wav".to_string(),
        duration_secs: 1.0,
        created_at: "2025-01-01T00:00:00Z".to_string(),
        file_size_bytes: 1024,
        error: None,
    };
    let json = serde_json::to_string(&info).unwrap();
    // Error field should be omitted when None due to skip_serializing_if
    assert!(!json.contains("error"));
}

// =============================================================================
// Model Availability Tests
// =============================================================================

#[test]
fn test_start_recording_returns_error_when_model_not_available() {
    let state = create_test_state();
    let result = start_recording_impl(&state, None, false);

    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    assert!(
        error_msg.contains("download the transcription model"),
        "Expected user-friendly model error, got: {}",
        error_msg
    );
}

#[test]
fn test_start_recording_succeeds_when_model_is_available() {
    let state = create_test_state();
    let result = start_recording_impl(&state, None, true);

    assert!(result.is_ok());
}

#[test]
fn test_start_recording_model_error_message_is_user_friendly() {
    let state = create_test_state();
    let result = start_recording_impl(&state, None, false);

    let error_msg = result.unwrap_err();
    // Verify the exact user-friendly message
    assert_eq!(
        error_msg,
        "Please download the transcription model first."
    );
}

#[test]
fn test_start_recording_model_check_comes_before_state_check() {
    // Even if recording state would allow recording, model check should fail first
    let state = create_test_state();

    // Verify we're in Idle state (would normally allow starting)
    assert_eq!(
        get_recording_state_impl(&state).unwrap().state,
        RecordingState::Idle
    );

    // Model not available should fail before state is checked
    let result = start_recording_impl(&state, None, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("download the transcription model"));

    // State should remain Idle (no transition attempted)
    assert_eq!(
        get_recording_state_impl(&state).unwrap().state,
        RecordingState::Idle
    );
}


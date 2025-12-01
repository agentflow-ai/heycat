// Command implementation logic - testable functions separate from Tauri wrappers

use crate::audio::{encode_wav, AudioThreadHandle, SystemFileWriter, DEFAULT_SAMPLE_RATE};
use crate::recording::{AudioData, RecordingManager, RecordingMetadata, RecordingState};
use serde::Serialize;
use std::sync::Mutex;

/// Information about the current recording state for frontend consumption
#[derive(Debug, Clone, Serialize)]
pub struct RecordingStateInfo {
    /// Current state (Idle, Recording, Processing)
    pub state: RecordingState,
}

/// Implementation of start_recording
///
/// # Arguments
/// * `state` - The recording manager state
/// * `audio_thread` - Optional audio thread handle for starting capture
///
/// # Errors
/// Returns an error string if:
/// - Already recording
/// - State transition fails
/// - Audio capture fails to start
/// - State lock is poisoned
pub fn start_recording_impl(
    state: &Mutex<RecordingManager>,
    audio_thread: Option<&AudioThreadHandle>,
) -> Result<(), String> {
    let mut manager = state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    // Check current state
    if manager.get_state() != RecordingState::Idle {
        return Err("Already recording".to_string());
    }

    // Start recording with default sample rate
    let buffer = manager
        .start_recording(DEFAULT_SAMPLE_RATE)
        .map_err(|e| e.to_string())?;

    // Start audio capture if audio thread is available
    if let Some(audio_thread) = audio_thread {
        match audio_thread.start(buffer) {
            Ok(sample_rate) => {
                // Update with actual sample rate from device
                manager.set_sample_rate(sample_rate);
            }
            Err(e) => {
                // Audio capture failed - rollback state and return error
                manager.reset_to_idle();
                return Err(format!("Audio capture failed: {:?}", e));
            }
        }
    }

    Ok(())
}

/// Implementation of stop_recording
///
/// # Arguments
/// * `state` - The recording manager state
/// * `audio_thread` - Optional audio thread handle for stopping capture
///
/// # Returns
/// Recording metadata including duration, file path, and sample count
///
/// # Errors
/// Returns an error string if:
/// - Not currently recording
/// - State transition fails
/// - WAV encoding fails
/// - State lock is poisoned
pub fn stop_recording_impl(
    state: &Mutex<RecordingManager>,
    audio_thread: Option<&AudioThreadHandle>,
) -> Result<RecordingMetadata, String> {
    let mut manager = state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;

    // Check current state
    if manager.get_state() != RecordingState::Recording {
        return Err("Not currently recording".to_string());
    }

    // Stop audio capture if audio thread is available
    if let Some(audio_thread) = audio_thread {
        let _ = audio_thread.stop(); // Best effort, ignore errors
    }

    // Get the actual sample rate before transitioning
    let sample_rate = manager.get_sample_rate().unwrap_or(DEFAULT_SAMPLE_RATE);

    // Transition to Processing
    manager
        .transition_to(RecordingState::Processing)
        .map_err(|e| e.to_string())?;

    // Get the audio buffer and encode
    let buffer = manager.get_audio_buffer().map_err(|e| e.to_string())?;
    let samples = buffer.lock().map_err(|e| e.to_string())?.clone();
    let sample_count = samples.len();

    // Encode WAV if we have samples
    let file_path = if !samples.is_empty() {
        let writer = SystemFileWriter;
        encode_wav(&samples, sample_rate, &writer)
            .map_err(|e| format!("Encoding error: {:?}", e))?
    } else {
        // No samples recorded - return placeholder
        String::new()
    };

    // Calculate duration using actual sample rate
    let duration_secs = sample_count as f64 / sample_rate as f64;

    // Transition to Idle
    manager
        .transition_to(RecordingState::Idle)
        .map_err(|e| e.to_string())?;

    Ok(RecordingMetadata {
        duration_secs,
        file_path,
        sample_count,
    })
}

/// Implementation of get_recording_state
///
/// # Returns
/// Current state information for the frontend
///
/// # Errors
/// Returns an error string if the state lock is poisoned
pub fn get_recording_state_impl(
    state: &Mutex<RecordingManager>,
) -> Result<RecordingStateInfo, String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(RecordingStateInfo {
        state: manager.get_state(),
    })
}

/// Implementation of get_last_recording_buffer
///
/// # Returns
/// Audio data from the most recent completed recording
///
/// # Errors
/// Returns an error string if:
/// - No previous recording exists
/// - State lock is poisoned
pub fn get_last_recording_buffer_impl(
    state: &Mutex<RecordingManager>,
) -> Result<AudioData, String> {
    let manager = state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    manager
        .get_last_recording_buffer()
        .map_err(|e| e.to_string())
}

/// Implementation of clear_last_recording_buffer
///
/// Clears the retained recording buffer to free memory
///
/// # Errors
/// Returns an error string if the state lock is poisoned
pub fn clear_last_recording_buffer_impl(state: &Mutex<RecordingManager>) -> Result<(), String> {
    let mut manager = state
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    manager.clear_last_recording();
    Ok(())
}

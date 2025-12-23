// Command implementation logic - testable functions separate from Tauri wrappers

use crate::audio::{
    encode_wav, parse_duration_from_file, AudioThreadHandle, SharedDenoiser, SystemFileWriter,
    TARGET_SAMPLE_RATE,
};
use std::sync::Arc;
use crate::recording::{AudioData, RecordingManager, RecordingMetadata, RecordingState};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

/// Information about a single recording for frontend consumption
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecordingInfo {
    /// Filename of the recording (e.g., "recording-2025-12-01-143025.wav")
    pub filename: String,
    /// Full path to the recording file
    pub file_path: String,
    /// Duration of the recording in seconds
    pub duration_secs: f64,
    /// Creation timestamp in ISO 8601 format
    pub created_at: String,
    /// File size in bytes
    pub file_size_bytes: u64,
    /// Error message if the recording has issues (missing file, corrupt metadata)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

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
/// * `model_available` - Whether the transcription model is available
/// * `device_name` - Optional device name to use; falls back to default if not found
/// * `shared_denoiser` - Optional shared denoiser for noise suppression (loaded at app startup)
///
/// # Errors
/// Returns an error string if:
/// - Transcription model not available
/// - Already recording
/// - State transition fails
/// - Audio capture fails to start
/// - State lock is poisoned
pub fn start_recording_impl(
    state: &Mutex<RecordingManager>,
    audio_thread: Option<&AudioThreadHandle>,
    model_available: bool,
    device_name: Option<String>,
    shared_denoiser: Option<Arc<SharedDenoiser>>,
) -> Result<(), String> {
    crate::debug!(
        "start_recording_impl called, model_available={}, device={:?}",
        model_available, device_name
    );

    // Check model availability first
    if !model_available {
        crate::debug!("Recording rejected: model not available");
        return Err("Please download the transcription model first.".to_string());
    }
    let mut manager = state.lock().map_err(|_| {
        crate::error!("Failed to acquire recording state lock in start_recording_impl");
        "Unable to access recording state. Please try again or restart the application."
    })?;

    // Check current state - allow starting from Idle or Listening (for wake word activation)
    let current_state = manager.get_state();
    crate::debug!("Current recording state: {:?}", current_state);
    if current_state != RecordingState::Idle && current_state != RecordingState::Listening {
        crate::debug!("Recording rejected: already in {:?} state", current_state);
        return Err(
            "Cannot start recording: already recording or processing.".to_string(),
        );
    }

    // Start recording with default sample rate
    let buffer = manager
        .start_recording(TARGET_SAMPLE_RATE)
        .map_err(|e| {
            crate::error!("Failed to start recording: {:?}", e);
            "Failed to initialize recording."
        })?;
    crate::debug!("Recording buffer initialized");

    // Start audio capture if audio thread is available
    if let Some(audio_thread) = audio_thread {
        match audio_thread.start_with_device_and_denoiser(buffer, device_name, shared_denoiser) {
            Ok(sample_rate) => {
                // Update with actual sample rate from device
                manager.set_sample_rate(sample_rate);
                crate::info!("Audio capture started at {}Hz", sample_rate);
            }
            Err(e) => {
                // Audio capture failed - rollback state and return error
                crate::error!("Audio capture failed: {:?}", e);
                manager.reset_to_idle();
                return Err(
                    "Could not access the microphone. Please check that your microphone is connected and permissions are granted.".to_string(),
                );
            }
        }
    } else {
        crate::debug!("No audio thread available, recording without capture");
    }

    crate::info!("Recording started successfully");
    Ok(())
}

/// Implementation of stop_recording
///
/// # Arguments
/// * `state` - The recording manager state
/// * `audio_thread` - Optional audio thread handle for stopping capture
/// * `return_to_listening` - If true, return to Listening state instead of Idle
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
    return_to_listening: bool,
) -> Result<RecordingMetadata, String> {
    crate::debug!("stop_recording_impl called");

    let mut manager = state.lock().map_err(|_| {
        crate::error!("Failed to acquire recording state lock in stop_recording_impl");
        "Unable to access recording state. Please try again or restart the application."
    })?;

    // Check current state
    let current_state = manager.get_state();
    crate::debug!("Current recording state: {:?}", current_state);
    if current_state != RecordingState::Recording {
        crate::debug!("Stop rejected: not in Recording state");
        return Err("No recording in progress. Start a recording first.".to_string());
    }

    // Stop audio capture if audio thread is available
    let stop_result = if let Some(audio_thread) = audio_thread {
        crate::debug!("Stopping audio thread");
        match audio_thread.stop() {
            Ok(result) => Some(result),
            Err(e) => {
                crate::error!("Audio thread stop failed: {:?}", e);
                // Continue with recording stop - we can't "unstop", but log the error
                None
            }
        }
    } else {
        crate::debug!("No audio thread to stop");
        None
    };

    // Get the actual sample rate before transitioning
    let sample_rate = manager.get_sample_rate().unwrap_or(TARGET_SAMPLE_RATE);
    crate::debug!("Sample rate: {}Hz", sample_rate);

    // Transition to Processing
    manager
        .transition_to(RecordingState::Processing)
        .map_err(|e| {
            crate::error!("Failed to transition to Processing: {:?}", e);
            "Failed to process recording."
        })?;
    crate::debug!("Transitioned to Processing state");

    // Get the audio buffer and encode
    let buffer = manager
        .get_audio_buffer()
        .map_err(|_| "No recorded audio available.")?;

    // CRITICAL: Drain ring buffer to accumulated before encoding
    // The audio callback pushes to the lock-free ring buffer, but lock() only
    // returns the accumulated Vec. Without draining, samples stay in ring buffer.
    let drained = buffer.drain_samples();
    crate::debug!("Drained {} samples from ring buffer before WAV encoding", drained.len());

    // Clone samples and release lock before encoding - we can't hold the lock
    // across the WAV encoding I/O operation (it would block other threads)
    let samples = buffer
        .lock()
        .map_err(|_| "Unable to access recorded audio.")?
        .clone();
    let sample_count = samples.len();
    crate::debug!("Got {} samples from buffer", sample_count);

    // Encode WAV if we have samples
    let file_path = if !samples.is_empty() {
        let writer = SystemFileWriter;
        let path = encode_wav(&samples, sample_rate, &writer)
            .map_err(|e| {
                crate::error!("WAV encoding failed: {:?}", e);
                "Failed to save the recording. Please check disk space and try again."
            })?;
        crate::debug!("WAV encoded to: {}", path);
        path
    } else {
        crate::debug!("No samples to encode");
        // No samples recorded - return placeholder
        String::new()
    };

    // Calculate duration using actual sample rate
    let duration_secs = sample_count as f64 / sample_rate as f64;

    // Transition to Listening (if enabled) or Idle
    let target_state = if return_to_listening {
        RecordingState::Listening
    } else {
        RecordingState::Idle
    };
    manager
        .transition_to(target_state)
        .map_err(|e| {
            crate::error!("Failed to transition to {:?}: {:?}", target_state, e);
            "Failed to complete recording."
        })?;

    // Extract stop reason from result
    let stop_reason = stop_result.and_then(|r| r.reason);

    crate::info!("Recording stopped: {} samples, {:.2}s, stop_reason={:?}",
          sample_count, duration_secs, stop_reason);

    Ok(RecordingMetadata {
        duration_secs,
        file_path,
        sample_count,
        stop_reason,
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
    let manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;
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
    let manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;
    manager.get_last_recording_buffer().map_err(|_| {
        "No recording available. Please make a recording first.".to_string()
    })
}

/// Implementation of clear_last_recording_buffer
///
/// Clears the retained recording buffer to free memory
///
/// # Errors
/// Returns an error string if the state lock is poisoned
pub fn clear_last_recording_buffer_impl(state: &Mutex<RecordingManager>) -> Result<(), String> {
    let mut manager = state.lock().map_err(|_| {
        "Unable to access recording state. Please try again or restart the application."
    })?;
    manager.clear_last_recording();
    Ok(())
}

/// Get the recordings directory path with worktree context
///
/// Uses the same path as SystemFileWriter for consistency
fn get_recordings_dir_with_context(
    worktree_context: Option<&crate::worktree::WorktreeContext>,
) -> PathBuf {
    crate::paths::get_recordings_dir(worktree_context)
        .unwrap_or_else(|_| PathBuf::from(".").join("heycat").join("recordings"))
}

/// Get the recordings directory path (API-compatible, uses main repo path)
fn get_recordings_dir() -> PathBuf {
    get_recordings_dir_with_context(None)
}

/// Implementation of list_recordings
///
/// Lists all recordings from the app data directory with their metadata.
///
/// # Returns
/// A list of RecordingInfo sorted by creation time (newest first).
/// Returns an empty list if the recordings directory doesn't exist or is empty.
///
/// # Errors
/// Only returns an error if there's a critical system failure.
/// Individual file errors are logged and the file is skipped.
pub fn list_recordings_impl() -> Result<Vec<RecordingInfo>, String> {
    let recordings_dir = get_recordings_dir();

    // Return empty list if directory doesn't exist (not an error)
    if !recordings_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&recordings_dir).map_err(|e| {
        crate::error!("Failed to read recordings directory: {}", e);
        format!("Unable to access recordings directory: {}", e)
    })?;

    let mut recordings: Vec<RecordingInfo> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                crate::error!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Only process .wav files
        if path.extension().and_then(|s| s.to_str()) != Some("wav") {
            continue;
        }

        // Get filename - skip if we can't even get the filename
        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(name) => name.to_string(),
            None => {
                crate::error!("Invalid filename for {}", path.display());
                continue;
            }
        };

        let file_path_str = path.to_string_lossy().to_string();

        // Track errors for this recording
        let mut recording_error: Option<String> = None;

        // Get file metadata
        let (file_size_bytes, created_at) = match std::fs::metadata(&path) {
            Ok(metadata) => {
                let size = metadata.len();
                let created = metadata
                    .created()
                    .or_else(|_| metadata.modified())
                    .map(|t| {
                        let datetime: DateTime<Utc> = t.into();
                        datetime.to_rfc3339()
                    })
                    .unwrap_or_else(|e| {
                        crate::error!("Failed to get creation time for {}: {}", path.display(), e);
                        let err_msg = "Missing creation date";
                        recording_error = Some(err_msg.to_string());
                        String::new()
                    });
                (size, created)
            }
            Err(e) => {
                crate::error!("Failed to read metadata for {}: {}", path.display(), e);
                recording_error = Some(format!("Cannot read file metadata: {}", e));
                (0, String::new())
            }
        };

        // Parse duration from WAV header
        let duration_secs = match parse_duration_from_file(&path) {
            Ok(d) => d,
            Err(e) => {
                crate::error!(
                    "Failed to parse duration for {}: {:?}",
                    path.display(),
                    e
                );
                // Set error but include the recording with 0 duration
                let err_msg = format!("Corrupt audio file: {:?}", e);
                recording_error = Some(err_msg);
                0.0
            }
        };

        recordings.push(RecordingInfo {
            filename,
            file_path: file_path_str,
            duration_secs,
            created_at,
            file_size_bytes,
            error: recording_error,
        });
    }

    // Sort by created_at descending (newest first)
    recordings.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(recordings)
}

/// Implementation of delete_recording
///
/// Deletes a recording file from the filesystem.
///
/// # Arguments
/// * `file_path` - Path to the recording file to delete
///
/// # Returns
/// Ok(()) on success
///
/// # Errors
/// Returns an error string if:
/// - File does not exist
/// - File is not in the recordings directory (security check)
/// - Deletion fails
pub fn delete_recording_impl(file_path: &str) -> Result<(), String> {
    let path = std::path::Path::new(file_path);

    // Check if file exists
    if !path.exists() {
        return Err(format!("Recording file not found: {}", file_path));
    }

    // Security check: ensure file is in recordings directory
    let recordings_dir = get_recordings_dir();
    let canonical_path = path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    let canonical_recordings = recordings_dir
        .canonicalize()
        .unwrap_or_else(|_| recordings_dir.clone());

    if !canonical_path.starts_with(&canonical_recordings) {
        crate::error!(
            "Security: Attempted to delete file outside recordings directory: {}",
            file_path
        );
        return Err("Cannot delete files outside the recordings directory".to_string());
    }

    // Check it's a .wav file
    if path.extension().and_then(|s| s.to_str()) != Some("wav") {
        return Err("Can only delete .wav recording files".to_string());
    }

    // Delete the file
    std::fs::remove_file(path).map_err(|e| {
        crate::error!("Failed to delete recording {}: {}", file_path, e);
        format!("Failed to delete recording: {}", e)
    })?;

    crate::info!("Deleted recording: {}", file_path);
    Ok(())
}

/// Implementation of transcribe_file
///
/// Transcribes an audio file using the TDT (batch) model.
///
/// # Arguments
/// * `shared_model` - The shared transcription model state
/// * `file_path` - Path to the audio file to transcribe
///
/// # Returns
/// The transcribed text
///
/// # Errors
/// Returns an error string if:
/// - TDT model is not loaded
/// - File does not exist
/// - Transcription fails
pub fn transcribe_file_impl(
    shared_model: &crate::parakeet::SharedTranscriptionModel,
    file_path: &str,
) -> Result<String, String> {
    use crate::parakeet::TranscriptionService;

    crate::debug!("transcribe_file_impl called for: {}", file_path);

    // Check if TDT model is loaded
    if !shared_model.is_loaded() {
        return Err("Please download the Batch transcription model first.".to_string());
    }

    // Check if file exists
    if !std::path::Path::new(file_path).exists() {
        return Err(format!("Recording file not found: {}", file_path));
    }

    // Perform transcription
    let text = shared_model
        .transcribe(file_path)
        .map_err(|e| format!("Transcription failed: {}", e))?;

    crate::info!("Transcription complete: {} characters", text.len());
    Ok(text)
}

// =============================================================================
// Listening Commands
// =============================================================================

use crate::events::ListeningEventEmitter;
use crate::listening::{ListeningManager, ListeningPipeline, ListeningStatus};

/// Implementation of enable_listening
///
/// Enables listening mode, transitions to Listening state, and starts the pipeline.
///
/// # Arguments
/// * `listening_manager` - The listening manager state
/// * `recording_manager` - The recording manager state
/// * `listening_pipeline` - The listening pipeline state
/// * `audio_thread` - The audio thread handle for capturing audio
/// * `emitter` - Event emitter for wake word detection events
///
/// # Errors
/// Returns an error string if:
/// - Currently recording
/// - State transition fails
/// - Pipeline fails to start
/// - State lock is poisoned
pub fn enable_listening_impl<E: ListeningEventEmitter + 'static>(
    listening_manager: &Mutex<ListeningManager>,
    recording_manager: &Mutex<RecordingManager>,
    listening_pipeline: &Mutex<ListeningPipeline>,
    audio_thread: &AudioThreadHandle,
    emitter: std::sync::Arc<E>,
    device_name: Option<String>,
) -> Result<(), String> {
    crate::debug!("enable_listening_impl called");

    // First, enable listening mode and transition state
    {
        let mut lm = listening_manager.lock().map_err(|_| {
            crate::error!("Failed to acquire listening manager lock");
            "Unable to access listening state. Please try again."
        })?;

        lm.enable_listening(recording_manager).map_err(|e| {
            crate::error!("Failed to enable listening: {}", e);
            match e {
                crate::listening::ListeningError::RecordingInProgress => {
                    "Cannot enable listening while recording. Stop the recording first.".to_string()
                }
                crate::listening::ListeningError::LockError => {
                    "Unable to access recording state. Please try again.".to_string()
                }
                _ => format!("Failed to enable listening: {}", e),
            }
        })?;
    }

    // Start the listening pipeline
    {
        let mut pipeline = listening_pipeline.lock().map_err(|_| {
            crate::error!("Failed to acquire listening pipeline lock");
            "Unable to access listening pipeline. Please try again."
        })?;

        // Only start if not already running
        if !pipeline.is_running() {
            pipeline
                .start_with_device(audio_thread, emitter, device_name)
                .map_err(|e| {
                    crate::error!("Failed to start listening pipeline: {}", e);
                    format!("Failed to start listening: {}", e)
                })?;
            crate::info!("Listening pipeline started");
        }
    }

    crate::info!("Listening mode enabled");
    Ok(())
}

/// Implementation of disable_listening
///
/// Disables listening mode, stops the pipeline, and transitions to Idle state.
///
/// # Arguments
/// * `listening_manager` - The listening manager state
/// * `recording_manager` - The recording manager state
/// * `listening_pipeline` - The listening pipeline state
/// * `audio_thread` - The audio thread handle for stopping capture
///
/// # Errors
/// Returns an error string if:
/// - State transition fails
/// - State lock is poisoned
pub fn disable_listening_impl(
    listening_manager: &Mutex<ListeningManager>,
    recording_manager: &Mutex<RecordingManager>,
    listening_pipeline: &Mutex<ListeningPipeline>,
    audio_thread: &AudioThreadHandle,
) -> Result<(), String> {
    crate::debug!("disable_listening_impl called");

    // Stop the listening pipeline first
    {
        let mut pipeline = listening_pipeline.lock().map_err(|_| {
            crate::error!("Failed to acquire listening pipeline lock");
            "Unable to access listening pipeline. Please try again."
        })?;

        if pipeline.is_running() {
            pipeline.stop(audio_thread).map_err(|e| {
                crate::error!("Failed to stop listening pipeline: {}", e);
                format!("Failed to stop listening: {}", e)
            })?;
            crate::info!("Listening pipeline stopped");
        }
    }

    // Disable listening mode and transition state
    {
        let mut lm = listening_manager.lock().map_err(|_| {
            crate::error!("Failed to acquire listening manager lock");
            "Unable to access listening state. Please try again."
        })?;

        lm.disable_listening(recording_manager).map_err(|e| {
            crate::error!("Failed to disable listening: {}", e);
            format!("Failed to disable listening: {}", e)
        })?;
    }

    crate::info!("Listening mode disabled");
    Ok(())
}

/// Implementation of get_listening_status
///
/// Returns the current listening status including enabled flag and active state.
///
/// # Arguments
/// * `listening_manager` - The listening manager state
/// * `recording_manager` - The recording manager state
///
/// # Errors
/// Returns an error string if the state lock is poisoned
pub fn get_listening_status_impl(
    listening_manager: &Mutex<ListeningManager>,
    recording_manager: &Mutex<RecordingManager>,
) -> Result<ListeningStatus, String> {
    let lm = listening_manager.lock().map_err(|_| {
        crate::error!("Failed to acquire listening manager lock");
        "Unable to access listening state. Please try again."
    })?;

    lm.get_status(recording_manager).map_err(|e| {
        crate::error!("Failed to get listening status: {}", e);
        format!("Failed to get listening status: {}", e)
    })
}

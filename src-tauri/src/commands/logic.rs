// Command implementation logic - testable functions separate from Tauri wrappers

use crate::audio::{parse_duration_from_file, AudioThreadHandle, QualityWarning, TARGET_SAMPLE_RATE};

/// Error identifier for microphone access failures.
/// Used to detect microphone-related errors without fragile string matching.
pub const MICROPHONE_ERROR_MARKER: &str = "[MICROPHONE_ACCESS_ERROR]";
use crate::recording::{AudioData, RecordingManager, RecordingMetadata, RecordingState};

/// Extended result from stop_recording_impl that includes diagnostics
pub struct StopRecordingResult {
    /// The recording metadata
    pub metadata: RecordingMetadata,
    /// Quality warnings from the recording session
    pub warnings: Vec<QualityWarning>,
    /// Raw audio data (if debug mode was enabled) with device sample rate
    pub raw_audio: Option<(Vec<f32>, u32)>,
}
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
    /// Transcription text (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription: Option<String>,
    /// App name of the active window when recording started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_window_app_name: Option<String>,
    /// Bundle ID of the active window when recording started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_window_bundle_id: Option<String>,
    /// Window title of the active window when recording started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_window_title: Option<String>,
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

    // Check current state - only allow starting from Idle
    let current_state = manager.get_state();
    crate::debug!("Current recording state: {:?}", current_state);
    if current_state != RecordingState::Idle {
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
        match audio_thread.start_with_device(buffer, device_name) {
            Ok(sample_rate) => {
                // Update with actual sample rate from device
                manager.set_sample_rate(sample_rate);
                crate::info!("Audio capture started at {}Hz", sample_rate);
            }
            Err(e) => {
                // Audio capture failed - rollback state and return error
                crate::error!("Audio capture failed: {:?}", e);
                manager.reset_to_idle();
                return Err(format!(
                    "{} Could not access the microphone. Please check that your microphone is connected and permissions are granted.",
                    MICROPHONE_ERROR_MARKER
                ));
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
/// * `recordings_dir` - Directory for saving recordings (supports worktree isolation)
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
    recordings_dir: PathBuf,
) -> Result<RecordingMetadata, String> {
    // Call the extended implementation and discard diagnostics
    stop_recording_impl_extended(state, audio_thread, return_to_listening, recordings_dir)
        .map(|result| result.metadata)
}

/// Extended implementation of stop_recording that returns diagnostics
///
/// This is the full implementation that returns quality warnings and raw audio
/// in addition to recording metadata. Used by the command layer to emit events.
pub fn stop_recording_impl_extended(
    state: &Mutex<RecordingManager>,
    audio_thread: Option<&AudioThreadHandle>,
    return_to_listening: bool,
    recordings_dir: PathBuf,
) -> Result<StopRecordingResult, String> {
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

    // Extract capture file, stop reason, warnings, and raw audio from result
    let (capture_file, stop_reason, warnings, raw_audio) = match &stop_result {
        Some(result) => (
            result.capture_file.clone(),
            result.reason.clone(),
            result.warnings.clone(),
            result.raw_audio.clone(),
        ),
        None => (None, None, Vec::new(), None),
    };

    let _ = return_to_listening; // Suppress unused warning (kept for API compatibility)

    // Transition to Processing (required state machine step)
    manager
        .transition_to(RecordingState::Processing)
        .map_err(|e| {
            crate::error!("Failed to transition to Processing: {:?}", e);
            "Failed to process recording."
        })?;

    // Move temp file to final location (instant, no I/O - just a rename)
    // Falls back to encoding from buffer if no capture file (for tests)
    let (file_path, duration_secs, sample_count) = if let Some((temp_path, duration_ms)) = capture_file {
        // Fast path: Rename temp file directly (no re-encoding)
        // Transition to Idle immediately since file rename is instant
        manager
            .transition_to(RecordingState::Idle)
            .map_err(|e| {
                crate::error!("Failed to transition to Idle: {:?}", e);
                "Failed to complete recording."
            })?;

        // Release the state lock immediately
        drop(manager);
        crate::debug!("State lock released");

        // Ensure output directory exists
        if !recordings_dir.exists() {
            std::fs::create_dir_all(&recordings_dir).map_err(|e| {
                crate::error!("Failed to create recordings dir: {}", e);
                "Failed to save recording: cannot create recordings directory."
            })?;
        }

        // Generate final filename
        let now = chrono::Utc::now();
        let filename = format!("recording-{}.wav", now.format("%Y-%m-%d-%H%M%S"));
        let final_path = recordings_dir.join(&filename);
        let final_path_str = final_path.to_string_lossy().to_string();

        // Rename temp file to final location (instant - same filesystem)
        crate::debug!("Moving capture file: {} -> {}", temp_path, final_path_str);
        std::fs::rename(&temp_path, &final_path).map_err(|e| {
            crate::error!("Failed to move capture file: {}", e);
            // Try to clean up temp file on error
            let _ = std::fs::remove_file(&temp_path);
            "Failed to save the recording. Please check disk space and try again."
        })?;
        crate::debug!("Capture file moved successfully");

        // Calculate sample count from duration (16kHz)
        let sample_count = ((duration_ms as f64 / 1000.0) * TARGET_SAMPLE_RATE as f64) as usize;
        let duration_secs = duration_ms as f64 / 1000.0;

        (final_path_str, duration_secs, sample_count)
    } else {
        // No capture file - transition to Idle and return empty
        crate::debug!("No capture file available");
        manager
            .transition_to(RecordingState::Idle)
            .map_err(|e| {
                crate::error!("Failed to transition to Idle: {:?}", e);
                "Failed to complete recording."
            })?;
        drop(manager);
        (String::new(), 0.0, 0)
    };

    crate::info!("Recording stopped: {} samples, {:.2}s, stop_reason={:?}, warnings={}",
          sample_count, duration_secs, stop_reason, warnings.len());

    Ok(StopRecordingResult {
        metadata: RecordingMetadata {
            duration_secs,
            file_path,
            sample_count,
            stop_reason,
        },
        warnings,
        raw_audio,
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

/// Response for paginated list_recordings
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaginatedRecordingsResponse {
    /// The recordings for the current page
    pub recordings: Vec<RecordingInfo>,
    /// Total number of recordings available
    pub total_count: usize,
    /// Whether there are more recordings after this page
    pub has_more: bool,
}

use std::collections::HashMap;

/// Context data for a recording from database
#[derive(Debug, Clone, Default)]
pub struct RecordingContextData {
    /// Transcription text
    pub transcription: Option<String>,
    /// App name of the active window when recording started
    pub active_window_app_name: Option<String>,
    /// Bundle ID of the active window when recording started
    pub active_window_bundle_id: Option<String>,
    /// Window title of the active window when recording started
    pub active_window_title: Option<String>,
}

/// Implementation of list_recordings with pagination
///
/// Lists recordings from the specified directory with their metadata.
///
/// # Arguments
/// * `recordings_dir` - Directory containing recording files (supports worktree isolation)
/// * `limit` - Maximum number of recordings to return (for pagination)
/// * `offset` - Number of recordings to skip (for pagination)
/// * `recording_context` - Map of file_path to context data (from Turso database)
///
/// # Returns
/// A paginated response with recordings sorted by creation time (newest first).
/// Returns an empty list if the recordings directory doesn't exist or is empty.
///
/// # Errors
/// Only returns an error if there's a critical system failure.
/// Individual file errors are logged and the file is skipped.
pub fn list_recordings_impl(
    recordings_dir: PathBuf,
    limit: Option<usize>,
    offset: Option<usize>,
    recording_context: HashMap<String, RecordingContextData>,
) -> Result<PaginatedRecordingsResponse, String> {

    // Return empty list if directory doesn't exist (not an error)
    if !recordings_dir.exists() {
        return Ok(PaginatedRecordingsResponse {
            recordings: Vec::new(),
            total_count: 0,
            has_more: false,
        });
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

        // Look up context data by file path
        let context = recording_context.get(&file_path_str);

        recordings.push(RecordingInfo {
            filename,
            file_path: file_path_str,
            duration_secs,
            created_at,
            file_size_bytes,
            error: recording_error,
            transcription: context.and_then(|c| c.transcription.clone()),
            active_window_app_name: context.and_then(|c| c.active_window_app_name.clone()),
            active_window_bundle_id: context.and_then(|c| c.active_window_bundle_id.clone()),
            active_window_title: context.and_then(|c| c.active_window_title.clone()),
        });
    }

    // Sort by created_at descending (newest first)
    recordings.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Apply pagination
    let total_count = recordings.len();
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(usize::MAX);

    let paginated_recordings: Vec<RecordingInfo> = recordings
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let has_more = offset + paginated_recordings.len() < total_count;

    Ok(PaginatedRecordingsResponse {
        recordings: paginated_recordings,
        total_count,
        has_more,
    })
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

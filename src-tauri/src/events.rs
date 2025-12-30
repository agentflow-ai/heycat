// Recording events for frontend notification
// Defines event payloads and emission trait for testability

use serde::Serialize;

use crate::recording::RecordingMetadata;

/// Event names as constants for consistency
pub mod event_names {
    pub const RECORDING_STARTED: &str = "recording_started";
    pub const RECORDING_STOPPED: &str = "recording_stopped";
    pub const RECORDING_CANCELLED: &str = "recording_cancelled";
    pub const RECORDING_ERROR: &str = "recording_error";
    pub const AUDIO_DEVICE_ERROR: &str = "audio_device_error";
    pub const AUDIO_LEVEL: &str = "audio-level";
    pub const RECORDING_QUALITY_WARNING: &str = "recording_quality_warning";
    pub const TRANSCRIPTION_STARTED: &str = "transcription_started";
    pub const TRANSCRIPTION_COMPLETED: &str = "transcription_completed";
    pub const TRANSCRIPTION_ERROR: &str = "transcription_error";
    pub const SHORTCUT_KEY_CAPTURED: &str = "shortcut_key_captured";
}

/// Command-related event names
pub mod command_events {
    pub const COMMAND_MATCHED: &str = "command_matched";
    pub const COMMAND_EXECUTED: &str = "command_executed";
    pub const COMMAND_FAILED: &str = "command_failed";
    pub const COMMAND_AMBIGUOUS: &str = "command_ambiguous";
}

/// Hotkey-related event names
pub mod hotkey_events {
    pub const KEY_BLOCKING_UNAVAILABLE: &str = "key_blocking_unavailable";

    /// Payload for key_blocking_unavailable event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct KeyBlockingUnavailablePayload {
        /// Reason why key blocking is unavailable
        pub reason: String,
        /// ISO 8601 timestamp when the issue was detected
        pub timestamp: String,
    }
}

/// Trait for emitting hotkey events
/// Allows mocking in tests while using real Tauri AppHandle in production
pub trait HotkeyEventEmitter: Send + Sync {
    /// Emit key_blocking_unavailable event
    fn emit_key_blocking_unavailable(&self, payload: hotkey_events::KeyBlockingUnavailablePayload);
}

/// Dictionary-related event names
pub mod dictionary_events {
    pub const DICTIONARY_UPDATED: &str = "dictionary_updated";

    /// Payload for dictionary_updated event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct DictionaryUpdatedPayload {
        /// Type of mutation: "add", "update", or "delete"
        pub action: String,
        /// ID of the affected entry (present for all actions)
        pub entry_id: String,
    }
}

/// Window context-related event names
pub mod window_context_events {
    pub const WINDOW_CONTEXTS_UPDATED: &str = "window_contexts_updated";
    pub const ACTIVE_WINDOW_CHANGED: &str = "active_window_changed";

    /// Payload for window_contexts_updated event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct WindowContextsUpdatedPayload {
        /// Type of mutation: "add", "update", or "delete"
        pub action: String,
        /// ID of the affected context (present for all actions)
        pub context_id: String,
    }

    /// Payload for active_window_changed event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct ActiveWindowChangedPayload {
        /// Name of the foreground application
        pub app_name: String,
        /// Bundle ID of the application (macOS)
        pub bundle_id: Option<String>,
        /// Title of the active window
        pub window_title: Option<String>,
        /// ID of the matched context, if any
        pub matched_context_id: Option<String>,
        /// Name of the matched context, if any
        pub matched_context_name: Option<String>,
    }
}

/// Model-related event names
pub mod model_events {
    pub const MODEL_DOWNLOAD_COMPLETED: &str = "model_download_completed";
    pub const MODEL_FILE_DOWNLOAD_PROGRESS: &str = "model_file_download_progress";

    /// Payload for model_download_completed event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct ModelDownloadCompletedPayload {
        /// Type of model that was downloaded (e.g., "tdt", "eou")
        pub model_type: String,
        /// Path to the downloaded model directory
        pub model_path: String,
    }

    /// Payload for model_file_download_progress event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct ModelFileDownloadProgressPayload {
        /// Type of model being downloaded (e.g., "parakeet-tdt")
        pub model_type: String,
        /// Name of the file being downloaded
        pub file_name: String,
        /// Bytes downloaded so far for this file
        pub bytes_downloaded: u64,
        /// Total bytes for this file
        pub total_bytes: u64,
        /// Index of current file (0-based)
        pub file_index: usize,
        /// Total number of files to download
        pub total_files: usize,
        /// Download progress percentage (0-100)
        pub percent: f64,
    }
}

/// Payload for recording_started event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecordingStartedPayload {
    /// ISO 8601 timestamp when recording started
    pub timestamp: String,
}

/// Payload for recording_stopped event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecordingStoppedPayload {
    /// Metadata about the completed recording
    pub metadata: RecordingMetadata,
}

/// Payload for recording_error event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecordingErrorPayload {
    /// Descriptive error message
    pub message: String,
}

/// Payload for recording_cancelled event
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordingCancelledPayload {
    /// Reason for cancellation (e.g., "double-tap-escape")
    pub reason: String,
    /// ISO 8601 timestamp when recording was cancelled
    pub timestamp: String,
}

/// Payload for transcription_started event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TranscriptionStartedPayload {
    /// ISO 8601 timestamp when transcription started
    pub timestamp: String,
}

/// Payload for transcription_completed event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TranscriptionCompletedPayload {
    /// The transcribed text
    pub text: String,
    /// Duration of transcription in milliseconds
    pub duration_ms: u64,
}

/// Payload for transcription_error event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TranscriptionErrorPayload {
    /// Descriptive error message
    pub error: String,
}

/// Payload for command_matched event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommandMatchedPayload {
    /// The transcribed text that was matched
    pub transcription: String,
    /// ID of the matched command
    pub command_id: String,
    /// Trigger phrase of the matched command
    pub trigger: String,
    /// Match confidence (0.0 - 1.0)
    pub confidence: f64,
}

/// A candidate command for disambiguation
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommandCandidate {
    /// Command ID
    pub id: String,
    /// Trigger phrase
    pub trigger: String,
    /// Match confidence
    pub confidence: f64,
}

/// Payload for command_ambiguous event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommandAmbiguousPayload {
    /// The transcribed text
    pub transcription: String,
    /// List of candidate commands
    pub candidates: Vec<CommandCandidate>,
}

/// Payload for command_executed event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommandExecutedPayload {
    /// ID of the executed command
    pub command_id: String,
    /// Trigger phrase
    pub trigger: String,
    /// Result message
    pub message: String,
}

/// Payload for command_failed event
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommandFailedPayload {
    /// ID of the command that failed
    pub command_id: String,
    /// Trigger phrase
    pub trigger: String,
    /// Error code
    pub error_code: String,
    /// Error message
    pub error_message: String,
}

/// Trait for emitting recording events
/// Allows mocking in tests while using real Tauri AppHandle in production
pub trait RecordingEventEmitter: Send + Sync {
    /// Emit recording_started event
    fn emit_recording_started(&self, payload: RecordingStartedPayload);

    /// Emit recording_stopped event
    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload);

    /// Emit recording_cancelled event
    fn emit_recording_cancelled(&self, payload: RecordingCancelledPayload);

    /// Emit recording_error event
    fn emit_recording_error(&self, payload: RecordingErrorPayload);
}

/// Trait for emitting transcription events
/// Allows mocking in tests while using real Tauri AppHandle in production
pub trait TranscriptionEventEmitter: Send + Sync {
    /// Emit transcription_started event
    fn emit_transcription_started(&self, payload: TranscriptionStartedPayload);

    /// Emit transcription_completed event
    fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload);

    /// Emit transcription_error event
    fn emit_transcription_error(&self, payload: TranscriptionErrorPayload);
}

/// Trait for emitting command events
/// Allows mocking in tests while using real Tauri AppHandle in production
pub trait CommandEventEmitter: Send + Sync {
    /// Emit command_matched event
    fn emit_command_matched(&self, payload: CommandMatchedPayload);

    /// Emit command_executed event
    fn emit_command_executed(&self, payload: CommandExecutedPayload);

    /// Emit command_failed event
    fn emit_command_failed(&self, payload: CommandFailedPayload);

    /// Emit command_ambiguous event
    fn emit_command_ambiguous(&self, payload: CommandAmbiguousPayload);
}

/// Get the current timestamp in ISO 8601 format
pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
#[path = "events_test.rs"]
pub(crate) mod tests;

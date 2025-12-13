// Recording events for frontend notification
// Defines event payloads and emission trait for testability

use serde::Serialize;

use crate::recording::RecordingMetadata;

/// Event names as constants for consistency
pub mod event_names {
    pub const RECORDING_STARTED: &str = "recording_started";
    pub const RECORDING_STOPPED: &str = "recording_stopped";
    pub const RECORDING_ERROR: &str = "recording_error";
    pub const TRANSCRIPTION_STARTED: &str = "transcription_started";
    pub const TRANSCRIPTION_COMPLETED: &str = "transcription_completed";
    pub const TRANSCRIPTION_ERROR: &str = "transcription_error";
    pub const TRANSCRIPTION_PARTIAL: &str = "transcription_partial";
}

/// Command-related event names
pub mod command_events {
    pub const COMMAND_MATCHED: &str = "command_matched";
    pub const COMMAND_EXECUTED: &str = "command_executed";
    pub const COMMAND_FAILED: &str = "command_failed";
    pub const COMMAND_AMBIGUOUS: &str = "command_ambiguous";
}

/// Model-related event names
pub mod model_events {
    pub const MODEL_DOWNLOAD_COMPLETED: &str = "model_download_completed";
    pub const MODEL_FILE_DOWNLOAD_PROGRESS: &str = "model_file_download_progress";

    /// Payload for model_download_completed event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    pub struct ModelDownloadCompletedPayload {
        /// Path to the downloaded model file
        pub model_path: String,
    }

    /// Payload for model_file_download_progress event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
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

/// Payload for transcription_partial event (streaming transcription)
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TranscriptionPartialPayload {
    /// Accumulated partial transcription text so far
    pub text: String,
    /// Whether this is the final update before completed event
    pub is_final: bool,
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

    /// Emit transcription_partial event (for streaming transcription)
    fn emit_transcription_partial(&self, payload: TranscriptionPartialPayload);
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
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock emitter that records all emitted events for testing
    #[derive(Default)]
    pub struct MockEventEmitter {
        pub started_events: Arc<Mutex<Vec<RecordingStartedPayload>>>,
        pub stopped_events: Arc<Mutex<Vec<RecordingStoppedPayload>>>,
        pub error_events: Arc<Mutex<Vec<RecordingErrorPayload>>>,
        pub transcription_started_events: Arc<Mutex<Vec<TranscriptionStartedPayload>>>,
        pub transcription_completed_events: Arc<Mutex<Vec<TranscriptionCompletedPayload>>>,
        pub transcription_error_events: Arc<Mutex<Vec<TranscriptionErrorPayload>>>,
        pub transcription_partial_events: Arc<Mutex<Vec<TranscriptionPartialPayload>>>,
        pub command_matched_events: Arc<Mutex<Vec<CommandMatchedPayload>>>,
        pub command_executed_events: Arc<Mutex<Vec<CommandExecutedPayload>>>,
        pub command_failed_events: Arc<Mutex<Vec<CommandFailedPayload>>>,
        pub command_ambiguous_events: Arc<Mutex<Vec<CommandAmbiguousPayload>>>,
    }

    impl MockEventEmitter {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl RecordingEventEmitter for MockEventEmitter {
        fn emit_recording_started(&self, payload: RecordingStartedPayload) {
            self.started_events.lock().unwrap().push(payload);
        }

        fn emit_recording_stopped(&self, payload: RecordingStoppedPayload) {
            self.stopped_events.lock().unwrap().push(payload);
        }

        fn emit_recording_error(&self, payload: RecordingErrorPayload) {
            self.error_events.lock().unwrap().push(payload);
        }
    }

    impl TranscriptionEventEmitter for MockEventEmitter {
        fn emit_transcription_started(&self, payload: TranscriptionStartedPayload) {
            self.transcription_started_events.lock().unwrap().push(payload);
        }

        fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload) {
            self.transcription_completed_events.lock().unwrap().push(payload);
        }

        fn emit_transcription_error(&self, payload: TranscriptionErrorPayload) {
            self.transcription_error_events.lock().unwrap().push(payload);
        }

        fn emit_transcription_partial(&self, payload: TranscriptionPartialPayload) {
            self.transcription_partial_events.lock().unwrap().push(payload);
        }
    }

    impl CommandEventEmitter for MockEventEmitter {
        fn emit_command_matched(&self, payload: CommandMatchedPayload) {
            self.command_matched_events.lock().unwrap().push(payload);
        }

        fn emit_command_executed(&self, payload: CommandExecutedPayload) {
            self.command_executed_events.lock().unwrap().push(payload);
        }

        fn emit_command_failed(&self, payload: CommandFailedPayload) {
            self.command_failed_events.lock().unwrap().push(payload);
        }

        fn emit_command_ambiguous(&self, payload: CommandAmbiguousPayload) {
            self.command_ambiguous_events.lock().unwrap().push(payload);
        }
    }

    #[test]
    fn test_event_name_constants() {
        assert_eq!(event_names::RECORDING_STARTED, "recording_started");
        assert_eq!(event_names::RECORDING_STOPPED, "recording_stopped");
        assert_eq!(event_names::RECORDING_ERROR, "recording_error");
    }

    #[test]
    fn test_recording_started_payload_serialization() {
        let payload = RecordingStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("2025-01-01T12:00:00Z"));
    }

    #[test]
    fn test_recording_stopped_payload_serialization() {
        let metadata = RecordingMetadata {
            duration_secs: 5.5,
            file_path: "/tmp/test.wav".to_string(),
            sample_count: 88200,
            stop_reason: None,
        };
        let payload = RecordingStoppedPayload { metadata };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("metadata"));
        assert!(json.contains("duration_secs"));
        assert!(json.contains("5.5"));
    }

    #[test]
    fn test_recording_error_payload_serialization() {
        let payload = RecordingErrorPayload {
            message: "Microphone not found".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("message"));
        assert!(json.contains("Microphone not found"));
    }

    #[test]
    fn test_current_timestamp_is_iso8601() {
        let timestamp = current_timestamp();
        // ISO 8601 format: contains date separators and timezone
        assert!(timestamp.contains("T"));
        assert!(timestamp.contains("-"));
        // Should parse as valid RFC 3339 timestamp
        assert!(chrono::DateTime::parse_from_rfc3339(&timestamp).is_ok());
    }

    #[test]
    fn test_mock_emitter_records_started_event() {
        let emitter = MockEventEmitter::new();
        let payload = RecordingStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        emitter.emit_recording_started(payload.clone());

        let events = emitter.started_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], payload);
    }

    #[test]
    fn test_mock_emitter_records_stopped_event() {
        let emitter = MockEventEmitter::new();
        let metadata = RecordingMetadata {
            duration_secs: 3.0,
            file_path: "/tmp/recording.wav".to_string(),
            sample_count: 48000,
            stop_reason: None,
        };
        let payload = RecordingStoppedPayload {
            metadata: metadata.clone(),
        };
        emitter.emit_recording_stopped(payload.clone());

        let events = emitter.stopped_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], payload);
    }

    #[test]
    fn test_mock_emitter_records_error_event() {
        let emitter = MockEventEmitter::new();
        let payload = RecordingErrorPayload {
            message: "Audio device error".to_string(),
        };
        emitter.emit_recording_error(payload.clone());

        let events = emitter.error_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], payload);
    }

    #[test]
    fn test_mock_emitter_records_multiple_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_recording_started(RecordingStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        emitter.emit_recording_started(RecordingStartedPayload {
            timestamp: "2025-01-01T12:01:00Z".to_string(),
        });

        let events = emitter.started_events.lock().unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_payloads_are_clone() {
        let started = RecordingStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let cloned = started.clone();
        assert_eq!(started, cloned);

        let error = RecordingErrorPayload {
            message: "Error".to_string(),
        };
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_payloads_have_debug() {
        let started = RecordingStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let debug = format!("{:?}", started);
        assert!(debug.contains("RecordingStartedPayload"));

        let error = RecordingErrorPayload {
            message: "Error".to_string(),
        };
        let debug = format!("{:?}", error);
        assert!(debug.contains("RecordingErrorPayload"));
    }

    #[test]
    fn test_model_event_name_constant() {
        use super::model_events;
        assert_eq!(
            model_events::MODEL_DOWNLOAD_COMPLETED,
            "model_download_completed"
        );
    }

    #[test]
    fn test_model_download_completed_payload_serialization() {
        use super::model_events::ModelDownloadCompletedPayload;
        let payload = ModelDownloadCompletedPayload {
            model_path: "/path/to/model.bin".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("model_path"));
        assert!(json.contains("/path/to/model.bin"));
    }

    #[test]
    fn test_model_download_completed_payload_clone() {
        use super::model_events::ModelDownloadCompletedPayload;
        let payload = ModelDownloadCompletedPayload {
            model_path: "/path/to/model.bin".to_string(),
        };
        let cloned = payload.clone();
        assert_eq!(payload, cloned);
    }

    #[test]
    fn test_model_download_completed_payload_debug() {
        use super::model_events::ModelDownloadCompletedPayload;
        let payload = ModelDownloadCompletedPayload {
            model_path: "/path/to/model.bin".to_string(),
        };
        let debug = format!("{:?}", payload);
        assert!(debug.contains("ModelDownloadCompletedPayload"));
    }

    // Transcription event tests

    #[test]
    fn test_transcription_event_name_constants() {
        assert_eq!(event_names::TRANSCRIPTION_STARTED, "transcription_started");
        assert_eq!(
            event_names::TRANSCRIPTION_COMPLETED,
            "transcription_completed"
        );
        assert_eq!(event_names::TRANSCRIPTION_ERROR, "transcription_error");
    }

    #[test]
    fn test_transcription_started_payload_serialization() {
        let payload = TranscriptionStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("2025-01-01T12:00:00Z"));
    }

    #[test]
    fn test_transcription_completed_payload_serialization() {
        let payload = TranscriptionCompletedPayload {
            text: "Hello, world!".to_string(),
            duration_ms: 1234,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("Hello, world!"));
        assert!(json.contains("duration_ms"));
        assert!(json.contains("1234"));
    }

    #[test]
    fn test_transcription_error_payload_serialization() {
        let payload = TranscriptionErrorPayload {
            error: "Model not loaded".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Model not loaded"));
    }

    #[test]
    fn test_transcription_payloads_are_clone() {
        let started = TranscriptionStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let cloned = started.clone();
        assert_eq!(started, cloned);

        let completed = TranscriptionCompletedPayload {
            text: "Hello".to_string(),
            duration_ms: 100,
        };
        let cloned = completed.clone();
        assert_eq!(completed, cloned);

        let error = TranscriptionErrorPayload {
            error: "Error".to_string(),
        };
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_transcription_payloads_have_debug() {
        let started = TranscriptionStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let debug = format!("{:?}", started);
        assert!(debug.contains("TranscriptionStartedPayload"));

        let completed = TranscriptionCompletedPayload {
            text: "Hello".to_string(),
            duration_ms: 100,
        };
        let debug = format!("{:?}", completed);
        assert!(debug.contains("TranscriptionCompletedPayload"));

        let error = TranscriptionErrorPayload {
            error: "Error".to_string(),
        };
        let debug = format!("{:?}", error);
        assert!(debug.contains("TranscriptionErrorPayload"));
    }

    #[test]
    fn test_transcription_partial_event_name_constant() {
        assert_eq!(event_names::TRANSCRIPTION_PARTIAL, "transcription_partial");
    }

    #[test]
    fn test_transcription_partial_payload_serialization() {
        let payload = TranscriptionPartialPayload {
            text: "Hello world".to_string(),
            is_final: false,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("Hello world"));
        assert!(json.contains("is_final"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_transcription_partial_payload_clone() {
        let payload = TranscriptionPartialPayload {
            text: "Test".to_string(),
            is_final: true,
        };
        let cloned = payload.clone();
        assert_eq!(payload, cloned);
    }

    #[test]
    fn test_transcription_partial_payload_debug() {
        let payload = TranscriptionPartialPayload {
            text: "Test".to_string(),
            is_final: false,
        };
        let debug = format!("{:?}", payload);
        assert!(debug.contains("TranscriptionPartialPayload"));
    }

    #[test]
    fn test_mock_emitter_records_partial_event() {
        let emitter = MockEventEmitter::new();
        let payload = TranscriptionPartialPayload {
            text: "Hello".to_string(),
            is_final: false,
        };
        emitter.emit_transcription_partial(payload.clone());

        let events = emitter.transcription_partial_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], payload);
    }

    // Command event tests

    #[test]
    fn test_command_event_name_constants() {
        assert_eq!(command_events::COMMAND_MATCHED, "command_matched");
        assert_eq!(command_events::COMMAND_EXECUTED, "command_executed");
        assert_eq!(command_events::COMMAND_FAILED, "command_failed");
        assert_eq!(command_events::COMMAND_AMBIGUOUS, "command_ambiguous");
    }

    #[test]
    fn test_command_matched_payload_serialization() {
        let payload = CommandMatchedPayload {
            transcription: "open slack".to_string(),
            command_id: "123".to_string(),
            trigger: "open slack".to_string(),
            confidence: 0.95,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("transcription"));
        assert!(json.contains("command_id"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_command_ambiguous_payload_serialization() {
        let payload = CommandAmbiguousPayload {
            transcription: "open".to_string(),
            candidates: vec![
                CommandCandidate {
                    id: "1".to_string(),
                    trigger: "open slack".to_string(),
                    confidence: 0.85,
                },
                CommandCandidate {
                    id: "2".to_string(),
                    trigger: "open safari".to_string(),
                    confidence: 0.83,
                },
            ],
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("candidates"));
        assert!(json.contains("open slack"));
        assert!(json.contains("open safari"));
    }

    #[test]
    fn test_command_executed_payload_serialization() {
        let payload = CommandExecutedPayload {
            command_id: "123".to_string(),
            trigger: "open slack".to_string(),
            message: "Opened Slack.app".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("command_id"));
        assert!(json.contains("message"));
    }

    #[test]
    fn test_command_failed_payload_serialization() {
        let payload = CommandFailedPayload {
            command_id: "123".to_string(),
            trigger: "open nonexistent".to_string(),
            error_code: "NOT_FOUND".to_string(),
            error_message: "Application not found".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("error_code"));
        assert!(json.contains("error_message"));
    }

    #[test]
    fn test_command_payloads_are_clone() {
        let matched = CommandMatchedPayload {
            transcription: "test".to_string(),
            command_id: "1".to_string(),
            trigger: "test".to_string(),
            confidence: 0.9,
        };
        assert_eq!(matched, matched.clone());

        let ambiguous = CommandAmbiguousPayload {
            transcription: "test".to_string(),
            candidates: vec![],
        };
        assert_eq!(ambiguous, ambiguous.clone());

        let executed = CommandExecutedPayload {
            command_id: "1".to_string(),
            trigger: "test".to_string(),
            message: "done".to_string(),
        };
        assert_eq!(executed, executed.clone());

        let failed = CommandFailedPayload {
            command_id: "1".to_string(),
            trigger: "test".to_string(),
            error_code: "ERR".to_string(),
            error_message: "error".to_string(),
        };
        assert_eq!(failed, failed.clone());
    }

    #[test]
    fn test_mock_emitter_records_command_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_command_matched(CommandMatchedPayload {
            transcription: "open slack".to_string(),
            command_id: "1".to_string(),
            trigger: "open slack".to_string(),
            confidence: 0.95,
        });
        assert_eq!(emitter.command_matched_events.lock().unwrap().len(), 1);

        emitter.emit_command_executed(CommandExecutedPayload {
            command_id: "1".to_string(),
            trigger: "open slack".to_string(),
            message: "Opened".to_string(),
        });
        assert_eq!(emitter.command_executed_events.lock().unwrap().len(), 1);

        emitter.emit_command_failed(CommandFailedPayload {
            command_id: "1".to_string(),
            trigger: "test".to_string(),
            error_code: "ERR".to_string(),
            error_message: "error".to_string(),
        });
        assert_eq!(emitter.command_failed_events.lock().unwrap().len(), 1);

        emitter.emit_command_ambiguous(CommandAmbiguousPayload {
            transcription: "open".to_string(),
            candidates: vec![],
        });
        assert_eq!(emitter.command_ambiguous_events.lock().unwrap().len(), 1);
    }

    // Model file download progress event tests

    #[test]
    fn test_model_file_download_progress_event_name_constant() {
        assert_eq!(
            model_events::MODEL_FILE_DOWNLOAD_PROGRESS,
            "model_file_download_progress"
        );
    }

    #[test]
    fn test_model_file_download_progress_payload_serialization() {
        use super::model_events::ModelFileDownloadProgressPayload;
        let payload = ModelFileDownloadProgressPayload {
            model_type: "parakeet-tdt".to_string(),
            file_name: "encoder.onnx".to_string(),
            bytes_downloaded: 50_000_000,
            total_bytes: 100_000_000,
            file_index: 0,
            total_files: 4,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("model_type"));
        assert!(json.contains("parakeet-tdt"));
        assert!(json.contains("file_name"));
        assert!(json.contains("encoder.onnx"));
        assert!(json.contains("bytes_downloaded"));
        assert!(json.contains("50000000"));
        assert!(json.contains("total_bytes"));
        assert!(json.contains("100000000"));
        assert!(json.contains("file_index"));
        assert!(json.contains("total_files"));
    }

    #[test]
    fn test_model_file_download_progress_payload_clone() {
        use super::model_events::ModelFileDownloadProgressPayload;
        let payload = ModelFileDownloadProgressPayload {
            model_type: "parakeet-tdt".to_string(),
            file_name: "encoder.onnx".to_string(),
            bytes_downloaded: 50_000_000,
            total_bytes: 100_000_000,
            file_index: 0,
            total_files: 4,
        };
        let cloned = payload.clone();
        assert_eq!(payload, cloned);
    }

    #[test]
    fn test_model_file_download_progress_payload_debug() {
        use super::model_events::ModelFileDownloadProgressPayload;
        let payload = ModelFileDownloadProgressPayload {
            model_type: "parakeet-tdt".to_string(),
            file_name: "encoder.onnx".to_string(),
            bytes_downloaded: 50_000_000,
            total_bytes: 100_000_000,
            file_index: 0,
            total_files: 4,
        };
        let debug = format!("{:?}", payload);
        assert!(debug.contains("ModelFileDownloadProgressPayload"));
    }
}

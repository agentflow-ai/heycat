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
}

/// Command-related event names
pub mod command_events {
    pub const COMMAND_MATCHED: &str = "command_matched";
    pub const COMMAND_EXECUTED: &str = "command_executed";
    pub const COMMAND_FAILED: &str = "command_failed";
    pub const COMMAND_AMBIGUOUS: &str = "command_ambiguous";
}

/// Listening-related event names
pub mod listening_events {
    pub const WAKE_WORD_DETECTED: &str = "wake_word_detected";
    pub const LISTENING_STARTED: &str = "listening_started";
    pub const LISTENING_STOPPED: &str = "listening_stopped";
    pub const LISTENING_UNAVAILABLE: &str = "listening_unavailable";
    pub const RECORDING_CANCELLED: &str = "recording_cancelled";

    /// Payload for wake_word_detected event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct WakeWordDetectedPayload {
        /// Confidence score (0.0 - 1.0)
        pub confidence: f32,
        /// The transcribed text that triggered detection
        pub transcription: String,
        /// ISO 8601 timestamp when wake word was detected
        pub timestamp: String,
    }

    /// Payload for listening_started event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct ListeningStartedPayload {
        /// ISO 8601 timestamp when listening started
        pub timestamp: String,
    }

    /// Payload for listening_stopped event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct ListeningStoppedPayload {
        /// ISO 8601 timestamp when listening stopped
        pub timestamp: String,
    }

    /// Payload for listening_unavailable event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct ListeningUnavailablePayload {
        /// Reason why listening is unavailable
        pub reason: String,
        /// ISO 8601 timestamp when listening became unavailable
        pub timestamp: String,
    }

    /// Payload for recording_cancelled event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct RecordingCancelledPayload {
        /// The cancel phrase that was detected (e.g., "cancel", "nevermind")
        pub cancel_phrase: String,
        /// ISO 8601 timestamp when cancellation was triggered
        pub timestamp: String,
    }
}

/// Trait for emitting listening events
/// Allows mocking in tests while using real Tauri AppHandle in production
pub trait ListeningEventEmitter: Send + Sync {
    /// Emit wake_word_detected event
    fn emit_wake_word_detected(&self, payload: listening_events::WakeWordDetectedPayload);

    /// Emit listening_started event
    fn emit_listening_started(&self, payload: listening_events::ListeningStartedPayload);

    /// Emit listening_stopped event
    fn emit_listening_stopped(&self, payload: listening_events::ListeningStoppedPayload);

    /// Emit listening_unavailable event
    fn emit_listening_unavailable(&self, payload: listening_events::ListeningUnavailablePayload);

    /// Emit recording_cancelled event
    fn emit_recording_cancelled(&self, payload: listening_events::RecordingCancelledPayload);
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
pub(crate) mod tests {
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
        pub command_matched_events: Arc<Mutex<Vec<CommandMatchedPayload>>>,
        pub command_executed_events: Arc<Mutex<Vec<CommandExecutedPayload>>>,
        pub command_failed_events: Arc<Mutex<Vec<CommandFailedPayload>>>,
        pub command_ambiguous_events: Arc<Mutex<Vec<CommandAmbiguousPayload>>>,
        pub wake_word_detected_events: Arc<Mutex<Vec<listening_events::WakeWordDetectedPayload>>>,
        pub listening_started_events: Arc<Mutex<Vec<listening_events::ListeningStartedPayload>>>,
        pub listening_stopped_events: Arc<Mutex<Vec<listening_events::ListeningStoppedPayload>>>,
        pub listening_unavailable_events: Arc<Mutex<Vec<listening_events::ListeningUnavailablePayload>>>,
        pub recording_cancelled_events: Arc<Mutex<Vec<listening_events::RecordingCancelledPayload>>>,
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

    impl ListeningEventEmitter for MockEventEmitter {
        fn emit_wake_word_detected(&self, payload: listening_events::WakeWordDetectedPayload) {
            self.wake_word_detected_events.lock().unwrap().push(payload);
        }

        fn emit_listening_started(&self, payload: listening_events::ListeningStartedPayload) {
            self.listening_started_events.lock().unwrap().push(payload);
        }

        fn emit_listening_stopped(&self, payload: listening_events::ListeningStoppedPayload) {
            self.listening_stopped_events.lock().unwrap().push(payload);
        }

        fn emit_listening_unavailable(&self, payload: listening_events::ListeningUnavailablePayload) {
            self.listening_unavailable_events.lock().unwrap().push(payload);
        }

        fn emit_recording_cancelled(&self, payload: listening_events::RecordingCancelledPayload) {
            self.recording_cancelled_events.lock().unwrap().push(payload);
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
            model_type: "tdt".to_string(),
            model_path: "/path/to/model".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("modelType"));
        assert!(json.contains("modelPath"));
    }

    #[test]
    fn test_model_download_completed_payload_clone() {
        use super::model_events::ModelDownloadCompletedPayload;
        let payload = ModelDownloadCompletedPayload {
            model_type: "tdt".to_string(),
            model_path: "/path/to/model".to_string(),
        };
        let cloned = payload.clone();
        assert_eq!(payload, cloned);
    }

    #[test]
    fn test_model_download_completed_payload_debug() {
        use super::model_events::ModelDownloadCompletedPayload;
        let payload = ModelDownloadCompletedPayload {
            model_type: "tdt".to_string(),
            model_path: "/path/to/model".to_string(),
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

    #[test]
    fn test_mock_emitter_records_wake_word_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_wake_word_detected(listening_events::WakeWordDetectedPayload {
            confidence: 0.95,
            transcription: "hey cat".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        assert_eq!(emitter.wake_word_detected_events.lock().unwrap().len(), 1);

        let events = emitter.wake_word_detected_events.lock().unwrap();
        assert_eq!(events[0].confidence, 0.95);
        assert_eq!(events[0].transcription, "hey cat");
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
            percent: 50.0,
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("modelType"));
        assert!(json.contains("parakeet-tdt"));
        assert!(json.contains("fileName"));
        assert!(json.contains("encoder.onnx"));
        assert!(json.contains("bytesDownloaded"));
        assert!(json.contains("50000000"));
        assert!(json.contains("totalBytes"));
        assert!(json.contains("100000000"));
        assert!(json.contains("fileIndex"));
        assert!(json.contains("totalFiles"));
        assert!(json.contains("percent"));
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
            percent: 50.0,
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
            percent: 50.0,
        };
        let debug = format!("{:?}", payload);
        assert!(debug.contains("ModelFileDownloadProgressPayload"));
    }

    // Listening event tests

    #[test]
    fn test_wake_word_detected_event_name_constant() {
        use super::listening_events;
        assert_eq!(listening_events::WAKE_WORD_DETECTED, "wake_word_detected");
    }

    #[test]
    fn test_wake_word_detected_payload_serialization() {
        use super::listening_events::WakeWordDetectedPayload;
        let payload = WakeWordDetectedPayload {
            confidence: 0.95,
            transcription: "hey cat".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("confidence"));
        assert!(json.contains("0.95"));
        assert!(json.contains("transcription"));
        assert!(json.contains("hey cat"));
        assert!(json.contains("timestamp"));
    }

    #[test]
    fn test_wake_word_detected_payload_clone() {
        use super::listening_events::WakeWordDetectedPayload;
        let payload = WakeWordDetectedPayload {
            confidence: 0.95,
            transcription: "hey cat".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let cloned = payload.clone();
        assert_eq!(payload, cloned);
    }

    #[test]
    fn test_wake_word_detected_payload_debug() {
        use super::listening_events::WakeWordDetectedPayload;
        let payload = WakeWordDetectedPayload {
            confidence: 0.95,
            transcription: "hey cat".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let debug = format!("{:?}", payload);
        assert!(debug.contains("WakeWordDetectedPayload"));
    }

    #[test]
    fn test_listening_started_event_name_constant() {
        use super::listening_events;
        assert_eq!(listening_events::LISTENING_STARTED, "listening_started");
    }

    #[test]
    fn test_listening_stopped_event_name_constant() {
        use super::listening_events;
        assert_eq!(listening_events::LISTENING_STOPPED, "listening_stopped");
    }

    #[test]
    fn test_listening_unavailable_event_name_constant() {
        use super::listening_events;
        assert_eq!(listening_events::LISTENING_UNAVAILABLE, "listening_unavailable");
    }

    #[test]
    fn test_listening_started_payload_serialization() {
        use super::listening_events::ListeningStartedPayload;
        let payload = ListeningStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("2025-01-01T12:00:00Z"));
    }

    #[test]
    fn test_listening_stopped_payload_serialization() {
        use super::listening_events::ListeningStoppedPayload;
        let payload = ListeningStoppedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("2025-01-01T12:00:00Z"));
    }

    #[test]
    fn test_listening_unavailable_payload_serialization() {
        use super::listening_events::ListeningUnavailablePayload;
        let payload = ListeningUnavailablePayload {
            reason: "Microphone disconnected".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("reason"));
        assert!(json.contains("Microphone disconnected"));
        assert!(json.contains("timestamp"));
    }

    #[test]
    fn test_listening_payloads_are_clone() {
        use super::listening_events::{ListeningStartedPayload, ListeningStoppedPayload, ListeningUnavailablePayload};

        let started = ListeningStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(started, started.clone());

        let stopped = ListeningStoppedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(stopped, stopped.clone());

        let unavailable = ListeningUnavailablePayload {
            reason: "test".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(unavailable, unavailable.clone());
    }

    #[test]
    fn test_mock_emitter_records_listening_started_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_listening_started(listening_events::ListeningStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        assert_eq!(emitter.listening_started_events.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_mock_emitter_records_listening_stopped_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_listening_stopped(listening_events::ListeningStoppedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        assert_eq!(emitter.listening_stopped_events.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_mock_emitter_records_listening_unavailable_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_listening_unavailable(listening_events::ListeningUnavailablePayload {
            reason: "Microphone disconnected".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        assert_eq!(emitter.listening_unavailable_events.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_recording_cancelled_event_name_constant() {
        assert_eq!(listening_events::RECORDING_CANCELLED, "recording_cancelled");
    }

    #[test]
    fn test_recording_cancelled_payload_serialization() {
        use super::listening_events::RecordingCancelledPayload;
        let payload = RecordingCancelledPayload {
            cancel_phrase: "cancel".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("cancelPhrase"));
        assert!(json.contains("cancel"));
        assert!(json.contains("timestamp"));
    }

    #[test]
    fn test_recording_cancelled_payload_clone() {
        use super::listening_events::RecordingCancelledPayload;
        let payload = RecordingCancelledPayload {
            cancel_phrase: "nevermind".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        };
        let cloned = payload.clone();
        assert_eq!(payload, cloned);
    }

    #[test]
    fn test_mock_emitter_records_recording_cancelled_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_recording_cancelled(listening_events::RecordingCancelledPayload {
            cancel_phrase: "cancel".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        assert_eq!(emitter.recording_cancelled_events.lock().unwrap().len(), 1);

        let events = emitter.recording_cancelled_events.lock().unwrap();
        assert_eq!(events[0].cancel_phrase, "cancel");
    }
}

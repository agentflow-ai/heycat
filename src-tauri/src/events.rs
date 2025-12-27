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

/// Listening-related event names
pub mod listening_events {
    pub const WAKE_WORD_DETECTED: &str = "wake_word_detected";
    pub const LISTENING_STARTED: &str = "listening_started";
    pub const LISTENING_STOPPED: &str = "listening_stopped";
    pub const LISTENING_UNAVAILABLE: &str = "listening_unavailable";

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
}

/// Trait for emitting listening events
/// Allows mocking in tests while using real Tauri AppHandle in production
pub trait ListeningEventEmitter: Send + Sync {
    /// Emit wake_word_detected event
    fn emit_wake_word_detected(&self, payload: listening_events::WakeWordDetectedPayload);

    /// Emit listening_started event
    /// Note: Currently emitted from commands layer via Tauri app handle, not via this trait
    #[allow(dead_code)] // API consistency - commands emit via app handle
    fn emit_listening_started(&self, payload: listening_events::ListeningStartedPayload);

    /// Emit listening_stopped event
    /// Note: Currently emitted from commands layer via Tauri app handle, not via this trait
    #[allow(dead_code)] // API consistency - commands emit via app handle
    fn emit_listening_stopped(&self, payload: listening_events::ListeningStoppedPayload);

    /// Emit listening_unavailable event
    fn emit_listening_unavailable(&self, payload: listening_events::ListeningUnavailablePayload);
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
pub(crate) mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock emitter that records all emitted events for testing
    #[derive(Default)]
    pub struct MockEventEmitter {
        pub started_events: Arc<Mutex<Vec<RecordingStartedPayload>>>,
        pub stopped_events: Arc<Mutex<Vec<RecordingStoppedPayload>>>,
        pub cancelled_events: Arc<Mutex<Vec<RecordingCancelledPayload>>>,
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
        pub listening_unavailable_events:
            Arc<Mutex<Vec<listening_events::ListeningUnavailablePayload>>>,
        pub key_blocking_unavailable_events:
            Arc<Mutex<Vec<hotkey_events::KeyBlockingUnavailablePayload>>>,
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

        fn emit_recording_cancelled(&self, payload: RecordingCancelledPayload) {
            self.cancelled_events.lock().unwrap().push(payload);
        }

        fn emit_recording_error(&self, payload: RecordingErrorPayload) {
            self.error_events.lock().unwrap().push(payload);
        }
    }

    impl TranscriptionEventEmitter for MockEventEmitter {
        fn emit_transcription_started(&self, payload: TranscriptionStartedPayload) {
            self.transcription_started_events
                .lock()
                .unwrap()
                .push(payload);
        }

        fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload) {
            self.transcription_completed_events
                .lock()
                .unwrap()
                .push(payload);
        }

        fn emit_transcription_error(&self, payload: TranscriptionErrorPayload) {
            self.transcription_error_events
                .lock()
                .unwrap()
                .push(payload);
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

        fn emit_listening_unavailable(
            &self,
            payload: listening_events::ListeningUnavailablePayload,
        ) {
            self.listening_unavailable_events
                .lock()
                .unwrap()
                .push(payload);
        }
    }

    impl HotkeyEventEmitter for MockEventEmitter {
        fn emit_key_blocking_unavailable(
            &self,
            payload: hotkey_events::KeyBlockingUnavailablePayload,
        ) {
            self.key_blocking_unavailable_events
                .lock()
                .unwrap()
                .push(payload);
        }
    }

    #[test]
    fn test_current_timestamp_is_iso8601() {
        let timestamp = current_timestamp();
        assert!(timestamp.contains("T"));
        assert!(timestamp.contains("-"));
        assert!(chrono::DateTime::parse_from_rfc3339(&timestamp).is_ok());
    }

    // Verify serde camelCase rename works (smoke test for all payloads)
    #[test]
    fn test_serde_camel_case_rename() {
        use super::model_events::ModelFileDownloadProgressPayload;
        let payload = ModelFileDownloadProgressPayload {
            model_type: "test".to_string(),
            file_name: "test.onnx".to_string(),
            bytes_downloaded: 100,
            total_bytes: 200,
            file_index: 0,
            total_files: 1,
            percent: 50.0,
        };
        let json = serde_json::to_string(&payload).unwrap();
        // Verify snake_case fields are serialized as camelCase
        assert!(json.contains("modelType"));
        assert!(json.contains("fileName"));
        assert!(json.contains("bytesDownloaded"));
        assert!(json.contains("totalBytes"));
        assert!(json.contains("fileIndex"));
        assert!(json.contains("totalFiles"));
        assert!(!json.contains("model_type"));
        assert!(!json.contains("file_name"));
    }

    // MockEmitter tests - verify the mock infrastructure works correctly
    #[test]
    fn test_mock_emitter_records_recording_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_recording_started(RecordingStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        emitter.emit_recording_stopped(RecordingStoppedPayload {
            metadata: RecordingMetadata {
                duration_secs: 3.0,
                file_path: "/tmp/test.wav".to_string(),
                sample_count: 48000,
                stop_reason: None,
            },
        });
        emitter.emit_recording_error(RecordingErrorPayload {
            message: "Test error".to_string(),
        });

        assert_eq!(emitter.started_events.lock().unwrap().len(), 1);
        assert_eq!(emitter.stopped_events.lock().unwrap().len(), 1);
        assert_eq!(emitter.error_events.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_mock_emitter_records_transcription_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_transcription_started(TranscriptionStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        emitter.emit_transcription_completed(TranscriptionCompletedPayload {
            text: "Hello".to_string(),
            duration_ms: 100,
        });
        emitter.emit_transcription_error(TranscriptionErrorPayload {
            error: "Test error".to_string(),
        });

        assert_eq!(
            emitter.transcription_started_events.lock().unwrap().len(),
            1
        );
        assert_eq!(
            emitter
                .transcription_completed_events
                .lock()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(emitter.transcription_error_events.lock().unwrap().len(), 1);
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
        emitter.emit_command_executed(CommandExecutedPayload {
            command_id: "1".to_string(),
            trigger: "open slack".to_string(),
            message: "Opened".to_string(),
        });
        emitter.emit_command_failed(CommandFailedPayload {
            command_id: "1".to_string(),
            trigger: "test".to_string(),
            error_code: "ERR".to_string(),
            error_message: "error".to_string(),
        });
        emitter.emit_command_ambiguous(CommandAmbiguousPayload {
            transcription: "open".to_string(),
            candidates: vec![],
        });

        assert_eq!(emitter.command_matched_events.lock().unwrap().len(), 1);
        assert_eq!(emitter.command_executed_events.lock().unwrap().len(), 1);
        assert_eq!(emitter.command_failed_events.lock().unwrap().len(), 1);
        assert_eq!(emitter.command_ambiguous_events.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_mock_emitter_records_listening_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_wake_word_detected(listening_events::WakeWordDetectedPayload {
            confidence: 0.95,
            transcription: "hey cat".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        emitter.emit_listening_started(listening_events::ListeningStartedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        emitter.emit_listening_stopped(listening_events::ListeningStoppedPayload {
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });
        emitter.emit_listening_unavailable(listening_events::ListeningUnavailablePayload {
            reason: "Microphone disconnected".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });

        assert_eq!(emitter.wake_word_detected_events.lock().unwrap().len(), 1);
        assert_eq!(emitter.listening_started_events.lock().unwrap().len(), 1);
        assert_eq!(emitter.listening_stopped_events.lock().unwrap().len(), 1);
        assert_eq!(
            emitter.listening_unavailable_events.lock().unwrap().len(),
            1
        );
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

        assert_eq!(emitter.started_events.lock().unwrap().len(), 2);
    }

    #[test]
    fn test_mock_emitter_records_hotkey_events() {
        let emitter = MockEventEmitter::new();

        emitter.emit_key_blocking_unavailable(hotkey_events::KeyBlockingUnavailablePayload {
            reason: "Accessibility permission denied".to_string(),
            timestamp: "2025-01-01T12:00:00Z".to_string(),
        });

        assert_eq!(
            emitter.key_blocking_unavailable_events.lock().unwrap().len(),
            1
        );
        let payload = &emitter.key_blocking_unavailable_events.lock().unwrap()[0];
        assert_eq!(payload.reason, "Accessibility permission denied");
    }
}

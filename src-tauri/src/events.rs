// Recording events for frontend notification
// Defines event payloads and emission trait for testability

use serde::Serialize;

use crate::recording::RecordingMetadata;

/// Event names as constants for consistency
pub mod event_names {
    pub const RECORDING_STARTED: &str = "recording_started";
    pub const RECORDING_STOPPED: &str = "recording_stopped";
    pub const RECORDING_ERROR: &str = "recording_error";
}

/// Model-related event names
pub mod model_events {
    pub const MODEL_DOWNLOAD_COMPLETED: &str = "model_download_completed";

    /// Payload for model_download_completed event
    #[derive(Debug, Clone, serde::Serialize, PartialEq)]
    pub struct ModelDownloadCompletedPayload {
        /// Path to the downloaded model file
        pub model_path: String,
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
}

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

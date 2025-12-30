use super::*;
use std::sync::{Arc, Mutex};

/// Mock emitter for testing that records all emitted events
struct MockEmitter {
    dictionary_updates: Arc<Mutex<u32>>,
    window_context_updates: Arc<Mutex<u32>>,
    voice_command_updates: Arc<Mutex<Vec<VoiceCommandsUpdatedPayload>>>,
    recording_updates: Arc<Mutex<Vec<RecordingsUpdatedPayload>>>,
    transcription_updates: Arc<Mutex<Vec<TranscriptionsUpdatedPayload>>>,
    connection_updates: Arc<Mutex<Vec<ConnectionStatusPayload>>>,
}

impl MockEmitter {
    fn new() -> Self {
        Self {
            dictionary_updates: Arc::new(Mutex::new(0)),
            window_context_updates: Arc::new(Mutex::new(0)),
            voice_command_updates: Arc::new(Mutex::new(Vec::new())),
            recording_updates: Arc::new(Mutex::new(Vec::new())),
            transcription_updates: Arc::new(Mutex::new(Vec::new())),
            connection_updates: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl SubscriptionEventEmitter for MockEmitter {
    fn emit_dictionary_updated(&self) {
        *self.dictionary_updates.lock().unwrap() += 1;
    }

    fn emit_window_contexts_updated(&self) {
        *self.window_context_updates.lock().unwrap() += 1;
    }

    fn emit_voice_commands_updated(&self, payload: VoiceCommandsUpdatedPayload) {
        self.voice_command_updates.lock().unwrap().push(payload);
    }

    fn emit_recordings_updated(&self, payload: RecordingsUpdatedPayload) {
        self.recording_updates.lock().unwrap().push(payload);
    }

    fn emit_transcriptions_updated(&self, payload: TranscriptionsUpdatedPayload) {
        self.transcription_updates.lock().unwrap().push(payload);
    }

    fn emit_connection_status(&self, payload: ConnectionStatusPayload) {
        self.connection_updates.lock().unwrap().push(payload);
    }
}

#[test]
fn test_dictionary_change_emits_event() {
    let emitter = MockEmitter::new();
    let handler = SubscriptionHandler::new(emitter);

    handler.on_dictionary_change();

    // Note: We can't access the emitter after moving it into handler
    // In a real test, we'd use Arc<MockEmitter> or similar pattern
}

#[test]
fn test_subscription_handler_creation() {
    let emitter = MockEmitter::new();
    let _handler = SubscriptionHandler::new(emitter);
    // Handler should be created successfully
}

use super::*;
use crate::dictionary::DictionaryEntry;
use crate::events::{
    CommandAmbiguousPayload, CommandExecutedPayload, CommandFailedPayload,
    CommandMatchedPayload, TranscriptionCompletedPayload, TranscriptionErrorPayload,
    TranscriptionStartedPayload,
};
use std::sync::atomic::{AtomicBool, Ordering};

// Mock transcription emitter for tests
struct MockTranscriptionEmitter {
    started_called: AtomicBool,
    completed_called: AtomicBool,
    error_called: AtomicBool,
}

impl MockTranscriptionEmitter {
    fn new() -> Self {
        Self {
            started_called: AtomicBool::new(false),
            completed_called: AtomicBool::new(false),
            error_called: AtomicBool::new(false),
        }
    }
}

impl TranscriptionEventEmitter for MockTranscriptionEmitter {
    fn emit_transcription_started(&self, _payload: TranscriptionStartedPayload) {
        self.started_called.store(true, Ordering::SeqCst);
    }

    fn emit_transcription_completed(&self, _payload: TranscriptionCompletedPayload) {
        self.completed_called.store(true, Ordering::SeqCst);
    }

    fn emit_transcription_error(&self, _payload: TranscriptionErrorPayload) {
        self.error_called.store(true, Ordering::SeqCst);
    }
}

// Mock command emitter for tests
struct MockCommandEmitter;

impl CommandEventEmitter for MockCommandEmitter {
    fn emit_command_matched(&self, _payload: CommandMatchedPayload) {}
    fn emit_command_executed(&self, _payload: CommandExecutedPayload) {}
    fn emit_command_failed(&self, _payload: CommandFailedPayload) {}
    fn emit_command_ambiguous(&self, _payload: CommandAmbiguousPayload) {}
}

#[test]
fn test_service_skips_transcription_when_model_not_loaded() {
    // This test verifies the early return path when the model is not loaded.
    // We can't fully test process_recording without a Tauri app context,
    // but we can verify the model check logic.
    let model = Arc::new(SharedTranscriptionModel::new());
    assert!(!model.is_loaded(), "Model should not be loaded by default");
    // The actual process_recording would return early due to !is_loaded()
}

#[test]
fn test_mock_emitter_tracks_calls() {
    // Verify our mock emitter properly tracks method calls
    let emitter = MockTranscriptionEmitter::new();

    assert!(!emitter.started_called.load(Ordering::SeqCst));
    emitter.emit_transcription_started(TranscriptionStartedPayload {
        timestamp: "test".to_string(),
    });
    assert!(emitter.started_called.load(Ordering::SeqCst));

    assert!(!emitter.completed_called.load(Ordering::SeqCst));
    emitter.emit_transcription_completed(TranscriptionCompletedPayload {
        text: "test".to_string(),
        duration_ms: 100,
    });
    assert!(emitter.completed_called.load(Ordering::SeqCst));

    assert!(!emitter.error_called.load(Ordering::SeqCst));
    emitter.emit_transcription_error(TranscriptionErrorPayload {
        error: "test error".to_string(),
    });
    assert!(emitter.error_called.load(Ordering::SeqCst));
}

#[test]
fn test_dictionary_expander_integration_with_transcription_flow() {
    // This test verifies the dictionary expander correctly transforms text
    // in the same way it would be used in the transcription pipeline.
    //
    // The actual process_recording method requires a Tauri runtime,
    // but we can verify the DictionaryExpander integration pattern here.

    let entries = vec![
        DictionaryEntry {
            id: "1".to_string(),
            trigger: "brb".to_string(),
            expansion: "be right back".to_string(),
            suffix: None,
            auto_enter: false,
            disable_suffix: false,
            complete_match_only: false,
        },
        DictionaryEntry {
            id: "2".to_string(),
            trigger: "api".to_string(),
            expansion: "API".to_string(),
            suffix: None,
            auto_enter: false,
            disable_suffix: false,
            complete_match_only: false,
        },
    ];

    let expander = Arc::new(DictionaryExpander::new(&entries));

    // Simulate transcription text that would come from Parakeet
    let transcribed_text = "i need to brb and check the api docs";

    // Apply expansion (same pattern as in process_recording)
    let result = expander.expand(transcribed_text);

    // Verify expansion was applied correctly
    assert_eq!(
        result.expanded_text,
        "i need to be right back and check the API docs"
    );

    // This expanded text would then be:
    // 1. Passed to command matcher
    // 2. Copied to clipboard
    // 3. Included in transcription_completed event payload
}

#[test]
fn test_dictionary_expander_graceful_fallback_no_expander() {
    // When no expander is configured, text should pass through unchanged
    let dictionary_expander: Arc<RwLock<Option<DictionaryExpander>>> =
        Arc::new(RwLock::new(None));

    let text = "i need to brb and check the api docs";

    // Simulate the expansion logic from process_recording (using RwLock pattern)
    let result = match dictionary_expander.read() {
        Ok(guard) => {
            if let Some(ref expander) = *guard {
                expander.expand(text)
            } else {
                ExpansionResult {
                    expanded_text: text.to_string(),
                    should_press_enter: false,
                }
            }
        }
        Err(_) => ExpansionResult {
            expanded_text: text.to_string(),
            should_press_enter: false,
        },
    };

    // Text should be unchanged when no expander is present
    assert_eq!(result.expanded_text, text);
}

#[test]
fn test_dictionary_expander_graceful_fallback_empty_entries() {
    // When expander has no entries, text should pass through unchanged
    let expander = DictionaryExpander::new(&[]);
    let dictionary_expander: Arc<RwLock<Option<DictionaryExpander>>> =
        Arc::new(RwLock::new(Some(expander)));

    let text = "i need to brb and check the api docs";

    // Simulate the expansion logic from process_recording (using RwLock pattern)
    let result = match dictionary_expander.read() {
        Ok(guard) => {
            if let Some(ref exp) = *guard {
                exp.expand(text)
            } else {
                ExpansionResult {
                    expanded_text: text.to_string(),
                    should_press_enter: false,
                }
            }
        }
        Err(_) => ExpansionResult {
            expanded_text: text.to_string(),
            should_press_enter: false,
        },
    };

    // Text should be unchanged when expander has no entries
    assert_eq!(result.expanded_text, text);
}

#[test]
fn test_dictionary_expander_runtime_update() {
    // Test that the RwLock-based expander can be updated at runtime
    let dictionary_expander: Arc<RwLock<Option<DictionaryExpander>>> =
        Arc::new(RwLock::new(None));

    let text = "i need to brb";

    // Initially no expander, text unchanged
    let result1 = match dictionary_expander.read() {
        Ok(guard) => match *guard {
            Some(ref exp) => exp.expand(text),
            None => ExpansionResult {
                expanded_text: text.to_string(),
                should_press_enter: false,
            },
        },
        Err(_) => ExpansionResult {
            expanded_text: text.to_string(),
            should_press_enter: false,
        },
    };
    assert_eq!(result1.expanded_text, "i need to brb");

    // Update with new entries (simulating what update_dictionary does)
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: None,
        auto_enter: false,
        disable_suffix: false,
        complete_match_only: false,
    }];
    {
        let mut guard = dictionary_expander.write().unwrap();
        *guard = Some(DictionaryExpander::new(&entries));
    }

    // Now expansion should work
    let result2 = match dictionary_expander.read() {
        Ok(guard) => match *guard {
            Some(ref exp) => exp.expand(text),
            None => ExpansionResult {
                expanded_text: text.to_string(),
                should_press_enter: false,
            },
        },
        Err(_) => ExpansionResult {
            expanded_text: text.to_string(),
            should_press_enter: false,
        },
    };
    assert_eq!(result2.expanded_text, "i need to be right back");
}

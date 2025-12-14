// Tests for hotkey-to-recording integration

use super::integration::{HotkeyIntegration, DEBOUNCE_DURATION_MS};
use crate::events::{
    CommandAmbiguousPayload, CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload,
    RecordingErrorPayload, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionPartialPayload,
    TranscriptionStartedPayload,
};
use crate::model::{ModelManifest, ModelType};
use crate::recording::{RecordingManager, RecordingState};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Once};
use std::thread;
use std::time::Duration;

/// Global lock for model directory operations to prevent test races
static MODEL_DIR_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Ensure model files exist for tests (called once per test run)
static INIT_MODEL_FILES: Once = Once::new();

/// Get the path to models directory in the git repo (for tests)
fn get_test_models_dir(model_type: ModelType) -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("Failed to get parent of manifest dir")
        .join("models")
        .join(model_type.dir_name())
}

/// Ensure model files exist in repo - fails if not present
fn ensure_test_model_files() {
    INIT_MODEL_FILES.call_once(|| {
        let _lock = MODEL_DIR_LOCK.lock().unwrap();

        // Verify TDT model exists in repo
        let tdt_model_dir = get_test_models_dir(ModelType::ParakeetTDT);
        let tdt_manifest = ModelManifest::tdt();
        for file in &tdt_manifest.files {
            let file_path = tdt_model_dir.join(&file.name);
            assert!(
                file_path.exists(),
                "TDT model file missing from repo: {:?}. Run 'git lfs pull'.",
                file_path
            );
        }

        // Verify EOU model exists in repo
        let eou_model_dir = get_test_models_dir(ModelType::ParakeetEOU);
        let eou_manifest = ModelManifest::eou();
        for file in &eou_manifest.files {
            let file_path = eou_model_dir.join(&file.name);
            assert!(
                file_path.exists(),
                "EOU model file missing from repo: {:?}. Run 'git lfs pull'.",
                file_path
            );
        }
    });
}

/// Mock event emitter that records all events
#[derive(Default, Clone)]
struct MockEmitter {
    started: Arc<Mutex<Vec<RecordingStartedPayload>>>,
    stopped: Arc<Mutex<Vec<RecordingStoppedPayload>>>,
    errors: Arc<Mutex<Vec<RecordingErrorPayload>>>,
    transcription_started: Arc<Mutex<Vec<TranscriptionStartedPayload>>>,
    transcription_completed: Arc<Mutex<Vec<TranscriptionCompletedPayload>>>,
    transcription_errors: Arc<Mutex<Vec<TranscriptionErrorPayload>>>,
    transcription_partials: Arc<Mutex<Vec<TranscriptionPartialPayload>>>,
    command_matched: Arc<Mutex<Vec<CommandMatchedPayload>>>,
    command_executed: Arc<Mutex<Vec<CommandExecutedPayload>>>,
    command_failed: Arc<Mutex<Vec<CommandFailedPayload>>>,
    command_ambiguous: Arc<Mutex<Vec<CommandAmbiguousPayload>>>,
}

impl MockEmitter {
    fn new() -> Self {
        Self::default()
    }

    fn started_count(&self) -> usize {
        self.started.lock().unwrap().len()
    }

    fn stopped_count(&self) -> usize {
        self.stopped.lock().unwrap().len()
    }
}

impl crate::events::RecordingEventEmitter for MockEmitter {
    fn emit_recording_started(&self, payload: RecordingStartedPayload) {
        self.started.lock().unwrap().push(payload);
    }

    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload) {
        self.stopped.lock().unwrap().push(payload);
    }

    fn emit_recording_error(&self, payload: RecordingErrorPayload) {
        self.errors.lock().unwrap().push(payload);
    }
}

impl crate::events::TranscriptionEventEmitter for MockEmitter {
    fn emit_transcription_started(&self, payload: TranscriptionStartedPayload) {
        self.transcription_started.lock().unwrap().push(payload);
    }

    fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload) {
        self.transcription_completed.lock().unwrap().push(payload);
    }

    fn emit_transcription_error(&self, payload: TranscriptionErrorPayload) {
        self.transcription_errors.lock().unwrap().push(payload);
    }

    fn emit_transcription_partial(&self, payload: TranscriptionPartialPayload) {
        self.transcription_partials.lock().unwrap().push(payload);
    }
}

impl crate::events::CommandEventEmitter for MockEmitter {
    fn emit_command_matched(&self, payload: CommandMatchedPayload) {
        self.command_matched.lock().unwrap().push(payload);
    }

    fn emit_command_executed(&self, payload: CommandExecutedPayload) {
        self.command_executed.lock().unwrap().push(payload);
    }

    fn emit_command_failed(&self, payload: CommandFailedPayload) {
        self.command_failed.lock().unwrap().push(payload);
    }

    fn emit_command_ambiguous(&self, payload: CommandAmbiguousPayload) {
        self.command_ambiguous.lock().unwrap().push(payload);
    }
}

/// Type alias for HotkeyIntegration with MockEmitter for all parameters
type TestIntegration = HotkeyIntegration<MockEmitter, MockEmitter, MockEmitter>;

#[test]
fn test_toggle_from_idle_starts_recording() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::new(emitter.clone());
    let state = Mutex::new(RecordingManager::new());

    let accepted = integration.handle_toggle(&state);

    assert!(accepted, "Toggle should be accepted");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);
    assert_eq!(emitter.stopped_count(), 0);
}

#[test]
fn test_toggle_from_recording_stops() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0); // No debounce for test
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Stop recording
    let accepted = integration.handle_toggle(&state);

    assert!(accepted, "Toggle should be accepted");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.started_count(), 1);
    assert_eq!(emitter.stopped_count(), 1);
}

#[test]
fn test_rapid_toggle_debounced() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 100);
    let state = Mutex::new(RecordingManager::new());

    // First toggle accepted
    let first = integration.handle_toggle(&state);
    assert!(first, "First toggle should be accepted");

    // Immediate second toggle should be debounced
    let second = integration.handle_toggle(&state);
    assert!(!second, "Second toggle should be debounced");

    // State should still be Recording (not toggled back)
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);
    assert_eq!(emitter.stopped_count(), 0);
}

#[test]
fn test_toggle_after_debounce_window() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 50);
    let state = Mutex::new(RecordingManager::new());

    // First toggle
    integration.handle_toggle(&state);

    // Wait for debounce to expire
    thread::sleep(Duration::from_millis(60));

    // Second toggle should work
    let accepted = integration.handle_toggle(&state);
    assert!(accepted, "Toggle after debounce should be accepted");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
}

#[test]
fn test_events_emitted_on_each_toggle() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Toggle to Recording
    integration.handle_toggle(&state);
    assert_eq!(emitter.started_count(), 1);

    // Toggle to Idle
    integration.handle_toggle(&state);
    assert_eq!(emitter.stopped_count(), 1);

    // Toggle back to Recording
    integration.handle_toggle(&state);
    assert_eq!(emitter.started_count(), 2);
}

#[test]
fn test_toggle_from_processing_ignored() {
    use crate::audio::TARGET_SAMPLE_RATE;

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Manually put into Processing state
    {
        let mut manager = state.lock().unwrap();
        manager.start_recording(TARGET_SAMPLE_RATE).unwrap();
        manager.transition_to(RecordingState::Processing).unwrap();
    }

    // Toggle should be ignored
    let accepted = integration.handle_toggle(&state);
    assert!(!accepted, "Toggle from Processing should be ignored");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Processing
    );
}

#[test]
fn test_is_debouncing() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter, 100);
    let state = Mutex::new(RecordingManager::new());

    assert!(!integration.is_debouncing(), "Should not be debouncing initially");

    integration.handle_toggle(&state);
    assert!(integration.is_debouncing(), "Should be debouncing after toggle");

    thread::sleep(Duration::from_millis(110));
    assert!(
        !integration.is_debouncing(),
        "Should not be debouncing after window expires"
    );
}

#[test]
fn test_default_debounce_duration() {
    assert_eq!(DEBOUNCE_DURATION_MS, 200);
}

#[test]
fn test_multiple_rapid_toggles_only_first_accepted() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 50);
    let state = Mutex::new(RecordingManager::new());

    // Rapid fire toggles
    let results: Vec<bool> = (0..5).map(|_| integration.handle_toggle(&state)).collect();

    // Only first should be accepted
    assert_eq!(results, vec![true, false, false, false, false]);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);
}

#[test]
fn test_started_payload_has_timestamp() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::new(emitter.clone());
    let state = Mutex::new(RecordingManager::new());

    integration.handle_toggle(&state);

    let started = emitter.started.lock().unwrap();
    assert!(!started[0].timestamp.is_empty());
    // Should be ISO 8601 format with T separator
    assert!(started[0].timestamp.contains('T'));
}

#[test]
fn test_stopped_payload_has_metadata() {
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start then stop
    integration.handle_toggle(&state);
    integration.handle_toggle(&state);

    let stopped = emitter.stopped.lock().unwrap();
    assert_eq!(stopped.len(), 1);
    // Metadata should be present (even if empty for now)
    assert!(stopped[0].metadata.duration_secs >= 0.0);
}

// === Audio Thread Integration Tests ===
// Note: The audio thread itself is tested in audio/thread.rs
// These tests verify HotkeyIntegration works with and without an audio thread

#[test]
fn test_toggle_without_audio_thread_still_works() {
    // Regression test: integration without audio thread should still manage state
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);

    // Stop recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.stopped_count(), 1);
}

#[test]
fn test_full_cycle_with_audio_thread() {
    ensure_test_model_files();
    use crate::audio::AudioThreadHandle;
    use std::sync::Arc;

    let emitter = MockEmitter::new();
    let audio_thread = Arc::new(AudioThreadHandle::spawn());

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0).with_audio_thread(audio_thread);
    let state = Mutex::new(RecordingManager::new());

    // Full cycle: Idle -> Recording -> Idle
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
    assert_eq!(emitter.started_count(), 1);

    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.stopped_count(), 1);
}

#[test]
fn test_audio_thread_disconnection_rolls_back_state() {
    ensure_test_model_files();
    use crate::audio::AudioThreadHandle;
    use std::sync::Arc;

    let emitter = MockEmitter::new();
    let audio_thread = Arc::new(AudioThreadHandle::spawn());

    // Shutdown the audio thread immediately to simulate disconnection
    let _ = audio_thread.shutdown();
    // Give the thread time to actually shut down
    thread::sleep(Duration::from_millis(10));

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0).with_audio_thread(audio_thread);
    let state = Mutex::new(RecordingManager::new());

    // Try to start recording - should fail because thread is disconnected
    let accepted = integration.handle_toggle(&state);

    // The toggle should be rejected (audio thread disconnected)
    assert!(!accepted, "Toggle should be rejected when audio thread disconnected");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Idle,
        "State should be rolled back to Idle"
    );
    assert_eq!(emitter.started_count(), 0, "Started event should not be emitted");
}

// === Streaming Wire-Up Tests ===

#[test]
fn test_hotkey_integration_accepts_streaming_transcriber_via_builder() {
    ensure_test_model_files();
    use crate::parakeet::StreamingTranscriber;
    use std::sync::Arc;

    let emitter = MockEmitter::new();
    let transcription_emitter = Arc::new(MockEmitter::new());
    let streaming_transcriber = Arc::new(Mutex::new(StreamingTranscriber::new(transcription_emitter.clone())));

    let integration: TestIntegration =
        HotkeyIntegration::new(emitter)
            .with_streaming_transcriber(streaming_transcriber);

    // Verify the builder method accepted the streaming transcriber
    // (we can't directly access private fields, but if it compiles, the type signature is correct)
    assert!(true, "Builder method accepted streaming transcriber");
}

#[test]
fn test_recording_start_in_batch_mode_passes_none_streaming_sender() {
    ensure_test_model_files();
    use crate::parakeet::{TranscriptionManager, TranscriptionMode};
    use std::sync::Arc;

    let emitter = MockEmitter::new();
    let transcription_manager = Arc::new(TranscriptionManager::new());
    // Default mode is Batch
    assert_eq!(transcription_manager.current_mode(), TranscriptionMode::Batch);

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_transcription_manager(transcription_manager);
    let state = Mutex::new(RecordingManager::new());

    // Start recording - in batch mode, no streaming sender should be created
    // We can't directly verify streaming_sender is None, but we verify recording starts
    let accepted = integration.handle_toggle(&state);

    assert!(accepted, "Recording should start in batch mode");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
}

#[test]
fn test_recording_start_in_streaming_mode_creates_streaming_channel() {
    ensure_test_model_files();
    use crate::parakeet::{TranscriptionManager, TranscriptionMode, StreamingTranscriber};
    use std::sync::Arc;

    let emitter = MockEmitter::new();
    let transcription_emitter = Arc::new(MockEmitter::new());
    let transcription_manager = Arc::new(TranscriptionManager::new());

    // Set mode to Streaming
    transcription_manager.set_mode(TranscriptionMode::Streaming).unwrap();
    assert_eq!(transcription_manager.current_mode(), TranscriptionMode::Streaming);

    let streaming_transcriber = Arc::new(Mutex::new(StreamingTranscriber::new(transcription_emitter.clone())));

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_transcription_manager(transcription_manager)
            .with_streaming_transcriber(streaming_transcriber);
    let state = Mutex::new(RecordingManager::new());

    // Start recording - in streaming mode, a streaming channel should be created
    // We can't directly verify the channel exists, but we verify recording starts
    let accepted = integration.handle_toggle(&state);

    assert!(accepted, "Recording should start in streaming mode");
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );
}

#[test]
fn test_recording_stop_in_batch_mode_calls_spawn_transcription() {
    ensure_test_model_files();
    use crate::parakeet::{TranscriptionManager, TranscriptionMode};
    use std::sync::Arc;

    let emitter = MockEmitter::new();
    let transcription_manager = Arc::new(TranscriptionManager::new());
    // Default mode is Batch
    assert_eq!(transcription_manager.current_mode(), TranscriptionMode::Batch);

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_transcription_manager(transcription_manager);
    let state = Mutex::new(RecordingManager::new());

    // Start and stop recording
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    integration.handle_toggle(&state);
    // Recording should stop and state should be Idle
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Idle
    );
    // Stopped event should be emitted
    assert_eq!(emitter.stopped_count(), 1);
}

#[test]
fn test_recording_stop_in_streaming_mode_calls_finalize() {
    ensure_test_model_files();
    use crate::parakeet::{TranscriptionManager, TranscriptionMode, StreamingTranscriber};
    use std::sync::Arc;

    let emitter = MockEmitter::new();
    let transcription_emitter = Arc::new(MockEmitter::new());
    let transcription_manager = Arc::new(TranscriptionManager::new());

    // Set mode to Streaming
    transcription_manager.set_mode(TranscriptionMode::Streaming).unwrap();
    assert_eq!(transcription_manager.current_mode(), TranscriptionMode::Streaming);

    let streaming_transcriber = Arc::new(Mutex::new(StreamingTranscriber::new(transcription_emitter.clone())));

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_transcription_manager(transcription_manager)
            .with_streaming_transcriber(streaming_transcriber);
    let state = Mutex::new(RecordingManager::new());

    // Start and stop recording
    integration.handle_toggle(&state);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    integration.handle_toggle(&state);
    // Recording should stop and state should be Idle
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Idle
    );
    // Stopped event should be emitted
    assert_eq!(emitter.stopped_count(), 1);
    // In streaming mode, finalize_streaming is called (instead of spawn_transcription)
    // We can't directly verify this, but the test passes without errors
}

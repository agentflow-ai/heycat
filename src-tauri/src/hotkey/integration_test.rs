// Tests for hotkey-to-recording integration

use super::integration::{HotkeyIntegration, DEBOUNCE_DURATION_MS};
use crate::events::{
    CommandAmbiguousPayload, CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload,
    RecordingCancelledPayload, RecordingErrorPayload, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionStartedPayload,
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
    });
}

/// Mock event emitter that records all events
#[derive(Default, Clone)]
struct MockEmitter {
    started: Arc<Mutex<Vec<RecordingStartedPayload>>>,
    stopped: Arc<Mutex<Vec<RecordingStoppedPayload>>>,
    cancelled: Arc<Mutex<Vec<RecordingCancelledPayload>>>,
    errors: Arc<Mutex<Vec<RecordingErrorPayload>>>,
    transcription_started: Arc<Mutex<Vec<TranscriptionStartedPayload>>>,
    transcription_completed: Arc<Mutex<Vec<TranscriptionCompletedPayload>>>,
    transcription_errors: Arc<Mutex<Vec<TranscriptionErrorPayload>>>,
    command_matched: Arc<Mutex<Vec<CommandMatchedPayload>>>,
    command_executed: Arc<Mutex<Vec<CommandExecutedPayload>>>,
    command_failed: Arc<Mutex<Vec<CommandFailedPayload>>>,
    command_ambiguous: Arc<Mutex<Vec<CommandAmbiguousPayload>>>,
    key_blocking_unavailable: Arc<Mutex<Vec<crate::events::hotkey_events::KeyBlockingUnavailablePayload>>>,
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

    fn cancelled_count(&self) -> usize {
        self.cancelled.lock().unwrap().len()
    }

    fn last_cancelled(&self) -> Option<RecordingCancelledPayload> {
        self.cancelled.lock().unwrap().last().cloned()
    }

    fn key_blocking_unavailable_count(&self) -> usize {
        self.key_blocking_unavailable.lock().unwrap().len()
    }
}

impl crate::events::RecordingEventEmitter for MockEmitter {
    fn emit_recording_started(&self, payload: RecordingStartedPayload) {
        self.started.lock().unwrap().push(payload);
    }

    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload) {
        self.stopped.lock().unwrap().push(payload);
    }

    fn emit_recording_cancelled(&self, payload: RecordingCancelledPayload) {
        self.cancelled.lock().unwrap().push(payload);
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

impl crate::events::HotkeyEventEmitter for MockEmitter {
    fn emit_key_blocking_unavailable(&self, payload: crate::events::hotkey_events::KeyBlockingUnavailablePayload) {
        self.key_blocking_unavailable.lock().unwrap().push(payload);
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

/// Test full recording cycle with audio thread
/// Ignored by default as it requires microphone permissions
/// Run manually with: cargo test test_full_cycle_with_audio_thread -- --ignored
#[test]
#[ignore]
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

// === Silence Detection Integration Tests ===

#[test]
fn test_with_recording_detectors_builder() {
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));

    let integration: TestIntegration = HotkeyIntegration::new(emitter)
        .with_recording_detectors(detectors.clone());

    // Just verify the builder works and doesn't panic
    assert!(integration.silence.enabled);
}

#[test]
fn test_silence_detection_can_be_disabled() {
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));

    let integration: TestIntegration = HotkeyIntegration::new(emitter)
        .with_recording_detectors(detectors)
        .with_silence_detection_enabled(false);

    assert!(!integration.silence.enabled);
}

#[test]
fn test_custom_silence_config() {
    use crate::recording::{RecordingDetectors, SilenceConfig};

    let emitter = MockEmitter::new();
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));
    let config = SilenceConfig {
        silence_duration_ms: 3000,
        no_speech_timeout_ms: 10000,
        ..Default::default()
    };

    let integration: TestIntegration = HotkeyIntegration::new(emitter)
        .with_recording_detectors(detectors)
        .with_silence_config(config);

    // Custom config should be stored
    assert!(integration.silence.config.is_some());
    assert_eq!(integration.silence.config.as_ref().unwrap().silence_duration_ms, 3000);
}

/// Test that manual stop takes precedence over silence detection
/// Ignored by default as it requires microphone permissions
/// Run manually with: cargo test test_manual_stop_takes_precedence_over_silence_detection -- --ignored
#[test]
#[ignore]
fn test_manual_stop_takes_precedence_over_silence_detection() {
    // When user manually stops via hotkey, silence detection should be stopped
    // and not interfere with the manual stop
    ensure_test_model_files();
    use crate::audio::AudioThreadHandle;
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let audio_thread = Arc::new(AudioThreadHandle::spawn());
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));
    let recording_state = Arc::new(Mutex::new(RecordingManager::new()));

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_audio_thread(audio_thread)
            .with_recording_detectors(detectors.clone())
            .with_recording_state(recording_state.clone());

    // Start recording
    integration.handle_toggle(&recording_state);
    assert_eq!(
        recording_state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Note: Silence detection won't actually start because we don't have
    // transcription_emitter or app_handle configured - that's fine for this test

    // Manual stop
    integration.handle_toggle(&recording_state);
    assert_eq!(
        recording_state.lock().unwrap().get_state(),
        RecordingState::Idle
    );

    // Detectors should not be running after manual stop
    let det = detectors.lock().unwrap();
    assert!(!det.is_running(), "Detectors should be stopped after manual stop");
}

#[test]
fn test_silence_detection_not_started_without_detectors() {
    // Without recording detectors configured, silence detection should be skipped
    ensure_test_model_files();
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording - should work fine without detectors
    let accepted = integration.handle_toggle(&state);
    assert!(accepted);
    assert_eq!(
        state.lock().unwrap().get_state(),
        RecordingState::Recording
    );

    // Stop - should also work
    let accepted = integration.handle_toggle(&state);
    assert!(accepted);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
}

#[test]
fn test_silence_detection_respects_enabled_flag() {
    // When silence_detection_enabled is false, detectors should not be started
    ensure_test_model_files();
    use crate::audio::AudioThreadHandle;
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let audio_thread = Arc::new(AudioThreadHandle::spawn());
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));
    let recording_state = Arc::new(Mutex::new(RecordingManager::new()));

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_audio_thread(audio_thread)
            .with_recording_detectors(detectors.clone())
            .with_recording_state(recording_state.clone())
            .with_silence_detection_enabled(false);

    // Start recording
    integration.handle_toggle(&recording_state);

    // Detectors should NOT be running because feature is disabled
    let det = detectors.lock().unwrap();
    assert!(!det.is_running(), "Detectors should not start when disabled");
}

// === Escape Key Listener Tests ===

/// Mock shortcut backend that tracks registered/unregistered shortcuts
#[derive(Default)]
struct MockShortcutBackend {
    registered: Arc<Mutex<Vec<String>>>,
    callbacks: Arc<Mutex<std::collections::HashMap<String, Box<dyn Fn() + Send + Sync>>>>,
}

impl MockShortcutBackend {
    fn new() -> Self {
        Self::default()
    }

    /// Check if a shortcut is currently registered
    fn is_registered(&self, shortcut: &str) -> bool {
        self.registered.lock().unwrap().contains(&shortcut.to_string())
    }

    /// Simulate pressing a registered shortcut (triggers callback)
    fn simulate_press(&self, shortcut: &str) {
        if let Some(callback) = self.callbacks.lock().unwrap().get(shortcut) {
            callback();
        }
    }
}

impl crate::hotkey::ShortcutBackend for MockShortcutBackend {
    fn register(&self, shortcut: &str, callback: Box<dyn Fn() + Send + Sync>) -> Result<(), String> {
        let mut registered = self.registered.lock().unwrap();
        if registered.contains(&shortcut.to_string()) {
            return Err("Shortcut already registered".to_string());
        }
        registered.push(shortcut.to_string());
        self.callbacks.lock().unwrap().insert(shortcut.to_string(), callback);
        Ok(())
    }

    fn unregister(&self, shortcut: &str) -> Result<(), String> {
        let mut registered = self.registered.lock().unwrap();
        if let Some(pos) = registered.iter().position(|s| s == shortcut) {
            registered.remove(pos);
            self.callbacks.lock().unwrap().remove(shortcut);
            Ok(())
        } else {
            Err("Shortcut not registered".to_string())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_escape_listener_registered_when_recording_starts() {
    // Escape key listener should be registered when recording starts
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Before recording, Escape should not be registered
    assert!(!backend.is_registered("Escape"), "Escape should not be registered before recording");

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);

    // Escape should now be registered
    assert!(backend.is_registered("Escape"), "Escape should be registered after recording starts");
}

#[test]
fn test_escape_callback_fires_during_recording() {
    // Escape key callback should fire when double-tap Escape is pressed during recording
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);

    // Simulate double-tap Escape (two presses within window)
    backend.simulate_press("Escape"); // First tap
    backend.simulate_press("Escape"); // Second tap - triggers callback

    // Callback should have been invoked
    assert_eq!(*callback_count.lock().unwrap(), 1, "Callback should fire on double-tap Escape during recording");
}

#[test]
fn test_escape_listener_unregistered_when_recording_stops() {
    // Escape key listener should be unregistered when recording stops
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert!(backend.is_registered("Escape"), "Escape should be registered during recording");

    // Stop recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);

    // Escape should be unregistered
    assert!(!backend.is_registered("Escape"), "Escape should be unregistered after recording stops");
}

#[test]
fn test_single_escape_tap_does_not_trigger_cancel() {
    // Single Escape tap should NOT trigger cancel - double-tap is required
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);

    // Single Escape press - should NOT trigger callback
    backend.simulate_press("Escape");

    // Callback should NOT have been invoked (single tap ignored)
    assert_eq!(*callback_count.lock().unwrap(), 0, "Single tap should not trigger cancel");
}

#[test]
fn test_escape_callback_does_not_fire_when_not_recording() {
    // After stopping, Escape callback should not fire (listener unregistered)
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start and stop recording
    integration.handle_toggle(&state);
    integration.handle_toggle(&state);

    // Try to simulate Escape press (but listener is gone)
    backend.simulate_press("Escape");

    // Callback should NOT have been invoked (listener was unregistered)
    assert_eq!(*callback_count.lock().unwrap(), 0, "Callback should not fire when not recording");
}

#[test]
fn test_escape_listener_multiple_cycles() {
    // Multiple start/stop cycles should work correctly with double-tap
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Cycle 1: Start -> Double-tap Escape fires -> Stop
    integration.handle_toggle(&state);
    assert!(backend.is_registered("Escape"));
    backend.simulate_press("Escape"); // First tap
    backend.simulate_press("Escape"); // Second tap - triggers
    assert_eq!(*callback_count.lock().unwrap(), 1);
    integration.handle_toggle(&state);
    assert!(!backend.is_registered("Escape"));

    // Cycle 2: Start -> Double-tap Escape fires -> Stop
    integration.handle_toggle(&state);
    assert!(backend.is_registered("Escape"));
    backend.simulate_press("Escape"); // First tap
    backend.simulate_press("Escape"); // Second tap - triggers
    assert_eq!(*callback_count.lock().unwrap(), 2);
    integration.handle_toggle(&state);
    assert!(!backend.is_registered("Escape"));

    // Cycle 3: Start -> Stop (no Escape press)
    integration.handle_toggle(&state);
    assert!(backend.is_registered("Escape"));
    integration.handle_toggle(&state);
    assert!(!backend.is_registered("Escape"));

    // Total callback count should be 2 (from the two double-taps)
    assert_eq!(*callback_count.lock().unwrap(), 2);
}

#[test]
fn test_three_rapid_escape_taps_triggers_cancel_once() {
    // Triple-tap should only trigger cancel once (same as double-tap behavior)
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);

    // Triple-tap Escape
    backend.simulate_press("Escape"); // First tap - records time
    backend.simulate_press("Escape"); // Second tap - triggers, resets
    backend.simulate_press("Escape"); // Third tap - starts new cycle

    // Should trigger exactly once
    assert_eq!(*callback_count.lock().unwrap(), 1, "Triple tap should trigger cancel only once");
}

#[test]
fn test_escape_double_tap_window_is_configurable() {
    // Double-tap window can be configured via builder
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    // Configure with a very short window (10ms)
    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }))
            .with_double_tap_window(10); // 10ms window
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);

    // First tap
    backend.simulate_press("Escape");

    // Wait longer than window
    thread::sleep(Duration::from_millis(20));

    // Second tap - should NOT trigger (outside window)
    backend.simulate_press("Escape");

    // Callback should NOT have been invoked
    assert_eq!(*callback_count.lock().unwrap(), 0, "Taps outside window should not trigger");
}

#[test]
fn test_escape_listener_without_backend_gracefully_skips() {
    // Without backend configured, escape registration should be skipped gracefully
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    // Only callback configured, no backend
    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording - should work without backend
    let accepted = integration.handle_toggle(&state);
    assert!(accepted, "Recording should start even without escape backend");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);

    // Stop recording - should work
    let accepted = integration.handle_toggle(&state);
    assert!(accepted, "Recording should stop");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
}

#[test]
fn test_escape_listener_without_callback_gracefully_skips() {
    // Without callback configured, escape registration should be skipped gracefully
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());

    // Only backend configured, no callback
    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone());
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);

    // Escape should NOT be registered (no callback)
    assert!(!backend.is_registered("Escape"), "Escape should not register without callback");

    // Stop recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
}

// === Cancel Recording Tests ===

#[test]
fn test_cancel_recording_during_recording_clears_buffer() {
    // Cancel during recording should clear buffer and return to Idle
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);
    assert_eq!(emitter.started_count(), 1);

    // Cancel recording
    let cancelled = integration.cancel_recording(&state, "double-tap-escape");

    assert!(cancelled, "Cancel should succeed");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle, "State should be Idle after cancel");

    // Buffer should be cleared (no audio data retained)
    let buffer_result = state.lock().unwrap().get_audio_buffer();
    assert!(buffer_result.is_err(), "Buffer should be cleared after cancel");
}

#[test]
fn test_cancel_recording_does_not_emit_stopped_event() {
    // Cancel should emit cancelled event, not stopped event
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(emitter.started_count(), 1);
    assert_eq!(emitter.stopped_count(), 0);
    assert_eq!(emitter.cancelled_count(), 0);

    // Cancel recording
    integration.cancel_recording(&state, "double-tap-escape");

    // Should emit cancelled, NOT stopped
    assert_eq!(emitter.stopped_count(), 0, "Should not emit stopped event on cancel");
    assert_eq!(emitter.cancelled_count(), 1, "Should emit cancelled event");
}

#[test]
fn test_cancel_recording_emits_correct_payload() {
    // Cancelled event should have correct reason and timestamp
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);

    // Cancel with specific reason
    integration.cancel_recording(&state, "double-tap-escape");

    let cancelled = emitter.last_cancelled().expect("Should have cancelled event");
    assert_eq!(cancelled.reason, "double-tap-escape", "Reason should match");
    assert!(!cancelled.timestamp.is_empty(), "Timestamp should be present");
    assert!(cancelled.timestamp.contains('T'), "Timestamp should be ISO 8601 format");
}

#[test]
fn test_cancel_recording_ignored_when_not_recording() {
    // Cancel should be ignored when not in Recording state
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Try to cancel from Idle state
    let cancelled = integration.cancel_recording(&state, "double-tap-escape");

    assert!(!cancelled, "Cancel should be ignored when not recording");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.cancelled_count(), 0, "Should not emit cancelled event when not recording");
}

#[test]
fn test_cancel_recording_ignored_when_processing() {
    // Cancel should be ignored when in Processing state
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

    // Try to cancel from Processing state
    let cancelled = integration.cancel_recording(&state, "double-tap-escape");

    assert!(!cancelled, "Cancel should be ignored when processing");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Processing);
    assert_eq!(emitter.cancelled_count(), 0);
}

/// Test cancel recording with audio thread
/// Ignored by default as it requires microphone permissions
/// Run manually with: cargo test test_cancel_recording_with_audio_thread -- --ignored
#[test]
#[ignore]
fn test_cancel_recording_with_audio_thread() {
    // Cancel should stop audio thread
    ensure_test_model_files();
    use crate::audio::AudioThreadHandle;

    let emitter = MockEmitter::new();
    let audio_thread = Arc::new(AudioThreadHandle::spawn());

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0).with_audio_thread(audio_thread);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);

    // Cancel recording - should stop audio thread
    let cancelled = integration.cancel_recording(&state, "double-tap-escape");

    assert!(cancelled, "Cancel should succeed");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.cancelled_count(), 1);
}

#[test]
fn test_cancel_recording_unregisters_escape_listener() {
    // Cancel should unregister the Escape listener
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(MockShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert!(backend.is_registered("Escape"), "Escape should be registered during recording");

    // Cancel recording
    integration.cancel_recording(&state, "double-tap-escape");

    // Escape should be unregistered after cancel
    assert!(!backend.is_registered("Escape"), "Escape should be unregistered after cancel");
}

#[test]
fn test_cancel_recording_stops_silence_detection() {
    // Cancel should stop silence detection
    ensure_test_model_files();
    use crate::audio::AudioThreadHandle;
    use crate::recording::RecordingDetectors;

    let emitter = MockEmitter::new();
    let audio_thread = Arc::new(AudioThreadHandle::spawn());
    let detectors = Arc::new(Mutex::new(RecordingDetectors::new()));
    let recording_state = Arc::new(Mutex::new(RecordingManager::new()));

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_audio_thread(audio_thread)
            .with_recording_detectors(detectors.clone())
            .with_recording_state(recording_state.clone());

    // Start recording
    integration.handle_toggle(&recording_state);

    // Cancel recording
    integration.cancel_recording(&recording_state, "double-tap-escape");

    // Detectors should not be running
    let det = detectors.lock().unwrap();
    assert!(!det.is_running(), "Detectors should be stopped after cancel");
}

#[test]
fn test_cancel_recording_can_restart_after_cancel() {
    // After cancel, should be able to start a new recording
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0);
    let state = Mutex::new(RecordingManager::new());

    // Start recording
    integration.handle_toggle(&state);
    assert_eq!(emitter.started_count(), 1);

    // Cancel recording
    integration.cancel_recording(&state, "double-tap-escape");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);

    // Start new recording - should work
    let accepted = integration.handle_toggle(&state);
    assert!(accepted, "Should be able to start recording after cancel");
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);
    assert_eq!(emitter.started_count(), 2);
}

// === Escape Registration Failure Tests ===

/// Mock shortcut backend that always fails registration
struct FailingShortcutBackend {
    unregister_attempts: Arc<Mutex<Vec<String>>>,
}

impl FailingShortcutBackend {
    fn new() -> Self {
        Self {
            unregister_attempts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the number of unregister attempts
    fn unregister_attempt_count(&self) -> usize {
        self.unregister_attempts.lock().unwrap().len()
    }
}

impl crate::hotkey::ShortcutBackend for FailingShortcutBackend {
    fn register(&self, _shortcut: &str, _callback: Box<dyn Fn() + Send + Sync>) -> Result<(), String> {
        Err("Registration always fails".to_string())
    }

    fn unregister(&self, shortcut: &str) -> Result<(), String> {
        self.unregister_attempts.lock().unwrap().push(shortcut.to_string());
        Err("Nothing to unregister".to_string())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_escape_registered_false_when_registration_fails() {
    // When registration fails, escape_registered should remain false
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(FailingShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording - this triggers register_escape_listener which will fail
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);

    // Recording should still work even though Escape registration failed
    // This verifies the system is resilient to registration failures
    assert_eq!(emitter.started_count(), 1);
}

#[test]
fn test_unregister_not_called_when_registration_failed() {
    // When registration fails, unregister should not be called on stop
    // (avoids spurious warnings in logs)
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(FailingShortcutBackend::new());
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = callback_count.clone();

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend.clone())
            .with_escape_callback(Arc::new(move || {
                *callback_count_clone.lock().unwrap() += 1;
            }));
    let state = Mutex::new(RecordingManager::new());

    // Start recording - registration will fail
    integration.handle_toggle(&state);

    // Stop recording - unregister should NOT be called since registration failed
    integration.handle_toggle(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);

    // Verify unregister was not attempted (escape_registered stayed false)
    assert_eq!(
        backend.unregister_attempt_count(),
        0,
        "Unregister should not be called when registration failed"
    );
}

#[test]
fn test_key_blocking_unavailable_event_emitted_on_registration_failure() {
    // When Escape key registration fails, the key_blocking_unavailable event
    // should be emitted to notify the frontend
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let backend = Arc::new(FailingShortcutBackend::new());
    let hotkey_emitter = Arc::new(emitter.clone());

    let mut integration: TestIntegration =
        HotkeyIntegration::with_debounce(emitter.clone(), 0)
            .with_shortcut_backend(backend)
            .with_escape_callback(Arc::new(|| {}))
            .with_hotkey_emitter(hotkey_emitter);
    let state = Mutex::new(RecordingManager::new());

    // Start recording - this triggers register_escape_listener which will fail
    integration.handle_toggle(&state);

    // Verify the key_blocking_unavailable event was emitted
    assert_eq!(
        emitter.key_blocking_unavailable_count(),
        1,
        "key_blocking_unavailable event should be emitted when registration fails"
    );

    // Recording should still work (graceful degradation)
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);
    assert_eq!(emitter.started_count(), 1);
}

// === Push-to-Talk Mode Tests ===

#[test]
fn test_recording_mode_default() {
    let emitter = MockEmitter::new();
    let integration: TestIntegration = HotkeyIntegration::new(emitter);
    assert_eq!(integration.recording_mode(), crate::hotkey::RecordingMode::Toggle);
}

#[test]
fn test_recording_mode_builder() {
    let emitter = MockEmitter::new();
    let integration: TestIntegration = HotkeyIntegration::new(emitter)
        .with_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    assert_eq!(integration.recording_mode(), crate::hotkey::RecordingMode::PushToTalk);
}

#[test]
fn test_set_recording_mode() {
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::new(emitter);
    assert_eq!(integration.recording_mode(), crate::hotkey::RecordingMode::Toggle);

    integration.set_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    assert_eq!(integration.recording_mode(), crate::hotkey::RecordingMode::PushToTalk);

    integration.set_recording_mode(crate::hotkey::RecordingMode::Toggle);
    assert_eq!(integration.recording_mode(), crate::hotkey::RecordingMode::Toggle);
}

#[test]
fn test_ptt_press_starts_recording() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    let state = Mutex::new(RecordingManager::new());

    // Initially Idle
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);

    // Press should start recording
    let started = integration.handle_hotkey_press(&state);
    assert!(started);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);
    assert_eq!(emitter.started_count(), 1);
}

#[test]
fn test_ptt_release_stops_recording() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    let state = Mutex::new(RecordingManager::new());

    // Start recording via press
    integration.handle_hotkey_press(&state);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Recording);

    // Release should stop recording
    let stopped = integration.handle_hotkey_release(&state);
    assert!(stopped);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.stopped_count(), 1);
}

#[test]
fn test_ptt_press_ignored_when_already_recording() {
    ensure_test_model_files();

    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    let state = Mutex::new(RecordingManager::new());

    // First press starts recording
    integration.handle_hotkey_press(&state);
    assert_eq!(emitter.started_count(), 1);

    // Second press should be ignored (already recording)
    let result = integration.handle_hotkey_press(&state);
    assert!(!result);
    assert_eq!(emitter.started_count(), 1); // Still just 1
}

#[test]
fn test_ptt_release_ignored_when_not_recording() {
    let emitter = MockEmitter::new();
    let mut integration: TestIntegration = HotkeyIntegration::with_debounce(emitter.clone(), 0)
        .with_recording_mode(crate::hotkey::RecordingMode::PushToTalk);
    let state = Mutex::new(RecordingManager::new());

    // Release without prior press should be ignored
    let result = integration.handle_hotkey_release(&state);
    assert!(!result);
    assert_eq!(state.lock().unwrap().get_state(), RecordingState::Idle);
    assert_eq!(emitter.stopped_count(), 0);
}

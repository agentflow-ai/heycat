// SharedTranscriptionModel for thread-safe Parakeet model sharing
// Eliminates duplicate model instances (~3GB memory savings)

use parakeet_rs::ParakeetTDT;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::types::{TranscriptionError, TranscriptionResult, TranscriptionState};
use super::utils::fix_parakeet_text;
use crate::info;

// ============================================================================
// TranscribingGuard - RAII guard for state transitions
// ============================================================================

/// RAII guard that manages transcription state transitions.
///
/// This guard ensures that:
/// - State is set to `Transcribing` only when the guard is acquired
/// - State is automatically reset to `Idle` when the guard is dropped
/// - Panics during transcription don't leave the state stuck
/// - Explicit errors can be recorded via `complete_with_error`
///
/// # Example
/// ```ignore
/// fn transcribe(&self) -> Result<String, TranscriptionError> {
///     let _guard = TranscribingGuard::new(&self.state)?;
///     // State is now Transcribing
///
///     let result = self.do_work()?;
///     // Guard drops here, state becomes Idle
///     Ok(result)
/// }
/// ```
pub struct TranscribingGuard {
    state: Arc<Mutex<TranscriptionState>>,
    completed: bool,
}

impl TranscribingGuard {
    /// Create a new TranscribingGuard, setting state to Transcribing.
    ///
    /// # Errors
    /// - `LockPoisoned` if the state mutex is poisoned
    /// - `ModelNotLoaded` if the model is in Unloaded state
    pub fn new(state: Arc<Mutex<TranscriptionState>>) -> TranscriptionResult<Self> {
        let mut guard = state.lock().map_err(|_| TranscriptionError::LockPoisoned)?;
        if *guard == TranscriptionState::Unloaded {
            return Err(TranscriptionError::ModelNotLoaded);
        }
        *guard = TranscriptionState::Transcribing;
        drop(guard); // Release the lock before returning
        Ok(Self {
            state,
            completed: false,
        })
    }

    /// Mark the transcription as completed successfully.
    ///
    /// Sets state to `Completed` and marks the guard as completed
    /// so it won't reset to `Idle` on drop.
    pub fn complete_success(&mut self) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = TranscriptionState::Completed;
        }
        self.completed = true;
    }

    /// Mark the transcription as failed with an error.
    ///
    /// Sets state to `Error` and marks the guard as completed
    /// so it won't reset to `Idle` on drop.
    pub fn complete_with_error(&mut self) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = TranscriptionState::Error;
        }
        self.completed = true;
    }
}

impl Drop for TranscribingGuard {
    fn drop(&mut self) {
        // Only reset to Idle if we didn't explicitly complete
        // This handles both normal completion (where we want Idle)
        // and panics (where we want to reset from Transcribing)
        if !self.completed {
            if let Ok(mut guard) = self.state.lock() {
                // Only reset if still in Transcribing state
                // (in case someone else changed it)
                if *guard == TranscriptionState::Transcribing {
                    *guard = TranscriptionState::Idle;
                }
            }
        }
    }
}

/// Shared transcription model wrapper for ParakeetTDT
///
/// This struct provides thread-safe access to a single Parakeet model instance
/// that can be shared between components (TranscriptionManager and WakeWordDetector).
/// Previously, each component loaded its own ~3GB model, wasting memory.
///
/// Usage:
/// ```ignore
/// let shared_model = SharedTranscriptionModel::new();
/// shared_model.load(model_path)?;
///
/// // Both can share the same model:
/// let manager = TranscriptionManager::with_shared_model(shared_model.clone());
/// let detector = WakeWordDetector::with_shared_model(shared_model.clone());
/// ```
#[derive(Clone)]
pub struct SharedTranscriptionModel {
    /// The Parakeet TDT model wrapped in thread-safe primitives
    model: Arc<Mutex<Option<ParakeetTDT>>>,
    /// Current transcription state
    state: Arc<Mutex<TranscriptionState>>,
}

impl Default for SharedTranscriptionModel {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedTranscriptionModel {
    /// Create a new SharedTranscriptionModel without a loaded model
    pub fn new() -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(TranscriptionState::Unloaded)),
        }
    }

    /// Load the Parakeet TDT model from the given directory path
    ///
    /// This should be called once at application startup.
    pub fn load(&self, model_dir: &Path) -> TranscriptionResult<()> {
        let path_str = model_dir.to_str().ok_or_else(|| {
            TranscriptionError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        info!("Loading shared Parakeet TDT model from {}...", path_str);

        let tdt = ParakeetTDT::from_pretrained(path_str, None)
            .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;

        {
            let mut guard = self
                .model
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *guard = Some(tdt);
        }

        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *state = TranscriptionState::Idle;
        }

        info!("Shared Parakeet TDT model loaded successfully");
        Ok(())
    }

    /// Check if the model is loaded
    pub fn is_loaded(&self) -> bool {
        self.model
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// Get the current transcription state
    pub fn state(&self) -> TranscriptionState {
        self.state
            .lock()
            .map(|guard| *guard)
            .unwrap_or(TranscriptionState::Unloaded)
    }

    /// Reset state from Completed/Error back to Idle
    pub fn reset_to_idle(&self) -> TranscriptionResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| TranscriptionError::LockPoisoned)?;

        if *state == TranscriptionState::Completed || *state == TranscriptionState::Error {
            *state = TranscriptionState::Idle;
        }
        Ok(())
    }

    /// Transcribe audio from a WAV file to text
    ///
    /// This is the primary method for batch transcription (hotkey recording).
    ///
    /// Uses RAII guard to ensure state transitions are atomic and panic-safe:
    /// - State becomes Transcribing when guard is acquired
    /// - State becomes Completed/Error when guard completes
    /// - State resets to Idle on panic
    pub fn transcribe_file(&self, file_path: &str) -> TranscriptionResult<String> {
        if file_path.is_empty() {
            return Err(TranscriptionError::InvalidAudio(
                "Empty file path".to_string(),
            ));
        }

        // Acquire guard - sets state to Transcribing
        let mut state_guard = TranscribingGuard::new(self.state.clone())?;

        // Do the actual transcription work
        let result = {
            let mut model_guard = self
                .model
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;

            let tdt = model_guard.as_mut().ok_or(TranscriptionError::ModelNotLoaded)?;

            match tdt.transcribe_file(file_path, None) {
                Ok(transcribe_result) => {
                    let fixed_text = fix_parakeet_text(&transcribe_result.tokens);

                    info!("=== SharedTranscriptionModel transcribe_file result ===");
                    info!("result.text (broken): {:?}", transcribe_result.text);
                    info!("fixed_text: {:?}", fixed_text);
                    info!("=== end result ===");

                    Ok(fixed_text)
                }
                Err(e) => Err(TranscriptionError::TranscriptionFailed(e.to_string())),
            }
        };

        // Set completion state explicitly
        match &result {
            Ok(_) => state_guard.complete_success(),
            Err(_) => state_guard.complete_with_error(),
        }

        result
    }

    /// Transcribe audio samples directly (in-memory)
    ///
    /// This is the primary method for streaming transcription (wake word detection).
    ///
    /// # Arguments
    /// * `samples` - Audio samples as f32 values
    /// * `sample_rate` - Sample rate in Hz (typically 16000)
    /// * `channels` - Number of audio channels (typically 1 for mono)
    pub fn transcribe_samples(
        &self,
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u16,
    ) -> TranscriptionResult<String> {
        if samples.is_empty() {
            return Err(TranscriptionError::InvalidAudio(
                "Empty audio samples".to_string(),
            ));
        }

        // Note: We don't set state to Transcribing for streaming use cases
        // to avoid state conflicts with batch transcription. The state machine
        // is primarily for the batch transcription flow.

        let mut guard = self
            .model
            .lock()
            .map_err(|_| TranscriptionError::LockPoisoned)?;

        let tdt = guard.as_mut().ok_or(TranscriptionError::ModelNotLoaded)?;

        match tdt.transcribe_samples(samples, sample_rate, channels, None) {
            Ok(transcribe_result) => {
                let fixed_text = fix_parakeet_text(&transcribe_result.tokens);
                Ok(fixed_text)
            }
            Err(e) => Err(TranscriptionError::TranscriptionFailed(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_model_new_is_unloaded() {
        let model = SharedTranscriptionModel::new();
        assert!(!model.is_loaded());
        assert_eq!(model.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_shared_model_default_is_unloaded() {
        let model = SharedTranscriptionModel::default();
        assert!(!model.is_loaded());
        assert_eq!(model.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_transcribe_file_returns_error_when_model_not_loaded() {
        let model = SharedTranscriptionModel::new();
        let result = model.transcribe_file("/nonexistent/audio.wav");
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    }

    #[test]
    fn test_transcribe_file_returns_error_for_empty_path() {
        let model = SharedTranscriptionModel::new();
        let result = model.transcribe_file("");
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::InvalidAudio(_))));
    }

    #[test]
    fn test_transcribe_samples_returns_error_when_model_not_loaded() {
        let model = SharedTranscriptionModel::new();
        let result = model.transcribe_samples(vec![0.1, 0.2, 0.3], 16000, 1);
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    }

    #[test]
    fn test_transcribe_samples_returns_error_for_empty_samples() {
        let model = SharedTranscriptionModel::new();
        let result = model.transcribe_samples(vec![], 16000, 1);
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::InvalidAudio(_))));
    }

    #[test]
    fn test_load_fails_with_invalid_path() {
        let model = SharedTranscriptionModel::new();
        let result = model.load(Path::new("/nonexistent/path/to/model"));
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
    }

    #[test]
    fn test_reset_to_idle_from_completed() {
        let model = SharedTranscriptionModel::new();
        // Manually set state to Completed for testing
        {
            let mut state = model.state.lock().unwrap();
            *state = TranscriptionState::Completed;
        }
        assert_eq!(model.state(), TranscriptionState::Completed);

        model.reset_to_idle().unwrap();
        assert_eq!(model.state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_reset_to_idle_from_error() {
        let model = SharedTranscriptionModel::new();
        // Manually set state to Error for testing
        {
            let mut state = model.state.lock().unwrap();
            *state = TranscriptionState::Error;
        }
        assert_eq!(model.state(), TranscriptionState::Error);

        model.reset_to_idle().unwrap();
        assert_eq!(model.state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_reset_to_idle_noop_from_unloaded() {
        let model = SharedTranscriptionModel::new();
        assert_eq!(model.state(), TranscriptionState::Unloaded);

        model.reset_to_idle().unwrap();
        assert_eq!(model.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_shared_model_is_clone() {
        let model = SharedTranscriptionModel::new();
        let cloned = model.clone();

        // Both should share the same underlying model
        assert!(!model.is_loaded());
        assert!(!cloned.is_loaded());

        // State changes should be visible to both
        {
            let mut state = model.state.lock().unwrap();
            *state = TranscriptionState::Idle;
        }
        assert_eq!(model.state(), TranscriptionState::Idle);
        assert_eq!(cloned.state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_concurrent_access_does_not_panic() {
        use std::thread;

        let model = SharedTranscriptionModel::new();
        let model_clone1 = model.clone();
        let model_clone2 = model.clone();

        // Spawn threads that access the model concurrently
        let handle1 = thread::spawn(move || {
            for _ in 0..100 {
                let _ = model_clone1.is_loaded();
                let _ = model_clone1.state();
            }
        });

        let handle2 = thread::spawn(move || {
            for _ in 0..100 {
                let _ = model_clone2.is_loaded();
                let _ = model_clone2.state();
            }
        });

        // Main thread also accesses
        for _ in 0..100 {
            let _ = model.is_loaded();
            let _ = model.state();
        }

        handle1.join().unwrap();
        handle2.join().unwrap();
    }

    // ==========================================================================
    // TranscribingGuard Tests
    // ==========================================================================

    #[test]
    fn test_guard_sets_state_to_transcribing_on_creation() {
        let state = Arc::new(Mutex::new(TranscriptionState::Idle));
        let guard = TranscribingGuard::new(state.clone()).unwrap();
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Transcribing);
        drop(guard);
    }

    #[test]
    fn test_guard_resets_state_to_idle_on_drop() {
        let state = Arc::new(Mutex::new(TranscriptionState::Idle));
        {
            let _guard = TranscribingGuard::new(state.clone()).unwrap();
            assert_eq!(*state.lock().unwrap(), TranscriptionState::Transcribing);
        }
        // Guard dropped, state should be Idle
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Idle);
    }

    #[test]
    fn test_guard_resets_state_to_idle_on_panic() {
        use std::panic;

        let state = Arc::new(Mutex::new(TranscriptionState::Idle));
        let state_clone = state.clone();

        let result = panic::catch_unwind(move || {
            let _guard = TranscribingGuard::new(state_clone).unwrap();
            panic!("Simulated panic during transcription");
        });

        assert!(result.is_err()); // Panic occurred
        // Guard should have reset state to Idle despite panic
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Idle);
    }

    #[test]
    fn test_guard_complete_success_sets_completed_state() {
        let state = Arc::new(Mutex::new(TranscriptionState::Idle));
        {
            let mut guard = TranscribingGuard::new(state.clone()).unwrap();
            assert_eq!(*state.lock().unwrap(), TranscriptionState::Transcribing);
            guard.complete_success();
            assert_eq!(*state.lock().unwrap(), TranscriptionState::Completed);
        }
        // State remains Completed after drop (not reset to Idle)
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Completed);
    }

    #[test]
    fn test_guard_complete_with_error_sets_error_state() {
        let state = Arc::new(Mutex::new(TranscriptionState::Idle));
        {
            let mut guard = TranscribingGuard::new(state.clone()).unwrap();
            assert_eq!(*state.lock().unwrap(), TranscriptionState::Transcribing);
            guard.complete_with_error();
            assert_eq!(*state.lock().unwrap(), TranscriptionState::Error);
        }
        // State remains Error after drop (not reset to Idle)
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Error);
    }

    #[test]
    fn test_guard_fails_when_model_not_loaded() {
        let state = Arc::new(Mutex::new(TranscriptionState::Unloaded));
        let result = TranscribingGuard::new(state.clone());
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
        // State unchanged
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_concurrent_guards_are_consistent() {
        use std::thread;

        let state = Arc::new(Mutex::new(TranscriptionState::Idle));
        let mut handles = vec![];

        // Spawn threads that create and drop guards rapidly
        for _ in 0..10 {
            let state_clone = state.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    // Try to acquire guard - may fail if state is wrong
                    if let Ok(guard) = TranscribingGuard::new(state_clone.clone()) {
                        // Do some "work"
                        std::hint::black_box(1 + 1);
                        drop(guard);
                    }
                    // Small yield to increase contention
                    std::thread::yield_now();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // After all threads complete, state should be Idle (or possibly Transcribing if one is in progress)
        let final_state = *state.lock().unwrap();
        assert!(
            final_state == TranscriptionState::Idle || final_state == TranscriptionState::Transcribing,
            "Final state should be Idle or Transcribing, got {:?}",
            final_state
        );
    }
}

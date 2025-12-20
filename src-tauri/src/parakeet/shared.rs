// SharedTranscriptionModel for thread-safe Parakeet model sharing
// Eliminates duplicate model instances (~3GB memory savings)

use parakeet_rs::ParakeetTDT;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use super::types::{TranscriptionError, TranscriptionResult, TranscriptionService, TranscriptionState};
use super::utils::fix_parakeet_text;
use crate::{debug, info};

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
/// that can be shared between all transcription consumers and WakeWordDetector.
/// Previously, each component loaded its own ~3GB model, wasting memory.
///
/// ## Mutual Exclusion
///
/// The `transcription_lock` ensures that batch transcription (`transcribe_file`)
/// and streaming transcription (`transcribe_samples`) cannot run concurrently.
/// This prevents latency spikes and unpredictable behavior when both modes
/// try to use the model simultaneously.
///
/// Usage:
/// ```ignore
/// let shared_model = SharedTranscriptionModel::new();
/// shared_model.load(model_path)?;
///
/// // Both can share the same model:
/// let detector = WakeWordDetector::with_shared_model(shared_model.clone());
/// ```
#[derive(Clone)]
pub struct SharedTranscriptionModel {
    /// The Parakeet TDT model wrapped in thread-safe primitives
    model: Arc<Mutex<Option<ParakeetTDT>>>,
    /// Current transcription state
    state: Arc<Mutex<TranscriptionState>>,
    /// Transcription lock: ensures mutual exclusion between batch and streaming
    /// transcription. Only one transcription operation can proceed at a time.
    /// This prevents race conditions between `transcribe_file()` and `transcribe_samples()`.
    transcription_lock: Arc<Mutex<()>>,
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
            transcription_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Acquire exclusive access for transcription operations.
    ///
    /// This must be called before any transcription to ensure mutual exclusion
    /// between batch (`transcribe_file`) and streaming (`transcribe_samples`) modes.
    /// The returned guard holds the lock until dropped.
    fn acquire_transcription_lock(&self) -> TranscriptionResult<MutexGuard<'_, ()>> {
        self.transcription_lock
            .lock()
            .map_err(|_| TranscriptionError::LockPoisoned)
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
    #[allow(dead_code)] // Will be used for UI state display
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
    ///
    /// ## Mutual Exclusion
    ///
    /// Acquires `transcription_lock` before transcription to prevent concurrent
    /// execution with `transcribe_samples()`. The lock is held for the duration
    /// of the transcription and released when the guard is dropped.
    pub fn transcribe_file(&self, file_path: &str) -> TranscriptionResult<String> {
        if file_path.is_empty() {
            return Err(TranscriptionError::InvalidAudio(
                "Empty file path".to_string(),
            ));
        }

        // Acquire exclusive transcription access - blocks if streaming is active
        let _transcription_permit = self.acquire_transcription_lock()?;

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

                    debug!("Transcription result: {:?}", fixed_text);

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
    ///
    /// ## Mutual Exclusion
    ///
    /// Acquires `transcription_lock` before transcription to prevent concurrent
    /// execution with `transcribe_file()`. The lock is held for the duration
    /// of the transcription and released when the method returns.
    ///
    /// Note: We don't set state to Transcribing for streaming use cases
    /// to avoid state conflicts with batch transcription. The state machine
    /// is primarily for the batch transcription flow.
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

        // Acquire exclusive transcription access - blocks if batch transcription is active
        let _transcription_permit = self.acquire_transcription_lock()?;

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

impl TranscriptionService for SharedTranscriptionModel {
    fn load_model(&self, path: &Path) -> TranscriptionResult<()> {
        self.load(path)
    }

    fn transcribe(&self, file_path: &str) -> TranscriptionResult<String> {
        self.transcribe_file(file_path)
    }

    fn is_loaded(&self) -> bool {
        self.is_loaded()
    }

    fn state(&self) -> TranscriptionState {
        self.state()
    }

    fn reset_to_idle(&self) -> TranscriptionResult<()> {
        self.reset_to_idle()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests removed per docs/TESTING.md:
    // - test_shared_model_new_is_unloaded: Obvious default
    // - test_shared_model_default_is_unloaded: Obvious default (duplicate)
    // - test_shared_model_is_clone: Type system guarantee (Arc semantics)
    // - test_concurrent_access_does_not_panic: Rust type system guarantees Send+Sync

    // ==================== Behavior Tests ====================
    // These test actual user-visible behavior and error handling

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

    // ==================== State Machine Tests ====================
    // These test the complete state transition workflow

    #[test]
    fn test_reset_to_idle_state_transitions() {
        let model = SharedTranscriptionModel::new();

        // Test reset from Completed
        {
            let mut state = model.state.lock().unwrap();
            *state = TranscriptionState::Completed;
        }
        model.reset_to_idle().unwrap();
        assert_eq!(model.state(), TranscriptionState::Idle);

        // Test reset from Error
        {
            let mut state = model.state.lock().unwrap();
            *state = TranscriptionState::Error;
        }
        model.reset_to_idle().unwrap();
        assert_eq!(model.state(), TranscriptionState::Idle);

        // Test noop from Unloaded (doesn't transition)
        {
            let mut state = model.state.lock().unwrap();
            *state = TranscriptionState::Unloaded;
        }
        model.reset_to_idle().unwrap();
        assert_eq!(model.state(), TranscriptionState::Unloaded);
    }

    // ==================== TranscribingGuard Tests ====================
    // RAII guard is critical for panic safety - these are behavior tests

    #[test]
    fn test_guard_state_lifecycle() {
        let state = Arc::new(Mutex::new(TranscriptionState::Idle));

        // Test normal lifecycle: Idle -> Transcribing -> Idle
        {
            let _guard = TranscribingGuard::new(state.clone()).unwrap();
            assert_eq!(*state.lock().unwrap(), TranscriptionState::Transcribing);
        }
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Idle);

        // Test complete_success: stays Completed after drop
        {
            let mut guard = TranscribingGuard::new(state.clone()).unwrap();
            guard.complete_success();
            assert_eq!(*state.lock().unwrap(), TranscriptionState::Completed);
        }
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Completed);

        // Reset for next test
        *state.lock().unwrap() = TranscriptionState::Idle;

        // Test complete_with_error: stays Error after drop
        {
            let mut guard = TranscribingGuard::new(state.clone()).unwrap();
            guard.complete_with_error();
            assert_eq!(*state.lock().unwrap(), TranscriptionState::Error);
        }
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Error);
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

        assert!(result.is_err());
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Idle);
    }

    #[test]
    fn test_guard_fails_when_model_not_loaded() {
        let state = Arc::new(Mutex::new(TranscriptionState::Unloaded));
        let result = TranscribingGuard::new(state.clone());
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
        assert_eq!(*state.lock().unwrap(), TranscriptionState::Unloaded);
    }

    // ==================== Transcription Lock Tests ====================
    // Mutual exclusion is critical behavior - keep concurrency tests

    #[test]
    fn test_transcription_lock_blocks_concurrent_access() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;
        use std::time::Duration;

        let model = SharedTranscriptionModel::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        for _ in 0..10 {
            let model_clone = model.clone();
            let counter_clone = counter.clone();
            let max_concurrent_clone = max_concurrent.clone();

            handles.push(thread::spawn(move || {
                for _ in 0..5 {
                    let _guard = model_clone.acquire_transcription_lock().unwrap();
                    let current = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
                    max_concurrent_clone.fetch_max(current, Ordering::SeqCst);
                    thread::sleep(Duration::from_micros(10));
                    counter_clone.fetch_sub(1, Ordering::SeqCst);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(max_concurrent.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_transcription_lock_released_on_error_paths() {
        let model = SharedTranscriptionModel::new();

        // Error should release lock
        let _ = model.transcribe_samples(vec![], 16000, 1);

        // Lock should be acquirable again
        let guard = model.acquire_transcription_lock();
        assert!(guard.is_ok());
    }
}

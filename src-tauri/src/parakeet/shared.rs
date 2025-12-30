// SharedTranscriptionModel for thread-safe Parakeet model sharing
// Eliminates duplicate model instances (~3GB memory savings)
//
// Uses parking_lot::Mutex instead of std::sync::Mutex because:
// - parking_lot::Mutex does NOT poison on panic - it simply releases the lock
// - This allows subsequent transcriptions to proceed normally after a panic
// - std::sync::Mutex would poison the lock, making the model permanently unavailable

use hound::WavReader;
use parking_lot::{Mutex, MutexGuard};
use parakeet_rs::ParakeetTDT;
use std::path::Path;
use std::sync::Arc;

use super::types::{TranscriptionError, TranscriptionResult, TranscriptionService, TranscriptionState};
use super::utils::fix_parakeet_text;

// ============================================================================
// WAV Validation - Prevent panics in parakeet-rs
// ============================================================================

/// Validates a WAV file before transcription to prevent known panic triggers in parakeet-rs.
///
/// This function checks:
/// - File exists
/// - File is not empty (size > 0)
/// - Valid WAV header (parseable by hound)
/// - WAV contains at least one audio sample
///
/// # Arguments
/// * `file_path` - Path to the WAV file to validate
///
/// # Returns
/// * `Ok(())` if the WAV file is valid for transcription
/// * `Err(TranscriptionError::InvalidAudio)` with descriptive message if invalid
///
/// # Why This Is Needed
/// parakeet-rs panics with 'index out of bounds: the len is 0' when given empty
/// or invalid audio files (see audio.rs:29 in parakeet-rs). This validation
/// catches such files before they reach the model.
fn validate_wav_for_transcription(file_path: &str) -> TranscriptionResult<()> {
    let path = Path::new(file_path);

    // Check file exists
    if !path.exists() {
        return Err(TranscriptionError::InvalidAudio(format!(
            "File not found: {}",
            file_path
        )));
    }

    // Check file is not empty
    let metadata = std::fs::metadata(path).map_err(|e| {
        TranscriptionError::InvalidAudio(format!("Cannot read file metadata: {}", e))
    })?;

    if metadata.len() == 0 {
        return Err(TranscriptionError::InvalidAudio(
            "File is empty".to_string(),
        ));
    }

    // Validate WAV header using hound
    let reader = WavReader::open(path).map_err(|e| {
        TranscriptionError::InvalidAudio(format!("Invalid WAV file: {}", e))
    })?;

    // Check that WAV has samples (prevents 'index out of bounds' panic in parakeet-rs)
    if reader.len() == 0 {
        return Err(TranscriptionError::InvalidAudio(
            "WAV file contains no audio samples".to_string(),
        ));
    }

    Ok(())
}

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
    /// - `ModelNotLoaded` if the model is in Unloaded state
    ///
    /// Note: Uses parking_lot::Mutex which doesn't poison on panic,
    /// so LockPoisoned errors are no longer possible.
    pub fn new(state: Arc<Mutex<TranscriptionState>>) -> TranscriptionResult<Self> {
        let mut guard = state.lock();
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
        let mut guard = self.state.lock();
        *guard = TranscriptionState::Completed;
        self.completed = true;
    }

    /// Mark the transcription as failed with an error.
    ///
    /// Sets state to `Error` and marks the guard as completed
    /// so it won't reset to `Idle` on drop.
    pub fn complete_with_error(&mut self) {
        let mut guard = self.state.lock();
        *guard = TranscriptionState::Error;
        self.completed = true;
    }
}

impl Drop for TranscribingGuard {
    fn drop(&mut self) {
        // Only reset to Idle if we didn't explicitly complete
        // This handles both normal completion (where we want Idle)
        // and panics (where we want to reset from Transcribing)
        //
        // Note: parking_lot::Mutex doesn't poison on panic, so the lock
        // is always available after a panic, allowing recovery.
        if !self.completed {
            let mut guard = self.state.lock();
            // Only reset if still in Transcribing state
            // (in case someone else changed it)
            if *guard == TranscriptionState::Transcribing {
                *guard = TranscriptionState::Idle;
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
    ///
    /// Note: Uses parking_lot::Mutex which doesn't poison, so this always succeeds.
    fn acquire_transcription_lock(&self) -> MutexGuard<'_, ()> {
        self.transcription_lock.lock()
    }

    /// Load the Parakeet TDT model from the given directory path
    ///
    /// This should be called once at application startup.
    pub fn load(&self, model_dir: &Path) -> TranscriptionResult<()> {
        let path_str = model_dir.to_str().ok_or_else(|| {
            TranscriptionError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        crate::info!("Loading shared Parakeet TDT model from {}...", path_str);

        let tdt = ParakeetTDT::from_pretrained(path_str, None)
            .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;

        {
            let mut guard = self.model.lock();
            *guard = Some(tdt);
        }

        {
            let mut state = self.state.lock();
            *state = TranscriptionState::Idle;
        }

        crate::info!("Shared Parakeet TDT model loaded successfully");
        Ok(())
    }

    /// Check if the model is loaded
    pub fn is_loaded(&self) -> bool {
        self.model.lock().is_some()
    }

    /// Get the current transcription state
    #[allow(dead_code)] // Will be used for UI state display
    pub fn state(&self) -> TranscriptionState {
        *self.state.lock()
    }

    /// Reset state from Completed/Error back to Idle
    pub fn reset_to_idle(&self) -> TranscriptionResult<()> {
        let mut state = self.state.lock();

        if *state == TranscriptionState::Completed || *state == TranscriptionState::Error {
            *state = TranscriptionState::Idle;
        }
        Ok(())
    }

    /// Unload the model from memory and set state to Unloaded.
    ///
    /// This releases the ~3GB model memory. After unloading, `is_loaded()` will
    /// return false and transcription operations will fail with `ModelNotLoaded`.
    ///
    /// Thread-safe: acquires transcription lock to ensure no transcription is in progress.
    ///
    /// Used by: create-wake-handler-module-for-sleep-wake-events (spec #3)
    #[allow(dead_code)]
    pub fn unload(&self) -> TranscriptionResult<()> {
        // Acquire exclusive transcription access - blocks if transcription is active
        let _transcription_permit = self.acquire_transcription_lock()?;

        // Set model to None
        {
            let mut model_guard = self
                .model
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *model_guard = None;
        }

        // Set state to Unloaded
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *state = TranscriptionState::Unloaded;
        }

        crate::info!("Shared Parakeet TDT model unloaded");
        Ok(())
    }

    /// Reload the model from the given directory path.
    ///
    /// This unloads the current model (if any), then loads from the new path.
    /// Useful for reloading after system wake events when the model may be corrupted.
    ///
    /// Thread-safe: acquires transcription lock to ensure no transcription is in progress.
    /// State transitions: current state -> Unloaded -> Idle (on success)
    ///
    /// Used by: create-wake-handler-module-for-sleep-wake-events (spec #3)
    #[allow(dead_code)]
    pub fn reload(&self, model_dir: &Path) -> TranscriptionResult<()> {
        // Acquire exclusive transcription access - blocks if transcription is active
        let _transcription_permit = self.acquire_transcription_lock()?;

        // First unload (without acquiring lock again - we already have it)
        {
            let mut model_guard = self
                .model
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *model_guard = None;
        }
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *state = TranscriptionState::Unloaded;
        }
        crate::info!("Model unloaded for reload");

        // Now load the new model
        let path_str = model_dir.to_str().ok_or_else(|| {
            TranscriptionError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        crate::info!("Reloading shared Parakeet TDT model from {}...", path_str);

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

        crate::info!("Shared Parakeet TDT model reloaded successfully");
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
    ///
    /// ## Panic Resilience
    ///
    /// Uses parking_lot::Mutex which doesn't poison on panic. If parakeet-rs
    /// panics during transcription, the lock is released and subsequent
    /// transcriptions can proceed normally.
    pub fn transcribe_file(&self, file_path: &str) -> TranscriptionResult<String> {
        if file_path.is_empty() {
            return Err(TranscriptionError::InvalidAudio(
                "Empty file path".to_string(),
            ));
        }

        // Validate WAV file BEFORE acquiring locks to prevent parakeet-rs panics.
        // This catches empty/invalid files that would cause 'index out of bounds' errors.
        validate_wav_for_transcription(file_path)?;

        // Acquire exclusive transcription access - blocks if streaming is active
        let _transcription_permit = self.acquire_transcription_lock();

        // Acquire guard - sets state to Transcribing
        let mut state_guard = TranscribingGuard::new(self.state.clone())?;

        // Do the actual transcription work
        let result = {
            let mut model_guard = self.model.lock();

            let tdt = model_guard.as_mut().ok_or(TranscriptionError::ModelNotLoaded)?;

            match tdt.transcribe_file(file_path, None) {
                Ok(transcribe_result) => {
                    let fixed_text = fix_parakeet_text(&transcribe_result.tokens);

                    crate::debug!("Transcription result: {:?}", fixed_text);

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
    /// ## Panic Resilience
    ///
    /// Uses parking_lot::Mutex which doesn't poison on panic. If parakeet-rs
    /// panics during transcription, the lock is released and subsequent
    /// transcriptions can proceed normally.
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
        let _transcription_permit = self.acquire_transcription_lock();

        let mut guard = self.model.lock();

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
#[path = "shared_test.rs"]
mod tests;

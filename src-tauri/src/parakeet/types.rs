// Shared types for transcription services
// These types are used by all transcription backends (Parakeet, etc.)

use std::path::Path;

/// Transcription state machine states
/// State flow: Unloaded -> (load model) -> Idle -> Transcribing -> Completed/Error -> Idle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptionState {
    /// No model loaded, cannot transcribe
    Unloaded,
    /// Model loaded, ready to transcribe
    Idle,
    /// Currently processing audio
    Transcribing,
    /// Transcription completed successfully
    Completed,
    /// Transcription failed with error
    Error,
}

/// Errors that can occur during transcription operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TranscriptionError {
    /// Model has not been loaded yet
    #[error("Model not loaded")]
    ModelNotLoaded,
    /// Failed to load the model
    #[error("Failed to load model: {0}")]
    ModelLoadFailed(String),
    /// Failed during transcription
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),
    /// Audio data is invalid or empty
    #[error("Invalid audio: {0}")]
    InvalidAudio(String),
    // NOTE: LockPoisoned variant removed - parking_lot::Mutex doesn't poison on panic,
    // so this error case is no longer possible.
}

/// Result type for transcription operations
pub type TranscriptionResult<T> = Result<T, TranscriptionError>;

/// Trait for transcription services, enabling mockability in tests
#[allow(dead_code)]
pub trait TranscriptionService: Send + Sync {
    /// Load a model from the given path
    #[must_use = "this returns a Result that should be handled"]
    fn load_model(&self, path: &Path) -> TranscriptionResult<()>;

    /// Transcribe audio from a WAV file to text
    #[must_use = "this returns a Result that should be handled"]
    fn transcribe(&self, file_path: &str) -> TranscriptionResult<String>;

    /// Check if a model is loaded
    fn is_loaded(&self) -> bool;

    /// Get the current transcription state
    fn state(&self) -> TranscriptionState;

    /// Reset state from Completed/Error back to Idle
    /// This should be called after handling the transcription result
    #[must_use = "this returns a Result that should be handled"]
    fn reset_to_idle(&self) -> TranscriptionResult<()>;
}

// Tests removed per docs/TESTING.md:
// - Display/Debug trait tests: "if it compiles, it works"
// - Clone tests: Type system guarantee
// - Equality tests: Type system guarantee (#[derive(PartialEq)])
// - State existence tests: Type system guarantees enum variants exist

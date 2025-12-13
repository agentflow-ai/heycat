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
#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptionError {
    /// Model has not been loaded yet
    ModelNotLoaded,
    /// Failed to load the model
    ModelLoadFailed(String),
    /// Failed during transcription
    TranscriptionFailed(String),
    /// Audio data is invalid or empty
    InvalidAudio(String),
    /// Failed to acquire lock on context
    LockPoisoned,
}

impl std::fmt::Display for TranscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionError::ModelNotLoaded => write!(f, "Model not loaded"),
            TranscriptionError::ModelLoadFailed(msg) => write!(f, "Failed to load model: {}", msg),
            TranscriptionError::TranscriptionFailed(msg) => {
                write!(f, "Transcription failed: {}", msg)
            }
            TranscriptionError::InvalidAudio(msg) => write!(f, "Invalid audio: {}", msg),
            TranscriptionError::LockPoisoned => write!(f, "Internal lock error"),
        }
    }
}

impl std::error::Error for TranscriptionError {}

/// Result type for transcription operations
pub type TranscriptionResult<T> = Result<T, TranscriptionError>;

/// Trait for transcription services, enabling mockability in tests
pub trait TranscriptionService: Send + Sync {
    /// Load a model from the given path
    #[must_use = "this returns a Result that should be handled"]
    fn load_model(&self, path: &Path) -> TranscriptionResult<()>;

    /// Transcribe audio samples to text
    /// Audio must be 16kHz mono f32 samples
    #[must_use = "this returns a Result that should be handled"]
    fn transcribe(&self, samples: &[f32]) -> TranscriptionResult<String>;

    /// Check if a model is loaded
    fn is_loaded(&self) -> bool;

    /// Get the current transcription state
    fn state(&self) -> TranscriptionState;

    /// Reset state from Completed/Error back to Idle
    /// This should be called after handling the transcription result
    #[must_use = "this returns a Result that should be handled"]
    fn reset_to_idle(&self) -> TranscriptionResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_error_display() {
        assert_eq!(
            format!("{}", TranscriptionError::ModelNotLoaded),
            "Model not loaded"
        );
        assert!(format!("{}", TranscriptionError::ModelLoadFailed("test".to_string()))
            .contains("test"));
        assert!(format!(
            "{}",
            TranscriptionError::TranscriptionFailed("test".to_string())
        )
        .contains("test"));
        assert!(
            format!("{}", TranscriptionError::InvalidAudio("test".to_string())).contains("test")
        );
        assert_eq!(
            format!("{}", TranscriptionError::LockPoisoned),
            "Internal lock error"
        );
    }

    #[test]
    fn test_transcription_error_debug() {
        let error = TranscriptionError::ModelNotLoaded;
        let debug = format!("{:?}", error);
        assert!(debug.contains("ModelNotLoaded"));
    }

    #[test]
    fn test_transcription_state_equality() {
        assert_eq!(TranscriptionState::Idle, TranscriptionState::Idle);
        assert_ne!(TranscriptionState::Idle, TranscriptionState::Transcribing);
    }

    #[test]
    fn test_transcription_state_debug() {
        let state = TranscriptionState::Idle;
        let debug = format!("{:?}", state);
        assert!(debug.contains("Idle"));
    }

    #[test]
    fn test_transcription_state_clone() {
        let state = TranscriptionState::Transcribing;
        let cloned = state;
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_transcription_error_clone() {
        let error = TranscriptionError::ModelNotLoaded;
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_transcription_state_completed_and_error_exist() {
        // Verify Completed and Error states exist and are distinct
        assert_ne!(TranscriptionState::Completed, TranscriptionState::Error);
        assert_ne!(TranscriptionState::Completed, TranscriptionState::Idle);
        assert_ne!(TranscriptionState::Error, TranscriptionState::Idle);
    }

    #[test]
    fn test_transcription_state_completed_debug() {
        let state = TranscriptionState::Completed;
        let debug = format!("{:?}", state);
        assert!(debug.contains("Completed"));
    }

    #[test]
    fn test_transcription_state_error_debug() {
        let state = TranscriptionState::Error;
        let debug = format!("{:?}", state);
        assert!(debug.contains("Error"));
    }
}

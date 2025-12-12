// WhisperContext wrapper for thread-safe transcription
// Provides model loading and audio-to-text conversion

use std::path::Path;
use std::sync::{Arc, Mutex};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Transcription state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptionState {
    /// No model loaded, cannot transcribe
    Unloaded,
    /// Model loaded, ready to transcribe
    Idle,
    /// Currently processing audio
    Transcribing,
}

/// Errors that can occur during transcription operations
#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptionError {
    /// Model has not been loaded yet
    ModelNotLoaded,
    /// Failed to load the whisper model
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
    /// Load a whisper model from the given path
    fn load_model(&self, path: &Path) -> TranscriptionResult<()>;

    /// Transcribe audio samples to text
    /// Audio must be 16kHz mono f32 samples
    fn transcribe(&self, samples: &[f32]) -> TranscriptionResult<String>;

    /// Check if a model is loaded
    fn is_loaded(&self) -> bool;

    /// Get the current transcription state
    fn state(&self) -> TranscriptionState;
}

/// Thread-safe wrapper around WhisperContext
/// Uses Mutex to serialize access since whisper.cpp is not thread-safe
pub struct WhisperManager {
    context: Arc<Mutex<Option<WhisperContext>>>,
    state: Arc<Mutex<TranscriptionState>>,
}

impl Default for WhisperManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WhisperManager {
    /// Create a new WhisperManager without a loaded model
    pub fn new() -> Self {
        Self {
            context: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(TranscriptionState::Unloaded)),
        }
    }
}

impl TranscriptionService for WhisperManager {
    fn load_model(&self, path: &Path) -> TranscriptionResult<()> {
        let ctx = WhisperContext::new_with_params(
            path.to_str()
                .ok_or_else(|| TranscriptionError::ModelLoadFailed("Invalid path".to_string()))?,
            WhisperContextParameters::default(),
        )
        .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;

        // Store context and update state
        {
            let mut guard = self
                .context
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *guard = Some(ctx);
        }
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *state = TranscriptionState::Idle;
        }

        Ok(())
    }

    fn transcribe(&self, samples: &[f32]) -> TranscriptionResult<String> {
        // Validate audio input
        if samples.is_empty() {
            return Err(TranscriptionError::InvalidAudio(
                "Empty audio buffer".to_string(),
            ));
        }

        // Set state to transcribing
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            if *state == TranscriptionState::Unloaded {
                return Err(TranscriptionError::ModelNotLoaded);
            }
            *state = TranscriptionState::Transcribing;
        }

        // Perform transcription (mutex ensures single access)
        let result = {
            let guard = self
                .context
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;

            let ctx = guard.as_ref().ok_or(TranscriptionError::ModelNotLoaded)?;

            // Create whisper state for this transcription
            let mut whisper_state = ctx
                .create_state()
                .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))?;

            // Configure parameters for transcription
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);

            // Run transcription
            whisper_state
                .full(params, samples)
                .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))?;

            // Collect all segments into output text
            let num_segments = whisper_state.full_n_segments().map_err(|e| {
                TranscriptionError::TranscriptionFailed(format!("Failed to get segments: {}", e))
            })?;

            let mut text = String::new();
            for i in 0..num_segments {
                if let Ok(segment) = whisper_state.full_get_segment_text(i) {
                    text.push_str(&segment);
                }
            }

            Ok(text.trim().to_string())
        };

        // Reset state back to idle (or error state)
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *state = if result.is_ok() {
                TranscriptionState::Idle
            } else {
                TranscriptionState::Idle // Reset to idle even on error
            };
        }

        result
    }

    fn is_loaded(&self) -> bool {
        self.context
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    fn state(&self) -> TranscriptionState {
        self.state
            .lock()
            .map(|guard| *guard)
            .unwrap_or(TranscriptionState::Unloaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whisper_manager_new_is_unloaded() {
        let manager = WhisperManager::new();
        assert!(!manager.is_loaded());
        assert_eq!(manager.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_whisper_manager_default_is_unloaded() {
        let manager = WhisperManager::default();
        assert!(!manager.is_loaded());
    }

    #[test]
    fn test_transcribe_returns_error_when_model_not_loaded() {
        let manager = WhisperManager::new();
        let samples = vec![0.0f32; 16000]; // 1 second of silence
        let result = manager.transcribe(&samples);
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    }

    #[test]
    fn test_transcribe_returns_error_for_empty_audio() {
        let manager = WhisperManager::new();
        let samples: Vec<f32> = vec![];
        let result = manager.transcribe(&samples);
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::InvalidAudio(_))));
    }

    #[test]
    fn test_load_model_fails_with_invalid_path() {
        let manager = WhisperManager::new();
        let result = manager.load_model(Path::new("/nonexistent/path/to/model.bin"));
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
    }

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
}

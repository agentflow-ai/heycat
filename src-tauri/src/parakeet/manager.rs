// TranscriptionManager for Parakeet-based transcription
// Provides TDT (batch) and EOU (streaming) transcription using NVIDIA Parakeet models

use super::types::{TranscriptionError, TranscriptionResult, TranscriptionService, TranscriptionState};
use parakeet_rs::ParakeetTDT;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Thread-safe transcription manager for Parakeet models
/// Manages both TDT (batch) and EOU (streaming) contexts
pub struct TranscriptionManager {
    /// TDT context for batch transcription
    tdt_context: Arc<Mutex<Option<ParakeetTDT>>>,
    /// Current transcription state
    state: Arc<Mutex<TranscriptionState>>,
}

impl Default for TranscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptionManager {
    /// Create a new TranscriptionManager without a loaded model
    pub fn new() -> Self {
        Self {
            tdt_context: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(TranscriptionState::Unloaded)),
        }
    }

    /// Get the current transcription state
    pub fn state(&self) -> TranscriptionState {
        self.state
            .lock()
            .map(|guard| *guard)
            .unwrap_or(TranscriptionState::Unloaded)
    }
}

impl TranscriptionService for TranscriptionManager {
    fn load_model(&self, path: &Path) -> TranscriptionResult<()> {
        // Load the TDT model from the given path
        let tdt = ParakeetTDT::from_pretrained(
            path.to_str()
                .ok_or_else(|| TranscriptionError::ModelLoadFailed("Invalid path".to_string()))?,
            None,
        )
        .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;

        // Store context and update state
        {
            let mut guard = self
                .tdt_context
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

        // Perform transcription
        let result = {
            let mut guard = self
                .tdt_context
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;

            let tdt = guard.as_mut().ok_or(TranscriptionError::ModelNotLoaded)?;

            // Transcribe samples (16kHz mono)
            // parakeet-rs takes owned Vec, sample rate, channels, and optional timestamp mode
            tdt.transcribe_samples(samples.to_vec(), 16000, 1, None)
                .map(|result| result.text)
                .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))
        };

        // Update state to Completed or Error based on result
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *state = if result.is_ok() {
                TranscriptionState::Completed
            } else {
                TranscriptionState::Error
            };
        }

        result
    }

    fn is_loaded(&self) -> bool {
        self.tdt_context
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    fn state(&self) -> TranscriptionState {
        TranscriptionManager::state(self)
    }

    fn reset_to_idle(&self) -> TranscriptionResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| TranscriptionError::LockPoisoned)?;

        // Only reset from Completed or Error states
        if *state == TranscriptionState::Completed || *state == TranscriptionState::Error {
            *state = TranscriptionState::Idle;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcription_manager_new_is_unloaded() {
        let manager = TranscriptionManager::new();
        assert!(!manager.is_loaded());
        assert_eq!(manager.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_transcription_manager_default_is_unloaded() {
        let manager = TranscriptionManager::default();
        assert!(!manager.is_loaded());
        assert_eq!(manager.state(), TranscriptionState::Unloaded);
    }

    #[test]
    fn test_transcribe_returns_error_when_model_not_loaded() {
        let manager = TranscriptionManager::new();
        let samples = vec![0.0f32; 16000]; // 1 second of silence
        let result = manager.transcribe(&samples);
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    }

    #[test]
    fn test_transcribe_returns_error_for_empty_audio() {
        let manager = TranscriptionManager::new();
        let samples: Vec<f32> = vec![];
        let result = manager.transcribe(&samples);
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::InvalidAudio(_))));
    }

    #[test]
    fn test_load_model_fails_with_invalid_path() {
        let manager = TranscriptionManager::new();
        let result = manager.load_model(Path::new("/nonexistent/path/to/model"));
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
    }

    #[test]
    fn test_reset_to_idle_from_completed() {
        let manager = TranscriptionManager::new();
        // Manually set state to Completed for testing
        {
            let mut state = manager.state.lock().unwrap();
            *state = TranscriptionState::Completed;
        }
        assert_eq!(manager.state(), TranscriptionState::Completed);

        manager.reset_to_idle().unwrap();
        assert_eq!(manager.state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_reset_to_idle_from_error() {
        let manager = TranscriptionManager::new();
        // Manually set state to Error for testing
        {
            let mut state = manager.state.lock().unwrap();
            *state = TranscriptionState::Error;
        }
        assert_eq!(manager.state(), TranscriptionState::Error);

        manager.reset_to_idle().unwrap();
        assert_eq!(manager.state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_reset_to_idle_noop_from_idle() {
        let manager = TranscriptionManager::new();
        // Set to Idle first
        {
            let mut state = manager.state.lock().unwrap();
            *state = TranscriptionState::Idle;
        }

        manager.reset_to_idle().unwrap();
        assert_eq!(manager.state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_reset_to_idle_noop_from_unloaded() {
        let manager = TranscriptionManager::new();
        assert_eq!(manager.state(), TranscriptionState::Unloaded);

        manager.reset_to_idle().unwrap();
        // Should remain Unloaded, not reset
        assert_eq!(manager.state(), TranscriptionState::Unloaded);
    }
}

// TranscriptionManager for Parakeet-based transcription
// Provides TDT (batch) and EOU (streaming) transcription using NVIDIA Parakeet models

use super::types::{TranscriptionError, TranscriptionMode, TranscriptionResult, TranscriptionService, TranscriptionState};
use parakeet_rs::{ParakeetEOU, ParakeetTDT};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Thread-safe transcription manager for Parakeet models
/// Manages both TDT (batch) and EOU (streaming) contexts
pub struct TranscriptionManager {
    /// TDT context for batch transcription
    tdt_context: Arc<Mutex<Option<ParakeetTDT>>>,
    /// EOU context for streaming transcription
    eou_context: Arc<Mutex<Option<ParakeetEOU>>>,
    /// Current transcription mode
    mode: Arc<Mutex<TranscriptionMode>>,
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
            eou_context: Arc::new(Mutex::new(None)),
            mode: Arc::new(Mutex::new(TranscriptionMode::default())),
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

    /// Load the TDT model from the given directory path
    pub fn load_tdt_model(&self, model_dir: &Path) -> TranscriptionResult<()> {
        let path_str = model_dir.to_str().ok_or_else(|| {
            TranscriptionError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        let tdt = ParakeetTDT::from_pretrained(path_str, None)
            .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;

        {
            let mut guard = self
                .tdt_context
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *guard = Some(tdt);
        }

        // Update state to Idle if this was the first model loaded
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            if *state == TranscriptionState::Unloaded {
                *state = TranscriptionState::Idle;
            }
        }

        Ok(())
    }

    /// Load the EOU model from the given directory path
    pub fn load_eou_model(&self, model_dir: &Path) -> TranscriptionResult<()> {
        let path_str = model_dir.to_str().ok_or_else(|| {
            TranscriptionError::ModelLoadFailed("Invalid path encoding".to_string())
        })?;

        let eou = ParakeetEOU::from_pretrained(path_str, None)
            .map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;

        {
            let mut guard = self
                .eou_context
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            *guard = Some(eou);
        }

        // Update state to Idle if this was the first model loaded
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;
            if *state == TranscriptionState::Unloaded {
                *state = TranscriptionState::Idle;
            }
        }

        Ok(())
    }

    /// Check if TDT model is loaded
    pub fn is_tdt_loaded(&self) -> bool {
        self.tdt_context
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// Check if EOU model is loaded
    pub fn is_eou_loaded(&self) -> bool {
        self.eou_context
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// Get the current transcription mode
    pub fn current_mode(&self) -> TranscriptionMode {
        self.mode
            .lock()
            .map(|guard| *guard)
            .unwrap_or(TranscriptionMode::Batch)
    }

    /// Set the transcription mode
    pub fn set_mode(&self, mode: TranscriptionMode) -> TranscriptionResult<()> {
        let mut guard = self
            .mode
            .lock()
            .map_err(|_| TranscriptionError::LockPoisoned)?;
        *guard = mode;
        Ok(())
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

    fn transcribe(&self, file_path: &str) -> TranscriptionResult<String> {
        // Validate file path
        if file_path.is_empty() {
            return Err(TranscriptionError::InvalidAudio(
                "Empty file path".to_string(),
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

        // Perform transcription using transcribe_file
        let result = {
            let mut guard = self
                .tdt_context
                .lock()
                .map_err(|_| TranscriptionError::LockPoisoned)?;

            let tdt = guard.as_mut().ok_or(TranscriptionError::ModelNotLoaded)?;

            // Use transcribe_file - parakeet-rs handles audio loading and preprocessing
            tdt.transcribe_file(file_path, None)
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
        // Check if the model for the current mode is loaded
        match self.current_mode() {
            TranscriptionMode::Batch => self.is_tdt_loaded(),
            TranscriptionMode::Streaming => self.is_eou_loaded(),
        }
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
        let result = manager.transcribe("/nonexistent/audio.wav");
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelNotLoaded)));
    }

    #[test]
    fn test_transcribe_returns_error_for_empty_path() {
        let manager = TranscriptionManager::new();
        let result = manager.transcribe("");
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

    #[test]
    fn test_transcription_manager_state_transitions() {
        let manager = TranscriptionManager::new();

        // Initial state: Unloaded
        assert_eq!(manager.state(), TranscriptionState::Unloaded);

        // After setting to Idle (simulating model load success)
        {
            let mut state = manager.state.lock().unwrap();
            *state = TranscriptionState::Idle;
        }
        assert_eq!(manager.state(), TranscriptionState::Idle);

        // After setting to Transcribing
        {
            let mut state = manager.state.lock().unwrap();
            *state = TranscriptionState::Transcribing;
        }
        assert_eq!(manager.state(), TranscriptionState::Transcribing);

        // After setting to Completed
        {
            let mut state = manager.state.lock().unwrap();
            *state = TranscriptionState::Completed;
        }
        assert_eq!(manager.state(), TranscriptionState::Completed);

        // Reset to Idle
        manager.reset_to_idle().unwrap();
        assert_eq!(manager.state(), TranscriptionState::Idle);

        // Can also transition from Idle to Error
        {
            let mut state = manager.state.lock().unwrap();
            *state = TranscriptionState::Error;
        }
        assert_eq!(manager.state(), TranscriptionState::Error);

        // Reset from Error to Idle
        manager.reset_to_idle().unwrap();
        assert_eq!(manager.state(), TranscriptionState::Idle);
    }

    #[test]
    fn test_default_mode_is_batch() {
        let manager = TranscriptionManager::new();
        assert_eq!(manager.current_mode(), TranscriptionMode::Batch);
    }

    #[test]
    fn test_set_mode_to_streaming() {
        let manager = TranscriptionManager::new();
        assert_eq!(manager.current_mode(), TranscriptionMode::Batch);

        manager.set_mode(TranscriptionMode::Streaming).unwrap();
        assert_eq!(manager.current_mode(), TranscriptionMode::Streaming);
    }

    #[test]
    fn test_set_mode_back_to_batch() {
        let manager = TranscriptionManager::new();
        manager.set_mode(TranscriptionMode::Streaming).unwrap();
        manager.set_mode(TranscriptionMode::Batch).unwrap();
        assert_eq!(manager.current_mode(), TranscriptionMode::Batch);
    }

    #[test]
    fn test_is_tdt_loaded_false_initially() {
        let manager = TranscriptionManager::new();
        assert!(!manager.is_tdt_loaded());
    }

    #[test]
    fn test_is_eou_loaded_false_initially() {
        let manager = TranscriptionManager::new();
        assert!(!manager.is_eou_loaded());
    }

    #[test]
    fn test_is_loaded_checks_mode_batch() {
        let manager = TranscriptionManager::new();
        // In batch mode, is_loaded checks TDT
        assert_eq!(manager.current_mode(), TranscriptionMode::Batch);
        assert!(!manager.is_loaded());
        assert!(!manager.is_tdt_loaded());
    }

    #[test]
    fn test_is_loaded_checks_mode_streaming() {
        let manager = TranscriptionManager::new();
        manager.set_mode(TranscriptionMode::Streaming).unwrap();
        // In streaming mode, is_loaded checks EOU
        assert_eq!(manager.current_mode(), TranscriptionMode::Streaming);
        assert!(!manager.is_loaded());
        assert!(!manager.is_eou_loaded());
    }

    #[test]
    fn test_load_tdt_model_fails_with_invalid_path() {
        let manager = TranscriptionManager::new();
        let result = manager.load_tdt_model(Path::new("/nonexistent/path/to/model"));
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
    }

    #[test]
    fn test_load_eou_model_fails_with_invalid_path() {
        let manager = TranscriptionManager::new();
        let result = manager.load_eou_model(Path::new("/nonexistent/path/to/model"));
        assert!(result.is_err());
        assert!(matches!(result, Err(TranscriptionError::ModelLoadFailed(_))));
    }
}

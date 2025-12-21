// Listening state manager for always-on wake word detection
// Coordinates with RecordingManager for state transitions

use crate::recording::{RecordingManager, RecordingState};
use serde::Serialize;
use std::sync::Mutex;

/// Information about the current listening status
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListeningStatus {
    /// Whether listening mode is enabled (user preference)
    pub enabled: bool,
    /// Whether listening is currently active (in Listening state)
    pub active: bool,
    /// Whether the microphone is available
    pub mic_available: bool,
}

/// Errors that can occur during listening operations
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ListeningError {
    /// Invalid state transition attempted
    #[error("Cannot change listening state from {current_state:?}")]
    InvalidTransition { current_state: RecordingState },
    /// Recording is in progress
    #[error("Cannot enable listening while recording")]
    RecordingInProgress,
    /// State lock error
    #[error("Failed to acquire state lock")]
    LockError,
    /// Already in the requested state
    #[allow(dead_code)] // Error variant for future use
    #[error("Already in the requested listening state")]
    AlreadyInState,
}

/// Manager for listening mode state
///
/// Tracks whether listening mode is enabled (user preference) and coordinates
/// with RecordingManager for state transitions. The `listening_enabled` flag
/// persists across Recording states to determine return state.
pub struct ListeningManager {
    /// Whether listening mode is enabled by the user
    listening_enabled: bool,
    /// Whether the microphone is currently available
    mic_available: bool,
}

impl ListeningManager {
    /// Create a new ListeningManager with listening disabled
    pub fn new() -> Self {
        Self {
            listening_enabled: false,
            mic_available: true,
        }
    }

    /// Create a new ListeningManager with the specified enabled state
    ///
    /// Used to restore persisted settings on app startup.
    pub fn with_enabled(enabled: bool) -> Self {
        Self {
            listening_enabled: enabled,
            mic_available: true,
        }
    }

    /// Enable listening mode
    ///
    /// Sets the listening_enabled flag and transitions RecordingManager to Listening state
    /// if currently in Idle state.
    ///
    /// # Arguments
    /// * `recording_manager` - The RecordingManager to coordinate with
    ///
    /// # Returns
    /// `Ok(())` if listening was enabled successfully
    ///
    /// # Errors
    /// - `RecordingInProgress` if currently recording
    /// - `InvalidTransition` if state transition fails
    /// - `LockError` if the recording manager lock is poisoned
    pub fn enable_listening(
        &mut self,
        recording_manager: &Mutex<RecordingManager>,
    ) -> Result<(), ListeningError> {
        let mut manager = recording_manager
            .lock()
            .map_err(|_| ListeningError::LockError)?;

        let current_state = manager.get_state();

        match current_state {
            RecordingState::Recording | RecordingState::Processing => {
                Err(ListeningError::RecordingInProgress)
            }
            RecordingState::Listening => {
                // Already listening, just ensure flag is set
                self.listening_enabled = true;
                Ok(())
            }
            RecordingState::Idle => {
                // Transition to Listening state
                manager
                    .transition_to(RecordingState::Listening)
                    .map_err(|_| ListeningError::InvalidTransition { current_state })?;
                self.listening_enabled = true;
                Ok(())
            }
        }
    }

    /// Disable listening mode
    ///
    /// Clears the listening_enabled flag and transitions RecordingManager to Idle state
    /// if currently in Listening state.
    ///
    /// # Arguments
    /// * `recording_manager` - The RecordingManager to coordinate with
    ///
    /// # Returns
    /// `Ok(())` if listening was disabled successfully
    ///
    /// # Errors
    /// - `InvalidTransition` if state transition fails
    /// - `LockError` if the recording manager lock is poisoned
    pub fn disable_listening(
        &mut self,
        recording_manager: &Mutex<RecordingManager>,
    ) -> Result<(), ListeningError> {
        let mut manager = recording_manager
            .lock()
            .map_err(|_| ListeningError::LockError)?;

        let current_state = manager.get_state();

        // Always clear the flag, regardless of current state
        self.listening_enabled = false;

        match current_state {
            RecordingState::Listening => {
                // Transition to Idle state
                manager
                    .transition_to(RecordingState::Idle)
                    .map_err(|_| ListeningError::InvalidTransition { current_state })?;
                Ok(())
            }
            RecordingState::Recording | RecordingState::Processing => {
                // Recording in progress - flag is cleared but state stays
                // After recording completes, will return to Idle instead of Listening
                Ok(())
            }
            RecordingState::Idle => {
                // Already idle, just clear flag
                Ok(())
            }
        }
    }

    /// Get the current listening status
    ///
    /// # Arguments
    /// * `recording_manager` - The RecordingManager to check state from
    ///
    /// # Returns
    /// Current listening status including enabled flag, active state, and mic availability
    pub fn get_status(
        &self,
        recording_manager: &Mutex<RecordingManager>,
    ) -> Result<ListeningStatus, ListeningError> {
        let manager = recording_manager
            .lock()
            .map_err(|_| ListeningError::LockError)?;

        let current_state = manager.get_state();
        let active = current_state == RecordingState::Listening;

        Ok(ListeningStatus {
            enabled: self.listening_enabled,
            active,
            mic_available: self.mic_available,
        })
    }

    /// Check if listening mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.listening_enabled
    }

    /// Set microphone availability
    ///
    /// Called when the microphone becomes available or unavailable.
    #[allow(dead_code)] // Future use for mic status tracking
    pub fn set_mic_available(&mut self, available: bool) {
        self.mic_available = available;
    }

    /// Check if microphone is available
    #[allow(dead_code)] // Used in tests and for status checks
    pub fn is_mic_available(&self) -> bool {
        self.mic_available
    }

    /// Get the target state after recording completes
    ///
    /// Returns `Listening` if listening_enabled is true, otherwise `Idle`.
    /// Used by stop_recording to determine the return state.
    #[allow(dead_code)] // Future use for post-recording state determination
    pub fn get_post_recording_state(&self) -> RecordingState {
        if self.listening_enabled {
            RecordingState::Listening
        } else {
            RecordingState::Idle
        }
    }
}

impl Default for ListeningManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_recording_manager() -> Mutex<RecordingManager> {
        Mutex::new(RecordingManager::new())
    }

    #[test]
    fn test_new_listening_manager_disabled() {
        let manager = ListeningManager::new();
        assert!(!manager.is_enabled());
        assert!(manager.is_mic_available());
    }

    #[test]
    fn test_default_listening_manager_disabled() {
        let manager = ListeningManager::default();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_with_enabled_true() {
        let manager = ListeningManager::with_enabled(true);
        assert!(manager.is_enabled());
        assert!(manager.is_mic_available());
    }

    #[test]
    fn test_with_enabled_false() {
        let manager = ListeningManager::with_enabled(false);
        assert!(!manager.is_enabled());
        assert!(manager.is_mic_available());
    }

    #[test]
    fn test_enable_listening_from_idle() {
        let mut listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        let result = listening_manager.enable_listening(&recording_manager);
        assert!(result.is_ok());
        assert!(listening_manager.is_enabled());

        let rm = recording_manager.lock().unwrap();
        assert_eq!(rm.get_state(), RecordingState::Listening);
    }

    #[test]
    fn test_enable_listening_already_listening() {
        let mut listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        listening_manager.enable_listening(&recording_manager).unwrap();
        let result = listening_manager.enable_listening(&recording_manager);
        assert!(result.is_ok());
        assert!(listening_manager.is_enabled());
    }

    #[test]
    fn test_enable_listening_fails_while_recording() {
        let mut listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        // Start recording
        {
            let mut rm = recording_manager.lock().unwrap();
            rm.start_recording(16000).unwrap();
        }

        let result = listening_manager.enable_listening(&recording_manager);
        assert!(matches!(result, Err(ListeningError::RecordingInProgress)));
    }

    #[test]
    fn test_disable_listening_from_listening() {
        let mut listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        listening_manager.enable_listening(&recording_manager).unwrap();
        let result = listening_manager.disable_listening(&recording_manager);
        assert!(result.is_ok());
        assert!(!listening_manager.is_enabled());

        let rm = recording_manager.lock().unwrap();
        assert_eq!(rm.get_state(), RecordingState::Idle);
    }

    #[test]
    fn test_disable_listening_from_idle() {
        let mut listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        let result = listening_manager.disable_listening(&recording_manager);
        assert!(result.is_ok());
        assert!(!listening_manager.is_enabled());
    }

    #[test]
    fn test_disable_listening_during_recording_clears_flag() {
        let mut listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        // Enable listening first
        listening_manager.enable_listening(&recording_manager).unwrap();
        assert!(listening_manager.is_enabled());

        // Start recording (transitions from Listening to Recording)
        {
            let mut rm = recording_manager.lock().unwrap();
            rm.start_recording(16000).unwrap();
        }

        // Disable while recording
        let result = listening_manager.disable_listening(&recording_manager);
        assert!(result.is_ok());
        assert!(!listening_manager.is_enabled());

        // Recording state unchanged
        let rm = recording_manager.lock().unwrap();
        assert_eq!(rm.get_state(), RecordingState::Recording);
    }

    #[test]
    fn test_get_status_when_listening() {
        let mut listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        listening_manager.enable_listening(&recording_manager).unwrap();
        let status = listening_manager.get_status(&recording_manager).unwrap();

        assert!(status.enabled);
        assert!(status.active);
        assert!(status.mic_available);
    }

    #[test]
    fn test_get_status_when_idle() {
        let listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        let status = listening_manager.get_status(&recording_manager).unwrap();

        assert!(!status.enabled);
        assert!(!status.active);
        assert!(status.mic_available);
    }

    #[test]
    fn test_get_status_enabled_but_recording() {
        let mut listening_manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        // Enable listening
        listening_manager.enable_listening(&recording_manager).unwrap();

        // Start recording
        {
            let mut rm = recording_manager.lock().unwrap();
            rm.start_recording(16000).unwrap();
        }

        let status = listening_manager.get_status(&recording_manager).unwrap();
        assert!(status.enabled); // Flag still set
        assert!(!status.active); // Not in Listening state
    }

    #[test]
    fn test_set_mic_available() {
        let mut manager = ListeningManager::new();
        assert!(manager.is_mic_available());

        manager.set_mic_available(false);
        assert!(!manager.is_mic_available());

        manager.set_mic_available(true);
        assert!(manager.is_mic_available());
    }

    #[test]
    fn test_get_post_recording_state_with_listening_enabled() {
        let mut manager = ListeningManager::new();
        let recording_manager = create_test_recording_manager();

        manager.enable_listening(&recording_manager).unwrap();
        assert_eq!(manager.get_post_recording_state(), RecordingState::Listening);
    }

    #[test]
    fn test_get_post_recording_state_with_listening_disabled() {
        let manager = ListeningManager::new();
        assert_eq!(manager.get_post_recording_state(), RecordingState::Idle);
    }

    // Tests removed per docs/TESTING.md:
    // - test_listening_error_display: Display trait test
    // - test_listening_status_serialization: Serialization derive test
}

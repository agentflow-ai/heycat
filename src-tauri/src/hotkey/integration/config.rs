//! Configuration structs for HotkeyIntegration.
//!
//! These structs group related fields for improved maintainability.
//! Each config struct represents a logical capability that can be optionally enabled.

use crate::events::{CommandEventEmitter, TranscriptionEventEmitter};
use crate::hotkey::double_tap::DEFAULT_DOUBLE_TAP_WINDOW_MS;
use crate::hotkey::{NullShortcutBackend, ShortcutBackend};
use crate::parakeet::SharedTranscriptionModel;
use crate::recording::SilenceConfig;
use crate::turso::TursoClient;
use crate::voice_commands::executor::ActionDispatcher;
use crate::voice_commands::matcher::CommandMatcher;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

/// Maximum concurrent transcriptions allowed
pub const MAX_CONCURRENT_TRANSCRIPTIONS: usize = 2;

/// Default transcription timeout in seconds
/// If transcription takes longer than this, it will be cancelled and an error emitted
pub const DEFAULT_TRANSCRIPTION_TIMEOUT_SECS: u64 = 60;

/// Debounce duration for hotkey presses (200ms)
pub const DEBOUNCE_DURATION_MS: u64 = 200;

/// Configuration for transcription capabilities
///
/// Groups all fields needed for automatic transcription after recording stops.
/// When present, recordings will be automatically transcribed and events emitted.
///
/// Note: Both `shared_model` and `emitter` must be Some for transcription to work.
/// The Option wrappers allow incremental builder pattern configuration.
pub struct TranscriptionConfig<T: TranscriptionEventEmitter> {
    /// Shared transcription model for performing transcriptions
    pub shared_model: Option<Arc<SharedTranscriptionModel>>,
    /// Event emitter for transcription events (started, completed, error)
    pub emitter: Option<Arc<T>>,
    /// Semaphore to limit concurrent transcriptions
    pub semaphore: Arc<Semaphore>,
    /// Maximum time allowed for transcription before timeout
    pub timeout: Duration,
    /// Optional callback for delegating transcription to external service
    pub callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

/// Configuration for silence detection during recording
///
/// Controls whether and how silence detection triggers auto-stop.
#[derive(Clone)]
pub struct SilenceDetectionConfig {
    /// Whether silence detection is enabled (default: true)
    pub enabled: bool,
    /// Custom silence configuration (optional, uses defaults if None)
    pub config: Option<SilenceConfig>,
}

impl Default for SilenceDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            config: None,
        }
    }
}

/// Configuration for voice command matching and execution
///
/// Groups all fields needed for matching transcribed text to commands
/// and executing the matched actions.
///
/// Note: All fields except `emitter` have default implementations.
/// The `emitter` must be set for command events to be emitted.
pub struct VoiceCommandConfig<C: CommandEventEmitter> {
    /// Turso client for fetching voice commands
    pub turso_client: Arc<TursoClient>,
    /// Matcher for finding commands from transcribed text
    pub matcher: Arc<CommandMatcher>,
    /// Dispatcher for executing matched command actions
    pub dispatcher: Arc<ActionDispatcher>,
    /// Event emitter for command events (matched, executed, failed, ambiguous)
    /// Optional to support incremental builder pattern
    pub emitter: Option<Arc<C>>,
}

/// Configuration for Escape key cancel functionality
///
/// Groups fields for double-tap Escape key detection to cancel recording.
///
/// Note: Both `backend` and `callback` must be Some for escape handling to work.
/// The Option wrappers on callback allow incremental builder pattern configuration.
pub struct EscapeKeyConfig {
    /// Backend for registering/unregistering global shortcuts
    pub backend: Arc<dyn ShortcutBackend + Send + Sync>,
    /// Callback invoked when double-tap Escape is detected (optional for incremental config)
    pub callback: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Time window for double-tap detection in milliseconds.
    /// See [`DEFAULT_DOUBLE_TAP_WINDOW_MS`](crate::hotkey::double_tap::DEFAULT_DOUBLE_TAP_WINDOW_MS) for the default value (300ms).
    pub double_tap_window_ms: u64,
}

impl Default for EscapeKeyConfig {
    fn default() -> Self {
        Self {
            backend: Arc::new(NullShortcutBackend),
            callback: None,
            double_tap_window_ms: DEFAULT_DOUBLE_TAP_WINDOW_MS,
        }
    }
}

/// Result of executing a transcription task
pub struct TranscriptionResult {
    /// The transcribed text
    pub text: String,
    /// Duration of transcription in milliseconds
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detection_config_default() {
        let config = SilenceDetectionConfig::default();
        assert!(config.enabled);
        assert!(config.config.is_none());
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_CONCURRENT_TRANSCRIPTIONS, 2);
        assert_eq!(DEFAULT_TRANSCRIPTION_TIMEOUT_SECS, 60);
        assert_eq!(DEBOUNCE_DURATION_MS, 200);
    }
}

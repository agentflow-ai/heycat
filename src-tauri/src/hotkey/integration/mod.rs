//! Hotkey-to-recording integration module
//!
//! Connects global hotkey to recording state with debouncing.
//! Uses unified command implementations for start/stop logic.
//!
//! ## Module Organization
//!
//! - `config`: Configuration structs for transcription, silence detection, voice commands, etc.
//! - `toggle_handler`: Handle toggle mode (press once to start, again to stop)
//! - `ptt_handler`: Handle push-to-talk mode (hold to record, release to stop)
//! - `cancel_handler`: Handle recording cancellation via double-tap Escape
//! - `transcription_runner`: Core transcription execution and voice command matching
//! - `silence_handler`: Silence detection for auto-stop recordings
//! - `escape_handler`: Escape key listener registration/unregistration
//! - `clipboard_helper`: Clipboard and paste simulation utilities

mod cancel_handler;
mod clipboard_helper;
pub mod config;
mod escape_handler;
mod ptt_handler;
mod silence_handler;
mod toggle_handler;
mod transcription_runner;

#[cfg(test)]
mod cancel_handler_test;
#[cfg(test)]
mod ptt_handler_test;
#[cfg(test)]
mod toggle_handler_test;

pub use config::{
    EscapeKeyConfig, SilenceDetectionConfig, TranscriptionConfig, VoiceCommandConfig,
    DEBOUNCE_DURATION_MS, DEFAULT_TRANSCRIPTION_TIMEOUT_SECS, MAX_CONCURRENT_TRANSCRIPTIONS,
};

use crate::audio::{AudioMonitorHandle, AudioThreadHandle};
use crate::events::{CommandEventEmitter, HotkeyEventEmitter, RecordingEventEmitter, TranscriptionEventEmitter};
use crate::hotkey::double_tap::{DoubleTapDetector, DEFAULT_DOUBLE_TAP_WINDOW_MS};
use crate::hotkey::{RecordingMode, ShortcutBackend};
use crate::parakeet::SharedTranscriptionModel;
use crate::recording::{RecordingDetectors, RecordingManager, SilenceConfig};
use crate::turso::TursoClient;
use crate::voice_commands::executor::ActionDispatcher;
use crate::voice_commands::matcher::CommandMatcher;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio::sync::Semaphore;

/// Type alias for the double-tap detector with callback
type DoubleTapDetectorState = Option<Arc<Mutex<DoubleTapDetector<Box<dyn Fn() + Send + Sync>>>>>;

/// Handles hotkey toggle with debouncing and event emission
///
/// Configuration is organized into logical sub-structs:
/// - `transcription`: Transcription capabilities (model, emitter, timeout)
/// - `audio`: Audio capture and recording management
/// - `voice_commands`: Voice command matching and execution
/// - `escape`: Escape key cancel functionality
/// - `silence`: Silence detection configuration
///
/// Top-level fields handle debouncing, listening state, and app integration.
pub struct HotkeyIntegration<
    R: RecordingEventEmitter,
    T: TranscriptionEventEmitter,
    C: CommandEventEmitter,
> {
    // === Debounce/Timing ===
    pub(crate) last_toggle_time: Option<Instant>,
    pub(crate) debounce_duration: Duration,

    // === Recording Mode ===
    /// Current recording mode (Toggle or PushToTalk)
    recording_mode: RecordingMode,

    // === Recording (required) ===
    pub(crate) recording_emitter: R,

    // === Grouped Configurations ===
    /// Transcription configuration (model, emitter, semaphore, timeout)
    pub(crate) transcription: Option<TranscriptionConfig<T>>,
    /// Voice command configuration (registry, matcher, dispatcher, emitter)
    pub(crate) voice_commands: Option<VoiceCommandConfig<C>>,
    /// Escape key configuration (backend, callback, double-tap window)
    pub(crate) escape: Option<EscapeKeyConfig>,
    /// Silence detection configuration (public for test assertions)
    pub(crate) silence: SilenceDetectionConfig,

    // === Audio ===
    /// Optional audio thread handle - when present, starts/stops capture on toggle
    pub(crate) audio_thread: Option<Arc<AudioThreadHandle>>,
    /// Optional audio monitor handle - stopped before recording to prevent device conflict
    pub(crate) audio_monitor: Option<Arc<AudioMonitorHandle>>,
    /// Reference to recording state for getting audio buffer in transcription thread
    pub(crate) recording_state: Option<Arc<Mutex<RecordingManager>>>,
    /// Recording detectors for silence-based auto-stop
    pub(crate) recording_detectors: Option<Arc<Mutex<RecordingDetectors>>>,

    // === App Integration ===
    /// Optional app handle for clipboard access
    pub(crate) app_handle: Option<AppHandle>,
    /// Directory for saving recordings (supports worktree isolation)
    pub(crate) recordings_dir: std::path::PathBuf,

    // === Escape Key Runtime State ===
    /// Whether Escape key is currently registered (to track cleanup)
    pub(crate) escape_registered: Arc<AtomicBool>,
    /// Double-tap detector for Escape key (created on recording start)
    pub(crate) double_tap_detector: DoubleTapDetectorState,

    // === Hotkey Events ===
    /// Optional emitter for hotkey-related events (e.g., key blocking unavailable)
    pub(crate) hotkey_emitter: Option<Arc<dyn HotkeyEventEmitter>>,
}

impl<R: RecordingEventEmitter, T: TranscriptionEventEmitter + 'static, C: CommandEventEmitter + 'static>
    HotkeyIntegration<R, T, C>
{
    /// Create a new HotkeyIntegration with default debounce duration
    pub fn new(recording_emitter: R) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(DEBOUNCE_DURATION_MS),
            recording_mode: RecordingMode::default(),
            recording_emitter,
            transcription: None,
            voice_commands: None,
            escape: None,
            silence: SilenceDetectionConfig::default(),
            audio_thread: None,
            audio_monitor: None,
            recording_state: None,
            recording_detectors: None,
            app_handle: None,
            recordings_dir: crate::paths::get_recordings_dir(None)
                .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat").join("recordings")),
            escape_registered: Arc::new(AtomicBool::new(false)),
            double_tap_detector: None,
            hotkey_emitter: None,
        }
    }

    /// Get the current recording mode
    pub fn recording_mode(&self) -> RecordingMode {
        self.recording_mode
    }

    /// Update the recording mode at runtime
    pub fn set_recording_mode(&mut self, mode: RecordingMode) {
        self.recording_mode = mode;
        crate::debug!("Recording mode updated to: {:?}", mode);
    }

    /// Add app handle for clipboard access (builder pattern)
    pub fn with_app_handle(mut self, handle: AppHandle) -> Self {
        self.app_handle = Some(handle);
        self
    }

    /// Add recordings directory (builder pattern)
    pub fn with_recordings_dir(mut self, recordings_dir: std::path::PathBuf) -> Self {
        self.recordings_dir = recordings_dir;
        self
    }

    /// Get the selected audio device from persistent settings store
    pub(crate) fn get_selected_audio_device(&self) -> Option<String> {
        use crate::util::SettingsAccess;
        struct OptionalAppHandle<'a>(&'a Option<AppHandle>);
        impl SettingsAccess for OptionalAppHandle<'_> {
            fn app_handle(&self) -> Option<&AppHandle> {
                self.0.as_ref()
            }
        }
        OptionalAppHandle(&self.app_handle).get_setting("audio.selectedDevice")
    }

    /// Add an audio thread handle (builder pattern)
    pub fn with_audio_thread(mut self, handle: Arc<AudioThreadHandle>) -> Self {
        self.audio_thread = Some(handle);
        self
    }

    /// Add an audio monitor handle (builder pattern)
    pub fn with_audio_monitor(mut self, handle: Arc<AudioMonitorHandle>) -> Self {
        self.audio_monitor = Some(handle);
        self
    }

    /// Add SharedTranscriptionModel for auto-transcription (builder pattern)
    pub fn with_shared_transcription_model(mut self, model: Arc<SharedTranscriptionModel>) -> Self {
        if let Some(ref mut config) = self.transcription {
            config.shared_model = Some(model);
        } else {
            self.transcription = Some(TranscriptionConfig {
                shared_model: Some(model),
                emitter: None,
                semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
                timeout: Duration::from_secs(DEFAULT_TRANSCRIPTION_TIMEOUT_SECS),
                callback: None,
            });
        }
        self
    }

    /// Add transcription event emitter (builder pattern)
    pub fn with_transcription_emitter(mut self, emitter: Arc<T>) -> Self {
        if let Some(ref mut config) = self.transcription {
            config.emitter = Some(emitter);
        } else {
            self.transcription = Some(TranscriptionConfig {
                shared_model: None,
                emitter: Some(emitter),
                semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
                timeout: Duration::from_secs(DEFAULT_TRANSCRIPTION_TIMEOUT_SECS),
                callback: None,
            });
        }
        self
    }

    /// Add recording state reference (builder pattern)
    pub fn with_recording_state(mut self, state: Arc<Mutex<RecordingManager>>) -> Self {
        self.recording_state = Some(state);
        self
    }

    /// Add complete voice command configuration (builder pattern)
    pub fn with_voice_commands(mut self, config: VoiceCommandConfig<C>) -> Self {
        self.voice_commands = Some(config);
        self
    }

    /// Add Turso client for voice command fetching (builder pattern)
    #[allow(dead_code)]
    pub fn with_turso_client(mut self, turso_client: Arc<TursoClient>) -> Self {
        if let Some(ref mut config) = self.voice_commands {
            config.turso_client = turso_client;
        } else {
            self.voice_commands = Some(VoiceCommandConfig {
                turso_client,
                matcher: Arc::new(CommandMatcher::new()),
                dispatcher: Arc::new(ActionDispatcher::new()),
                emitter: None,
            });
        }
        self
    }

    /// Add command matcher (builder pattern)
    #[allow(dead_code)]
    pub fn with_command_matcher(mut self, matcher: Arc<CommandMatcher>) -> Self {
        if let Some(ref mut config) = self.voice_commands {
            config.matcher = matcher;
        }
        self
    }

    /// Add action dispatcher (builder pattern)
    #[allow(dead_code)]
    pub fn with_action_dispatcher(mut self, dispatcher: Arc<ActionDispatcher>) -> Self {
        if let Some(ref mut config) = self.voice_commands {
            config.dispatcher = dispatcher;
        }
        self
    }

    /// Add command event emitter (builder pattern)
    #[allow(dead_code)]
    pub fn with_command_emitter(mut self, emitter: Arc<C>) -> Self {
        if let Some(ref mut config) = self.voice_commands {
            config.emitter = Some(emitter);
        }
        self
    }

    /// Set custom transcription timeout (builder pattern)
    #[allow(dead_code)]
    pub fn with_transcription_timeout(mut self, timeout: Duration) -> Self {
        if let Some(ref mut config) = self.transcription {
            config.timeout = timeout;
        } else {
            self.transcription = Some(TranscriptionConfig {
                shared_model: None,
                emitter: None,
                semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
                timeout,
                callback: None,
            });
        }
        self
    }

    /// Add recording detectors for silence-based auto-stop (builder pattern)
    pub fn with_recording_detectors(mut self, detectors: Arc<Mutex<RecordingDetectors>>) -> Self {
        self.recording_detectors = Some(detectors);
        self
    }

    /// Enable or disable silence detection (builder pattern)
    #[allow(dead_code)]
    pub fn with_silence_detection_enabled(mut self, enabled: bool) -> Self {
        self.silence.enabled = enabled;
        self
    }

    /// Set custom silence configuration (builder pattern)
    #[allow(dead_code)]
    pub fn with_silence_config(mut self, config: SilenceConfig) -> Self {
        self.silence.config = Some(config);
        self
    }

    /// Add complete escape key configuration (builder pattern)
    #[allow(dead_code)]
    pub fn with_escape(mut self, config: EscapeKeyConfig) -> Self {
        self.escape = Some(config);
        self
    }

    /// Add shortcut backend for Escape key registration (builder pattern)
    pub fn with_shortcut_backend(mut self, backend: Arc<dyn ShortcutBackend + Send + Sync>) -> Self {
        if let Some(ref mut config) = self.escape {
            config.backend = backend;
        } else {
            self.escape = Some(EscapeKeyConfig {
                backend,
                callback: None,
                double_tap_window_ms: DEFAULT_DOUBLE_TAP_WINDOW_MS,
            });
        }
        self
    }

    /// Set escape callback (builder pattern)
    #[allow(dead_code)]
    pub fn with_escape_callback(mut self, callback: Arc<dyn Fn() + Send + Sync>) -> Self {
        if let Some(ref mut config) = self.escape {
            config.callback = Some(callback);
        } else {
            self.escape = Some(EscapeKeyConfig {
                backend: Arc::new(crate::hotkey::NullShortcutBackend),
                callback: Some(callback),
                double_tap_window_ms: DEFAULT_DOUBLE_TAP_WINDOW_MS,
            });
        }
        self
    }

    /// Set the time window for double-tap detection (builder pattern)
    #[allow(dead_code)]
    pub fn with_double_tap_window(mut self, window_ms: u64) -> Self {
        if let Some(ref mut config) = self.escape {
            config.double_tap_window_ms = window_ms;
        } else {
            self.escape = Some(EscapeKeyConfig {
                backend: Arc::new(crate::hotkey::NullShortcutBackend),
                callback: None,
                double_tap_window_ms: window_ms,
            });
        }
        self
    }

    /// Set the hotkey event emitter (builder pattern)
    pub fn with_hotkey_emitter(mut self, emitter: Arc<dyn HotkeyEventEmitter>) -> Self {
        self.hotkey_emitter = Some(emitter);
        self
    }

    /// Set the transcription callback (builder pattern)
    pub fn with_transcription_callback(mut self, callback: Arc<dyn Fn(String) + Send + Sync>) -> Self {
        if let Some(ref mut config) = self.transcription {
            config.callback = Some(callback);
        } else {
            self.transcription = Some(TranscriptionConfig {
                shared_model: None,
                emitter: None,
                semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_TRANSCRIPTIONS)),
                timeout: Duration::from_secs(DEFAULT_TRANSCRIPTION_TIMEOUT_SECS),
                callback: Some(callback),
            });
        }
        self
    }

    /// Set the escape callback after construction
    pub fn set_escape_callback(&mut self, callback: Arc<dyn Fn() + Send + Sync>) {
        if let Some(ref mut config) = self.escape {
            config.callback = Some(callback);
        } else {
            self.escape = Some(EscapeKeyConfig {
                backend: Arc::new(crate::hotkey::NullShortcutBackend),
                callback: Some(callback),
                double_tap_window_ms: DEFAULT_DOUBLE_TAP_WINDOW_MS,
            });
        }
    }

    /// Create with custom debounce duration (for testing)
    #[cfg(test)]
    pub fn with_debounce(recording_emitter: R, debounce_ms: u64) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(debounce_ms),
            recording_mode: RecordingMode::default(),
            recording_emitter,
            transcription: None,
            voice_commands: None,
            escape: None,
            silence: SilenceDetectionConfig::default(),
            audio_thread: None,
            audio_monitor: None,
            recording_state: None,
            recording_detectors: None,
            app_handle: None,
            recordings_dir: std::env::temp_dir().join("heycat-test-recordings"),
            escape_registered: Arc::new(AtomicBool::new(false)),
            double_tap_detector: None,
            hotkey_emitter: None,
        }
    }

    /// Check if currently in debounce window (for testing)
    #[cfg(test)]
    pub fn is_debouncing(&self) -> bool {
        if let Some(last) = self.last_toggle_time {
            Instant::now().duration_since(last) < self.debounce_duration
        } else {
            false
        }
    }
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

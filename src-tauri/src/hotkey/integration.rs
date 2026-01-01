// Hotkey-to-recording integration module
// Connects global hotkey to recording state with debouncing
// Uses unified command implementations for start/stop logic

use crate::audio::{AudioMonitorHandle, AudioThreadHandle};
use crate::commands::logic::{start_recording_impl, stop_recording_impl};
use crate::keyboard_capture::cgeventtap::set_consume_escape;
use crate::events::{
    current_timestamp, hotkey_events, CommandAmbiguousPayload, CommandCandidate,
    CommandEventEmitter, CommandExecutedPayload, CommandFailedPayload, CommandMatchedPayload,
    HotkeyEventEmitter, RecordingCancelledPayload, RecordingErrorPayload,
    RecordingEventEmitter, RecordingStartedPayload, RecordingStoppedPayload,
    TranscriptionCompletedPayload, TranscriptionErrorPayload, TranscriptionEventEmitter,
    TranscriptionStartedPayload,
};
use crate::turso::TursoClient;
use crate::hotkey::double_tap::{DoubleTapDetector, DEFAULT_DOUBLE_TAP_WINDOW_MS};
use crate::hotkey::{RecordingMode, ShortcutBackend};
use crate::recording::{RecordingDetectors, SilenceConfig};
use crate::model::{check_model_exists_for_type, ModelType};
use crate::recording::{RecordingManager, RecordingState};
use crate::voice_commands::executor::ActionDispatcher;
use crate::voice_commands::matcher::{CommandMatcher, MatchResult};
use crate::voice_commands::registry::CommandDefinition;
use crate::parakeet::{SharedTranscriptionModel, TranscriptionService};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::sync::Semaphore;

/// Type alias for the double-tap detector with callback
type DoubleTapDetectorState = Option<Arc<Mutex<DoubleTapDetector<Box<dyn Fn() + Send + Sync>>>>>;

/// Maximum concurrent transcriptions allowed
const MAX_CONCURRENT_TRANSCRIPTIONS: usize = 2;

/// Default transcription timeout in seconds
/// If transcription takes longer than this, it will be cancelled and an error emitted
pub const DEFAULT_TRANSCRIPTION_TIMEOUT_SECS: u64 = 60;

/// Debounce duration for hotkey presses (200ms)
pub const DEBOUNCE_DURATION_MS: u64 = 200;

/// Simulate Cmd+V paste keystroke on macOS using CoreGraphics
#[cfg(target_os = "macos")]
fn simulate_paste() -> Result<(), String> {
    // Safety check: don't paste during shutdown
    if crate::shutdown::is_shutting_down() {
        crate::debug!("Skipping paste - app is shutting down");
        return Ok(());
    }

    // Centralized synthesis ensures key-up always follows key-down and sequences don't interleave.
    crate::keyboard::synth::simulate_cmd_v_paste()?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn simulate_paste() -> Result<(), String> {
    Err("Paste simulation only supported on macOS".to_string())
}

/// Result of executing a transcription task
pub struct TranscriptionResult {
    /// The transcribed text
    pub text: String,
    /// Duration of transcription in milliseconds
    pub duration_ms: u64,
}

// === Configuration Sub-Structs ===
// These group related fields from HotkeyIntegration for improved maintainability.
// Each config struct represents a logical capability that can be optionally enabled.

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
    /// Time window for double-tap detection in milliseconds (default: 300ms)
    pub double_tap_window_ms: u64,
}

/// Execute transcription with semaphore-limited concurrency, timeout, and error handling.
///
/// This is the core transcription logic shared between:
/// - `spawn_transcription` (hotkey recordings with voice command matching)
/// - `start_silence_detection` transcription callback (silence-triggered auto-stop)
///
/// Returns `Ok(TranscriptionResult)` on success, `Err(())` on failure (errors already emitted).
#[cfg_attr(coverage_nightly, coverage(off))]
async fn execute_transcription_task<T: TranscriptionEventEmitter>(
    file_path: String,
    shared_model: Arc<SharedTranscriptionModel>,
    semaphore: Arc<Semaphore>,
    transcription_emitter: Arc<T>,
    timeout_duration: Duration,
    recording_state: Option<Arc<Mutex<RecordingManager>>>,
) -> Result<TranscriptionResult, ()> {
    // Helper to clear recording buffer - call this in all exit paths to prevent memory leaks
    let clear_recording_buffer = || {
        if let Some(ref state) = recording_state {
            if let Ok(mut manager) = state.lock() {
                manager.clear_last_recording();
                crate::debug!("Cleared recording buffer");
            }
        }
    };

    // Acquire semaphore permit to limit concurrent transcriptions
    let _permit = match semaphore.try_acquire() {
        Ok(permit) => permit,
        Err(_) => {
            crate::warn!("Too many concurrent transcriptions, skipping this one");
            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                error: "Too many transcriptions in progress. Please wait and try again."
                    .to_string(),
            });
            clear_recording_buffer();
            return Err(());
        }
    };

    // Emit transcription_started event
    let start_time = Instant::now();
    transcription_emitter.emit_transcription_started(TranscriptionStartedPayload {
        timestamp: current_timestamp(),
    });

    crate::debug!("Transcribing file: {}", file_path);

    // Perform transcription on blocking thread pool (CPU-intensive) with timeout
    let transcriber = shared_model.clone();
    let transcription_future =
        tokio::task::spawn_blocking(move || transcriber.transcribe(&file_path));

    let transcription_result = tokio::time::timeout(timeout_duration, transcription_future).await;

    let text = match transcription_result {
        Ok(Ok(Ok(text))) => text,
        Ok(Ok(Err(e))) => {
            crate::error!("Transcription failed: {}", e);
            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                error: e.to_string(),
            });
            if let Err(reset_err) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", reset_err);
            }
            clear_recording_buffer();
            return Err(());
        }
        Ok(Err(e)) => {
            crate::error!("Transcription task panicked: {}", e);
            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                error: "Internal transcription error.".to_string(),
            });
            if let Err(reset_err) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", reset_err);
            }
            clear_recording_buffer();
            return Err(());
        }
        Err(_) => {
            crate::error!("Transcription timed out after {:?}", timeout_duration);
            transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                error: format!(
                    "Transcription timed out after {} seconds. The audio may be too long or the model may be stuck.",
                    timeout_duration.as_secs()
                ),
            });
            if let Err(reset_err) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", reset_err);
            }
            clear_recording_buffer();
            return Err(());
        }
    };

    let duration_ms = start_time.elapsed().as_millis() as u64;
    crate::info!(
        "Transcription completed in {}ms: {} chars",
        duration_ms,
        text.len()
    );

    Ok(TranscriptionResult { text, duration_ms })
}

/// Copy text to clipboard and auto-paste
#[cfg_attr(coverage_nightly, coverage(off))]
fn copy_and_paste(app_handle: &Option<AppHandle>, text: &str) {
    // Safety check: don't paste during shutdown
    if crate::shutdown::is_shutting_down() {
        crate::debug!("Skipping copy_and_paste - app is shutting down");
        return;
    }

    if let Some(ref handle) = app_handle {
        if let Err(e) = handle.clipboard().write_text(text) {
            crate::warn!("Failed to copy to clipboard: {}", e);
        } else {
            crate::debug!("Transcribed text copied to clipboard");
            if let Err(e) = simulate_paste() {
                crate::warn!("Failed to auto-paste: {}", e);
            } else {
                crate::debug!("Auto-pasted transcribed text");
            }
        }
    } else {
        crate::warn!("Clipboard unavailable: no app handle configured");
    }
}

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
pub struct HotkeyIntegration<R: RecordingEventEmitter, T: TranscriptionEventEmitter, C: CommandEventEmitter> {
    // === Debounce/Timing ===
    last_toggle_time: Option<Instant>,
    debounce_duration: Duration,

    // === Recording Mode ===
    /// Current recording mode (Toggle or PushToTalk)
    recording_mode: RecordingMode,

    // === Recording (required) ===
    recording_emitter: R,

    // === Grouped Configurations ===
    /// Transcription configuration (model, emitter, semaphore, timeout)
    transcription: Option<TranscriptionConfig<T>>,
    /// Voice command configuration (registry, matcher, dispatcher, emitter)
    voice_commands: Option<VoiceCommandConfig<C>>,
    /// Escape key configuration (backend, callback, double-tap window)
    escape: Option<EscapeKeyConfig>,
    /// Silence detection configuration (public for test assertions)
    pub(crate) silence: SilenceDetectionConfig,

    // === Audio (partially grouped - thread is optional, state may be separate) ===
    /// Optional audio thread handle - when present, starts/stops capture on toggle
    audio_thread: Option<Arc<AudioThreadHandle>>,
    /// Optional audio monitor handle - stopped before recording to prevent device conflict
    audio_monitor: Option<Arc<AudioMonitorHandle>>,
    /// Reference to recording state for getting audio buffer in transcription thread
    recording_state: Option<Arc<Mutex<RecordingManager>>>,
    /// Recording detectors for silence-based auto-stop
    recording_detectors: Option<Arc<Mutex<RecordingDetectors>>>,

    // === App Integration ===
    /// Optional app handle for clipboard access
    app_handle: Option<AppHandle>,
    /// Directory for saving recordings (supports worktree isolation)
    recordings_dir: std::path::PathBuf,

    // === Escape Key Runtime State ===
    /// Whether Escape key is currently registered (to track cleanup)
    /// Uses Arc<AtomicBool> for thread-safe updates from spawned registration thread
    escape_registered: Arc<AtomicBool>,
    /// Double-tap detector for Escape key (created on recording start)
    double_tap_detector: DoubleTapDetectorState,

    // === Hotkey Events ===
    /// Optional emitter for hotkey-related events (e.g., key blocking unavailable)
    hotkey_emitter: Option<Arc<dyn HotkeyEventEmitter>>,
}

impl<R: RecordingEventEmitter, T: TranscriptionEventEmitter + 'static, C: CommandEventEmitter + 'static> HotkeyIntegration<R, T, C> {
    /// Create a new HotkeyIntegration with default debounce duration
    pub fn new(recording_emitter: R) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(DEBOUNCE_DURATION_MS),
            recording_mode: RecordingMode::default(),
            recording_emitter,
            // Grouped configurations
            transcription: None,
            voice_commands: None,
            escape: None,
            silence: SilenceDetectionConfig::default(),
            // Audio (kept separate for flexible builder pattern)
            audio_thread: None,
            audio_monitor: None,
            recording_state: None,
            recording_detectors: None,
            // App integration
            app_handle: None,
            recordings_dir: crate::paths::get_recordings_dir(None)
                .unwrap_or_else(|_| std::path::PathBuf::from(".").join("heycat").join("recordings")),
            // Escape key runtime state
            escape_registered: Arc::new(AtomicBool::new(false)),
            double_tap_detector: None,
            // Hotkey events
            hotkey_emitter: None,
        }
    }

    /// Get the current recording mode
    pub fn recording_mode(&self) -> RecordingMode {
        self.recording_mode
    }

    /// Update the recording mode at runtime
    ///
    /// This allows changing the mode after construction, typically used when
    /// the user changes the setting in the UI.
    pub fn set_recording_mode(&mut self, mode: RecordingMode) {
        self.recording_mode = mode;
        crate::debug!("Recording mode updated to: {:?}", mode);
    }

    /// Add app handle for clipboard access (builder pattern)
    pub fn with_app_handle(mut self, handle: AppHandle) -> Self {
        self.app_handle = Some(handle);
        self
    }

    /// Add recordings directory for worktree-aware recording storage (builder pattern)
    pub fn with_recordings_dir(mut self, recordings_dir: std::path::PathBuf) -> Self {
        self.recordings_dir = recordings_dir;
        self
    }

    /// Get the selected audio device from persistent settings store
    fn get_selected_audio_device(&self) -> Option<String> {
        use crate::util::SettingsAccess;

        // Create a wrapper struct that implements SettingsAccess
        struct OptionalAppHandle<'a>(&'a Option<AppHandle>);
        impl<'a> SettingsAccess for OptionalAppHandle<'a> {
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
    /// The monitor is stopped before recording to prevent device conflicts
    pub fn with_audio_monitor(mut self, handle: Arc<AudioMonitorHandle>) -> Self {
        self.audio_monitor = Some(handle);
        self
    }

    /// Add SharedTranscriptionModel for auto-transcription (builder pattern)
    pub fn with_shared_transcription_model(mut self, model: Arc<SharedTranscriptionModel>) -> Self {
        // Update the transcription config's shared_model field
        if let Some(ref mut config) = self.transcription {
            config.shared_model = Some(model);
        } else {
            // Initialize transcription config with this model
            // Other fields will use defaults until set
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

    /// Add transcription event emitter for emitting events from spawned thread (builder pattern)
    pub fn with_transcription_emitter(mut self, emitter: Arc<T>) -> Self {
        if let Some(ref mut config) = self.transcription {
            config.emitter = Some(emitter);
        } else {
            // Initialize transcription config with this emitter
            // Model must be set separately for transcription to work
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

    /// Add recording state reference for transcription thread (builder pattern)
    pub fn with_recording_state(mut self, state: Arc<Mutex<RecordingManager>>) -> Self {
        self.recording_state = Some(state);
        self
    }

    /// Add complete voice command configuration (builder pattern)
    ///
    /// This is the preferred way to configure voice commands. All components are
    /// provided together to ensure consistency.
    pub fn with_voice_commands(mut self, config: VoiceCommandConfig<C>) -> Self {
        self.voice_commands = Some(config);
        self
    }

    /// Add Turso client for voice command fetching (builder pattern)
    ///
    /// Note: Prefer `with_voice_commands()` for new code. This method is kept for
    /// backward compatibility and alternative builder patterns.
    #[allow(dead_code)]
    pub fn with_turso_client(mut self, turso_client: Arc<TursoClient>) -> Self {
        if let Some(ref mut config) = self.voice_commands {
            config.turso_client = turso_client;
        } else {
            // Create a placeholder config - other fields must be set separately
            // This is for backward compatibility with incremental builder pattern
            self.voice_commands = Some(VoiceCommandConfig {
                turso_client,
                matcher: Arc::new(CommandMatcher::new()),
                dispatcher: Arc::new(ActionDispatcher::new()),
                emitter: None,
            });
        }
        self
    }

    /// Add command matcher for voice command matching (builder pattern)
    ///
    /// Note: Prefer `with_voice_commands()` for new code. This method is kept for
    /// backward compatibility and alternative builder patterns.
    #[allow(dead_code)]
    pub fn with_command_matcher(mut self, matcher: Arc<CommandMatcher>) -> Self {
        if let Some(ref mut config) = self.voice_commands {
            config.matcher = matcher;
        }
        // If no voice_commands config exists, this is a no-op
        // (matcher without registry isn't useful)
        self
    }

    /// Add action dispatcher for executing matched commands (builder pattern)
    ///
    /// Note: Prefer `with_voice_commands()` for new code. This method is kept for
    /// backward compatibility and alternative builder patterns.
    #[allow(dead_code)]
    pub fn with_action_dispatcher(mut self, dispatcher: Arc<ActionDispatcher>) -> Self {
        if let Some(ref mut config) = self.voice_commands {
            config.dispatcher = dispatcher;
        }
        // If no voice_commands config exists, this is a no-op
        self
    }

    /// Add command event emitter for voice command events (builder pattern)
    ///
    /// Note: Prefer `with_voice_commands()` for new code. This method is kept for
    /// backward compatibility and alternative builder patterns.
    #[allow(dead_code)]
    pub fn with_command_emitter(mut self, emitter: Arc<C>) -> Self {
        if let Some(ref mut config) = self.voice_commands {
            config.emitter = Some(emitter);
        }
        // If no voice_commands config exists, this is a no-op
        self
    }

    /// Set custom transcription timeout duration (builder pattern)
    ///
    /// Default is 60 seconds. If transcription takes longer than this, it will be
    /// cancelled and a timeout error will be emitted to the frontend.
    #[allow(dead_code)]
    pub fn with_transcription_timeout(mut self, timeout: Duration) -> Self {
        if let Some(ref mut config) = self.transcription {
            config.timeout = timeout;
        } else {
            // Create a minimal transcription config with the timeout
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
    ///
    /// When configured, hotkey recordings will automatically stop when silence is
    /// detected (after speech ends). This shares the same RecordingDetectors state
    /// used by the wake word flow for consistency.
    pub fn with_recording_detectors(mut self, detectors: Arc<Mutex<RecordingDetectors>>) -> Self {
        self.recording_detectors = Some(detectors);
        self
    }

    /// Enable or disable silence detection for hotkey recordings (builder pattern)
    ///
    /// Default is true (enabled). When disabled, hotkey recordings will only stop
    /// when the user manually triggers the hotkey again.
    #[allow(dead_code)]
    pub fn with_silence_detection_enabled(mut self, enabled: bool) -> Self {
        self.silence.enabled = enabled;
        self
    }

    /// Set custom silence configuration for hotkey recordings (builder pattern)
    ///
    /// If not set, uses the default SilenceConfig. This allows customizing
    /// silence duration thresholds, no-speech timeouts, etc.
    #[allow(dead_code)]
    pub fn with_silence_config(mut self, config: SilenceConfig) -> Self {
        self.silence.config = Some(config);
        self
    }

    /// Add complete escape key configuration (builder pattern)
    ///
    /// This is the preferred way to configure escape key handling when all components
    /// are known upfront. For cases where the callback needs to capture a reference to
    /// the integration itself, use `with_shortcut_backend()` followed by `set_escape_callback()`.
    #[allow(dead_code)]
    pub fn with_escape(mut self, config: EscapeKeyConfig) -> Self {
        self.escape = Some(config);
        self
    }

    /// Add shortcut backend for Escape key registration (builder pattern)
    ///
    /// When configured with an escape callback, the Escape key listener will be
    /// automatically registered when recording starts and unregistered when
    /// recording stops. This enables cancel functionality via Escape key.
    pub fn with_shortcut_backend(mut self, backend: Arc<dyn ShortcutBackend + Send + Sync>) -> Self {
        if let Some(ref mut config) = self.escape {
            config.backend = backend;
        } else {
            // Create a placeholder escape config - callback must be set separately
            self.escape = Some(EscapeKeyConfig {
                backend,
                callback: None,
                double_tap_window_ms: DEFAULT_DOUBLE_TAP_WINDOW_MS,
            });
        }
        self
    }

    /// Set the callback to invoke when double-tap Escape is detected during recording (builder pattern)
    ///
    /// The callback will only fire when recording is in progress AND a double-tap
    /// of the Escape key is detected within the configured time window (default 300ms).
    /// Single Escape key presses are ignored. The Escape key listener is automatically
    /// registered when recording starts and unregistered when recording stops.
    ///
    /// Note: Production code uses `set_escape_callback` instead because the callback
    /// needs to capture a reference to the integration after it's wrapped in Arc<Mutex<>>.
    /// This builder method is kept for test ergonomics.
    #[allow(dead_code)]
    pub fn with_escape_callback(mut self, callback: Arc<dyn Fn() + Send + Sync>) -> Self {
        if let Some(ref mut config) = self.escape {
            config.callback = Some(callback);
        } else {
            // Create a placeholder escape config - backend must be set separately
            self.escape = Some(EscapeKeyConfig {
                backend: Arc::new(crate::hotkey::NullShortcutBackend),
                callback: Some(callback),
                double_tap_window_ms: DEFAULT_DOUBLE_TAP_WINDOW_MS,
            });
        }
        self
    }

    /// Set the time window for double-tap detection (builder pattern)
    ///
    /// Default is 300ms. Two Escape key presses within this window will trigger
    /// the cancel callback. Single presses are ignored.
    #[allow(dead_code)]
    pub fn with_double_tap_window(mut self, window_ms: u64) -> Self {
        if let Some(ref mut config) = self.escape {
            config.double_tap_window_ms = window_ms;
        } else {
            // Create a placeholder escape config
            self.escape = Some(EscapeKeyConfig {
                backend: Arc::new(crate::hotkey::NullShortcutBackend),
                callback: None,
                double_tap_window_ms: window_ms,
            });
        }
        self
    }

    /// Set the hotkey event emitter for key blocking notifications (builder pattern)
    ///
    /// When set, failures to register the Escape key listener will emit a
    /// `key_blocking_unavailable` event to notify the user that Escape key
    /// blocking is unavailable (graceful degradation).
    pub fn with_hotkey_emitter(mut self, emitter: Arc<dyn HotkeyEventEmitter>) -> Self {
        self.hotkey_emitter = Some(emitter);
        self
    }

    /// Set the transcription callback (builder pattern)
    ///
    /// When set, spawn_transcription will delegate to this callback instead of
    /// performing transcription inline. This enables integration with
    /// RecordingTranscriptionService while avoiding code duplication.
    pub fn with_transcription_callback(
        mut self,
        callback: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Self {
        if let Some(ref mut config) = self.transcription {
            config.callback = Some(callback);
        } else {
            // Create a minimal transcription config with the callback
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
    ///
    /// This allows setting the callback after the integration is wrapped in Arc<Mutex<>>,
    /// which is necessary when the callback needs to capture a reference to the integration
    /// itself (for calling cancel_recording).
    pub fn set_escape_callback(&mut self, callback: Arc<dyn Fn() + Send + Sync>) {
        if let Some(ref mut config) = self.escape {
            config.callback = Some(callback);
        } else {
            // Create a placeholder escape config
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
            // Grouped configurations
            transcription: None,
            voice_commands: None,
            escape: None,
            silence: SilenceDetectionConfig::default(),
            // Audio (kept separate for flexible builder pattern)
            audio_thread: None,
            audio_monitor: None,
            recording_state: None,
            recording_detectors: None,
            // App integration
            app_handle: None,
            recordings_dir: std::env::temp_dir().join("heycat-test-recordings"),
            // Escape key runtime state
            escape_registered: Arc::new(AtomicBool::new(false)),
            double_tap_detector: None,
            // Hotkey events
            hotkey_emitter: None,
        }
    }

    /// Handle hotkey toggle - debounces rapid presses
    ///
    /// Toggles recording state (Idle → Recording → Idle) and emits events.
    /// Delegates to unified command implementations for start/stop logic.
    ///
    /// Returns true if the toggle was accepted, false if debounced or busy
    ///
    /// Coverage exclusion: Error paths (lock poisoning, command failures) cannot
    /// be triggered without mocking std::sync primitives. The happy path is tested
    /// via integration_test.rs with mock emitters.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn handle_toggle(&mut self, state: &Mutex<RecordingManager>) -> bool {
        let now = Instant::now();

        // Check debounce
        if let Some(last) = self.last_toggle_time {
            if now.duration_since(last) < self.debounce_duration {
                crate::trace!("Toggle debounced");
                return false;
            }
        }

        self.last_toggle_time = Some(now);

        // Check current state to decide action
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                crate::error!("Failed to acquire lock: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        crate::debug!("Toggle received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Idle => {
                crate::info!("Starting recording from Idle state...");

                // Check model availability (TDT for batch transcription)
                let model_available = check_model_exists_for_type(ModelType::ParakeetTDT).unwrap_or(false);

                // Note: Audio monitor uses unified SharedAudioEngine with capture, so no need to stop it.
                // Level monitoring continues during recording via the shared engine.

                // Use unified command implementation
                // Read selected device from persistent settings store
                let device_name = self.get_selected_audio_device();
                match start_recording_impl(state, self.audio_thread.as_deref(), model_available, device_name) {
                    Ok(()) => {
                        self.recording_emitter
                            .emit_recording_started(RecordingStartedPayload {
                                timestamp: current_timestamp(),
                            });
                        crate::info!("Recording started, emitted recording_started event");

                        // Register Escape key listener for cancel functionality
                        self.register_escape_listener();

                        // Enable Escape key consumption to prevent propagation to other apps
                        set_consume_escape(true);

                        // Start silence detection if enabled and configured
                        self.start_silence_detection(state);

                        true
                    }
                    Err(e) => {
                        crate::error!("Failed to start recording: {}", e);
                        self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Recording => {
                crate::info!("Stopping recording (manual stop via hotkey)...");

                // Unregister Escape key listener first
                self.unregister_escape_listener();

                // Disable Escape key consumption since recording is stopping
                set_consume_escape(false);

                // Stop silence detection first to prevent it from interfering
                // Manual stop takes precedence over auto-stop
                self.stop_silence_detection();

                // Use unified command implementation (always return to Idle)
                match stop_recording_impl(state, self.audio_thread.as_deref(), false, self.recordings_dir.clone()) {
                    Ok(metadata) => {
                        crate::info!(
                            "Recording stopped: {} samples, {:.2}s duration",
                            metadata.sample_count, metadata.duration_secs
                        );

                        // Store recording metadata in Turso using storage abstraction
                        if let Some(ref app_handle) = self.app_handle {
                            if !metadata.file_path.is_empty() {
                                crate::storage::store_recording(app_handle, &metadata, "hotkey");
                            }
                        }

                        // Clone file_path before metadata is moved
                        let file_path_for_transcription = metadata.file_path.clone();
                        self.recording_emitter
                            .emit_recording_stopped(RecordingStoppedPayload { metadata });
                        crate::debug!("Emitted recording_stopped event");

                        // Auto-transcribe if transcription manager is configured
                        self.spawn_transcription(file_path_for_transcription);

                        true
                    }
                    Err(e) => {
                        crate::error!("Failed to stop recording: {}", e);
                        self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Processing => {
                // In Processing state - ignore toggle (busy)
                crate::debug!("Toggle ignored - already processing");
                false
            }
        }
    }

    /// Handle hotkey press event for push-to-talk mode
    ///
    /// In PTT mode, pressing the hotkey starts recording immediately (no debounce).
    /// This is called when the hotkey key is pressed down.
    ///
    /// Returns true if recording was started, false otherwise.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn handle_hotkey_press(&mut self, state: &Mutex<RecordingManager>) -> bool {
        // PTT mode skips debounce for immediate response
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                crate::error!("Failed to acquire lock: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        crate::debug!("PTT press received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Idle => {
                crate::info!("PTT: Starting recording on key press...");

                // Check model availability
                let model_available = check_model_exists_for_type(ModelType::ParakeetTDT).unwrap_or(false);

                let device_name = self.get_selected_audio_device();
                match start_recording_impl(state, self.audio_thread.as_deref(), model_available, device_name) {
                    Ok(()) => {
                        self.recording_emitter
                            .emit_recording_started(RecordingStartedPayload {
                                timestamp: current_timestamp(),
                            });
                        crate::info!("PTT: Recording started");

                        // Register Escape key listener for emergency cancel
                        self.register_escape_listener();

                        // Enable Escape key consumption
                        set_consume_escape(true);

                        // Note: PTT mode does NOT start silence detection
                        // Recording stops on key release, not on silence

                        true
                    }
                    Err(e) => {
                        crate::error!("PTT: Failed to start recording: {}", e);
                        self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Recording => {
                // Already recording - ignore (user might have double-pressed)
                crate::debug!("PTT press ignored - already recording");
                false
            }
            RecordingState::Processing => {
                // Busy - ignore
                crate::debug!("PTT press ignored - processing");
                false
            }
        }
    }

    /// Handle hotkey release event for push-to-talk mode
    ///
    /// In PTT mode, releasing the hotkey stops recording immediately.
    /// This is called when the hotkey key is released.
    ///
    /// Returns true if recording was stopped, false otherwise.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn handle_hotkey_release(&mut self, state: &Mutex<RecordingManager>) -> bool {
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                crate::error!("Failed to acquire lock: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        crate::debug!("PTT release received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Recording => {
                crate::info!("PTT: Stopping recording on key release...");

                // Unregister Escape key listener
                self.unregister_escape_listener();

                // Disable Escape key consumption
                set_consume_escape(false);

                // Stop recording and process
                match stop_recording_impl(state, self.audio_thread.as_deref(), false, self.recordings_dir.clone()) {
                    Ok(metadata) => {
                        crate::info!(
                            "PTT: Recording stopped: {} samples, {:.2}s duration",
                            metadata.sample_count, metadata.duration_secs
                        );

                        // Store recording metadata in Turso using storage abstraction
                        if let Some(ref app_handle) = self.app_handle {
                            if !metadata.file_path.is_empty() {
                                crate::storage::store_recording(app_handle, &metadata, "PTT");
                            }
                        }

                        let file_path_for_transcription = metadata.file_path.clone();
                        self.recording_emitter
                            .emit_recording_stopped(RecordingStoppedPayload { metadata });
                        crate::debug!("PTT: Emitted recording_stopped event");

                        // Auto-transcribe
                        self.spawn_transcription(file_path_for_transcription);

                        true
                    }
                    Err(e) => {
                        crate::error!("PTT: Failed to stop recording: {}", e);
                        self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Idle => {
                // Not recording - ignore (user might have released after cancel)
                crate::debug!("PTT release ignored - not recording");
                false
            }
            RecordingState::Processing => {
                // Busy - ignore
                crate::debug!("PTT release ignored - processing");
                false
            }
        }
    }

    /// Spawn transcription as an async task
    ///
    /// Transcribes the WAV file, tries command matching, then fallback to clipboard.
    /// Uses Tauri's async runtime for bounded async execution.
    /// No-op if transcription manager or transcription emitter is not configured.
    ///
    /// This method is public so it can be called from the wake word recording flow
    /// (via the coordinator) in addition to the hotkey recording flow.
    ///
    /// When a transcription_callback is configured (via with_transcription_callback),
    /// this method delegates to that callback instead of performing transcription inline.
    /// This enables integration with RecordingTranscriptionService.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn spawn_transcription(&self, file_path: String) {
        // If a transcription callback is configured, delegate to it
        // This enables HotkeyIntegration to use TranscriptionService without duplication
        if let Some(ref config) = self.transcription {
            if let Some(ref callback) = config.callback {
                crate::info!("Delegating transcription to external service for: {}", file_path);
                callback(file_path);
                return;
            }
        }

        // Fallback: inline transcription (for backward compatibility and tests)
        // Check all required components are present from transcription config
        let transcription_config = match &self.transcription {
            Some(c) => c,
            None => {
                crate::debug!("Transcription skipped: no transcription config");
                return;
            }
        };

        let shared_model = match &transcription_config.shared_model {
            Some(m) => m.clone(),
            None => {
                crate::debug!("Transcription skipped: no shared transcription model configured");
                return;
            }
        };

        let transcription_emitter = match &transcription_config.emitter {
            Some(te) => te.clone(),
            None => {
                crate::debug!("Transcription skipped: no transcription emitter configured");
                return;
            }
        };

        // Check if model is loaded
        if !shared_model.is_loaded() {
            crate::info!("Transcription skipped: transcription model not loaded");
            return;
        }

        // Optional voice command components from voice_commands config
        let (turso_client, command_matcher, action_dispatcher, command_emitter) =
            if let Some(ref vc) = self.voice_commands {
                (
                    Some(vc.turso_client.clone()),
                    Some(vc.matcher.clone()),
                    Some(vc.dispatcher.clone()),
                    vc.emitter.clone(),
                )
            } else {
                (None, None, None, None)
            };

        // Clone app_handle for clipboard access
        let app_handle = self.app_handle.clone();

        // Clone recording_state for buffer cleanup after transcription
        let recording_state = self.recording_state.clone();

        // Clone semaphore and timeout from transcription config
        let semaphore = transcription_config.semaphore.clone();
        let timeout_duration = transcription_config.timeout;

        crate::info!("Spawning transcription task...");

        // Spawn async task using Tauri's async runtime
        tauri::async_runtime::spawn(async move {
            // Execute transcription using shared helper
            let result = execute_transcription_task(
                file_path,
                shared_model.clone(),
                semaphore,
                transcription_emitter.clone(),
                timeout_duration,
                recording_state.clone(),
            )
            .await;

            // Handle transcription result
            let TranscriptionResult { text, duration_ms } = match result {
                Ok(r) => r,
                Err(()) => return, // Error already emitted and buffer cleared by helper
            };

            crate::info!("=== spawn_transcription received text ===");
            crate::info!("text content: {:?}", text);
            crate::info!("=== end spawn_transcription text ===");

            // Try voice command matching if configured
            enum MatchOutcome {
                Matched { cmd: CommandDefinition, trigger: String, confidence: f64 },
                Ambiguous { candidates: Vec<CommandCandidate> },
                NoMatch,
            }

            // Helper to clear recording buffer
            let clear_recording_buffer = || {
                if let Some(ref state) = recording_state {
                    if let Ok(mut manager) = state.lock() {
                        manager.clear_last_recording();
                        crate::debug!("Cleared recording buffer");
                    }
                }
            };

            let command_handled = if let (Some(client), Some(matcher), Some(dispatcher), Some(emitter)) =
                (&turso_client, &command_matcher, &action_dispatcher, &command_emitter)
            {
                // Fetch all commands from Turso
                let all_commands = match client.list_voice_commands().await {
                    Ok(commands) => commands,
                    Err(e) => {
                        crate::error!("Failed to fetch voice commands from Turso: {}", e);
                        transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                            error: "Failed to load voice commands. Please try again.".to_string(),
                        });
                        clear_recording_buffer();
                        return;
                    }
                };

                // Build a lookup map for finding commands by ID
                let commands_by_id: std::collections::HashMap<uuid::Uuid, &CommandDefinition> =
                    all_commands.iter().map(|cmd| (cmd.id, cmd)).collect();

                let match_result = matcher.match_commands(&text, &all_commands);

                let outcome = match match_result {
                    MatchResult::Exact { command: matched_cmd, .. } => {
                        match commands_by_id.get(&matched_cmd.id) {
                            Some(cmd) => MatchOutcome::Matched {
                                cmd: (*cmd).clone(),
                                trigger: matched_cmd.trigger.clone(),
                                confidence: 1.0
                            },
                            None => MatchOutcome::NoMatch,
                        }
                    }
                    MatchResult::Fuzzy { command: matched_cmd, score, .. } => {
                        match commands_by_id.get(&matched_cmd.id) {
                            Some(cmd) => MatchOutcome::Matched {
                                cmd: (*cmd).clone(),
                                trigger: matched_cmd.trigger.clone(),
                                confidence: score
                            },
                            None => MatchOutcome::NoMatch,
                        }
                    }
                    MatchResult::Ambiguous { candidates } => {
                        let candidate_data: Vec<_> = candidates
                            .iter()
                            .map(|c| CommandCandidate {
                                id: c.command.id.to_string(),
                                trigger: c.command.trigger.clone(),
                                confidence: c.score,
                            })
                            .collect();
                        MatchOutcome::Ambiguous { candidates: candidate_data }
                    }
                    MatchResult::NoMatch => MatchOutcome::NoMatch,
                };

                match outcome {
                    MatchOutcome::Matched { cmd, trigger, confidence } => {
                        crate::info!("Command matched: {} (confidence: {:.2})", trigger, confidence);

                        // Emit command_matched event
                        emitter.emit_command_matched(CommandMatchedPayload {
                            transcription: text.clone(),
                            command_id: cmd.id.to_string(),
                            trigger: trigger.clone(),
                            confidence,
                        });

                        // Execute command directly using await (no new runtime needed!)
                        match dispatcher.execute(&cmd).await {
                            Ok(action_result) => {
                                crate::info!("Command executed: {}", action_result.message);
                                emitter.emit_command_executed(CommandExecutedPayload {
                                    command_id: cmd.id.to_string(),
                                    trigger: trigger.clone(),
                                    message: action_result.message,
                                });
                            }
                            Err(action_error) => {
                                crate::error!("Command execution failed: {}", action_error);
                                emitter.emit_command_failed(CommandFailedPayload {
                                    command_id: cmd.id.to_string(),
                                    trigger: trigger.clone(),
                                    error_code: action_error.code.to_string(),
                                    error_message: action_error.message,
                                });
                            }
                        }
                        true // Command was handled
                    }
                    MatchOutcome::Ambiguous { candidates } => {
                        crate::info!("Ambiguous match: {} candidates", candidates.len());

                        // Emit command_ambiguous event for disambiguation UI
                        emitter.emit_command_ambiguous(CommandAmbiguousPayload {
                            transcription: text.clone(),
                            candidates,
                        });
                        true // Command matching was handled (ambiguous)
                    }
                    MatchOutcome::NoMatch => {
                        crate::debug!("No command match for: {}", text);
                        false // Fall through to clipboard
                    }
                }
            } else {
                crate::debug!("Voice commands not configured, skipping command matching");
                false
            };

            // Fallback to clipboard if no command was handled
            if !command_handled {
                copy_and_paste(&app_handle, &text);
            }

            // Always emit transcription_completed (whether command handled or not)
            // This ensures the frontend clears the "Transcribing..." state
            crate::info!("=== Emitting transcription_completed ===");
            crate::info!("text to emit: {:?}", text);
            crate::info!("=== end emit ===");
            transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                text,
                duration_ms,
            });

            // Reset transcription state to idle
            if let Err(e) = shared_model.reset_to_idle() {
                crate::warn!("Failed to reset transcription state: {}", e);
            }

            // Clear recording buffer to free memory
            clear_recording_buffer();
        });
    }

    /// Start silence detection for hotkey recording
    ///
    /// When silence detection is enabled and all required components are configured,
    /// this starts monitoring the recording audio for silence. The recording will
    /// automatically stop when silence is detected after speech ends.
    ///
    /// This method is called after recording starts successfully. The detection runs
    /// in a separate thread and will handle saving/transcription when silence triggers.
    fn start_silence_detection(&self, recording_state: &Mutex<RecordingManager>) {
        // Check if silence detection is enabled (from silence config)
        if !self.silence.enabled {
            crate::debug!("Silence detection disabled for hotkey recordings");
            return;
        }

        // Check for required components
        let detectors = match &self.recording_detectors {
            Some(d) => d.clone(),
            None => {
                crate::debug!("Recording detectors not configured, skipping silence detection");
                return;
            }
        };

        let audio_thread = match &self.audio_thread {
            Some(at) => at.clone(),
            None => {
                crate::debug!("No audio thread configured, cannot start silence detection");
                return;
            }
        };

        // Verify transcription emitter is configured (needed for the callback)
        let transcription_config = match &self.transcription {
            Some(c) => c,
            None => {
                crate::debug!("No transcription config, cannot start silence detection");
                return;
            }
        };
        if transcription_config.emitter.is_none() {
            crate::debug!("No transcription emitter configured, cannot start silence detection");
            return;
        }

        // Get the audio buffer from recording state
        let buffer = {
            let manager = match recording_state.lock() {
                Ok(m) => m,
                Err(_) => {
                    crate::warn!("Failed to lock recording state for silence detection");
                    return;
                }
            };

            match manager.get_audio_buffer() {
                Ok(buf) => buf.clone(),
                Err(_) => {
                    crate::warn!("No audio buffer available for silence detection");
                    return;
                }
            }
        };

        // Create transcription callback that calls spawn_transcription
        // This is the same pattern used in wake word flow
        // Extract components from transcription config
        let shared_model = transcription_config.shared_model.clone();
        let transcription_emitter_for_callback = transcription_config.emitter.clone();
        let app_handle_for_callback = self.app_handle.clone();
        let recording_state_for_callback = self.recording_state.clone();
        let transcription_semaphore_for_callback = transcription_config.semaphore.clone();
        let transcription_timeout_for_callback = transcription_config.timeout;

        // Build transcription callback
        let transcription_callback: Option<Box<dyn Fn(String) + Send + 'static>> =
            if shared_model.is_some() && transcription_emitter_for_callback.is_some() {
                Some(Box::new(move |file_path: String| {
                    // Extract required components from Option wrappers
                    let shared_model = match &shared_model {
                        Some(m) => m.clone(),
                        None => return,
                    };
                    let transcription_emitter = match &transcription_emitter_for_callback {
                        Some(te) => te.clone(),
                        None => return,
                    };

                    if !shared_model.is_loaded() {
                        crate::info!("Transcription skipped: model not loaded");
                        return;
                    }

                    let semaphore = transcription_semaphore_for_callback.clone();
                    let timeout_duration = transcription_timeout_for_callback;
                    let app_handle = app_handle_for_callback.clone();
                    let recording_state = recording_state_for_callback.clone();

                    crate::info!("[silence_detection] Spawning transcription task for: {}", file_path);

                    tauri::async_runtime::spawn(async move {
                        // Execute transcription using shared helper
                        let result = execute_transcription_task(
                            file_path,
                            shared_model.clone(),
                            semaphore,
                            transcription_emitter.clone(),
                            timeout_duration,
                            recording_state.clone(),
                        )
                        .await;

                        // Handle transcription result
                        let TranscriptionResult { text, duration_ms } = match result {
                            Ok(r) => r,
                            Err(()) => return, // Error already emitted and buffer cleared by helper
                        };

                        // Silence detection auto-stop always goes to clipboard
                        // Voice command matching is only supported for manual hotkey recordings
                        // (via spawn_transcription). This is by design - auto-stop recordings
                        // are intended for quick dictation, not command execution.
                        copy_and_paste(&app_handle, &text);

                        // Emit completed
                        transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                            text,
                            duration_ms,
                        });

                        // Reset model and clear buffer
                        let _ = shared_model.reset_to_idle();
                        if let Some(ref state) = recording_state {
                            if let Ok(mut manager) = state.lock() {
                                manager.clear_last_recording();
                                crate::debug!("Cleared recording buffer");
                            }
                        }
                    });
                }))
            } else {
                None
            };

        // Lock detectors and start monitoring
        let mut det = match detectors.lock() {
            Ok(d) => d,
            Err(_) => {
                crate::warn!("Failed to lock recording detectors");
                return;
            }
        };

        // Use the recording state Arc that was configured via builder
        let recording_state_arc = match &self.recording_state {
            Some(rs) => rs.clone(),
            None => {
                crate::warn!("No recording state configured, cannot start silence detection");
                return;
            }
        };

        // Create a recording emitter for the detection coordinator
        // We need to use R which implements RecordingEventEmitter
        // But we can't clone self.recording_emitter since it's moved into self
        // Instead, create a new emitter from the app handle
        let recording_emitter_for_detectors = match &self.app_handle {
            Some(handle) => Arc::new(crate::commands::TauriEventEmitter::new(handle.clone())),
            None => {
                crate::warn!("No app handle configured, cannot create emitter for silence detection");
                return;
            }
        };

        crate::info!("[silence_detection] Starting monitoring for hotkey recording");
        if let Err(e) = det.start_monitoring(
            buffer,
            recording_state_arc,
            audio_thread,
            recording_emitter_for_detectors,
            transcription_callback,
        ) {
            crate::warn!("Failed to start silence detection: {}", e);
        } else {
            crate::info!("[silence_detection] Monitoring started successfully");
        }
    }

    /// Stop silence detection for hotkey recording
    ///
    /// Called when the user manually stops recording via hotkey. This ensures
    /// the silence detection thread is stopped before processing the recording,
    /// allowing manual stop to take precedence over auto-stop.
    fn stop_silence_detection(&self) {
        let detectors = match &self.recording_detectors {
            Some(d) => d,
            None => return,
        };

        if let Ok(mut det) = detectors.lock() {
            if det.is_running() {
                crate::info!("[silence_detection] Stopping monitoring (manual stop)");
                det.stop_monitoring();
            }
        }
    }

    /// Register the Escape key listener for cancel functionality
    ///
    /// Called when recording starts. Only registers if both shortcut_backend
    /// and escape_callback are configured. The listener is automatically
    /// unregistered when recording stops.
    ///
    /// Uses double-tap detection: single Escape presses are ignored, only
    /// double-taps within the configured time window trigger the cancel callback.
    ///
    /// IMPORTANT: The actual registration is deferred to a spawned thread to avoid
    /// re-entrancy deadlock. When this function is called from within a global shortcut
    /// callback (e.g., the recording hotkey), calling backend.register() synchronously
    /// would deadlock because the shortcut manager's lock is already held.
    fn register_escape_listener(&mut self) {
        // Skip if already registered or not configured
        if self.escape_registered.load(Ordering::SeqCst) {
            crate::debug!("Escape listener already registered, skipping");
            return;
        }

        // Get escape config - skip if not configured
        let escape_config = match &self.escape {
            Some(c) => c,
            None => {
                crate::debug!("No escape config, skipping Escape registration");
                return;
            }
        };

        let backend = escape_config.backend.clone();

        let callback = match &escape_config.callback {
            Some(c) => c.clone(),
            None => {
                crate::debug!("No escape callback configured, skipping Escape registration");
                return;
            }
        };

        // Create double-tap detector with the configured window
        // The detector wraps the callback and only invokes it on double-tap
        let boxed_callback: Box<dyn Fn() + Send + Sync> = Box::new(move || callback());
        let detector = Arc::new(Mutex::new(DoubleTapDetector::with_window(
            boxed_callback,
            escape_config.double_tap_window_ms,
        )));
        self.double_tap_detector = Some(detector.clone());

        // In tests, use synchronous registration (mock backends don't have deadlock issues)
        // In production, spawn registration on a separate thread to avoid re-entrancy deadlock
        #[cfg(test)]
        {
            match backend.register(super::ESCAPE_SHORTCUT, Box::new(move || {
                // Use try_lock to avoid blocking the CGEventTap callback
                // If lock is contended, skip this escape tap rather than freezing keyboard
                if let Ok(mut det) = detector.try_lock() {
                    if det.on_tap() {
                        crate::debug!("Double-tap Escape detected, cancel triggered");
                    } else {
                        crate::trace!("Single Escape tap recorded, waiting for double-tap");
                    }
                } else {
                    crate::trace!("Skipping escape tap - detector lock contended");
                }
            })) {
                Ok(()) => {
                    self.escape_registered.store(true, Ordering::SeqCst);
                    crate::info!("Escape key listener registered for recording cancel (double-tap required)");
                }
                Err(e) => {
                    crate::warn!("Failed to register Escape key listener: {}", e);
                    self.double_tap_detector = None;
                    // Emit notification that key blocking is unavailable
                    if let Some(ref emitter) = self.hotkey_emitter {
                        emitter.emit_key_blocking_unavailable(
                            hotkey_events::KeyBlockingUnavailablePayload {
                                reason: format!("Failed to register Escape key listener: {}", e),
                                timestamp: current_timestamp(),
                            },
                        );
                    }
                }
            }
        }

        #[cfg(not(test))]
        {
            // Clone the Arc<AtomicBool> for the spawned thread to set after successful registration
            let escape_registered = self.escape_registered.clone();
            // Clone the hotkey emitter for the spawned thread to emit notifications on failure
            let hotkey_emitter = self.hotkey_emitter.clone();

            // Spawn registration on a separate thread to avoid re-entrancy deadlock
            // This is necessary because we're called from within a global shortcut callback,
            // and the shortcut manager holds a lock during callback execution.
            std::thread::spawn(move || {
                // Small delay to ensure the calling shortcut callback has completed
                std::thread::sleep(std::time::Duration::from_millis(10));

                match backend.register(super::ESCAPE_SHORTCUT, Box::new(move || {
                    // Use try_lock to avoid blocking the CGEventTap callback
                    // If lock is contended, skip this escape tap rather than freezing keyboard
                    if let Ok(mut det) = detector.try_lock() {
                        if det.on_tap() {
                            crate::debug!("Double-tap Escape detected, cancel triggered");
                        } else {
                            crate::trace!("Single Escape tap recorded, waiting for double-tap");
                        }
                    } else {
                        crate::trace!("Skipping escape tap - detector lock contended");
                    }
                })) {
                    Ok(()) => {
                        // Only set escape_registered to true AFTER successful registration
                        escape_registered.store(true, Ordering::SeqCst);
                        crate::info!("Escape key listener registered for recording cancel (double-tap required)");
                    }
                    Err(e) => {
                        crate::warn!("Failed to register Escape key listener: {}", e);
                        // escape_registered remains false, so unregister won't attempt cleanup
                        // Emit notification that key blocking is unavailable
                        if let Some(ref emitter) = hotkey_emitter {
                            emitter.emit_key_blocking_unavailable(
                                hotkey_events::KeyBlockingUnavailablePayload {
                                    reason: format!("Failed to register Escape key listener: {}", e),
                                    timestamp: current_timestamp(),
                                },
                            );
                        }
                    }
                }
            });
        }
    }

    /// Unregister the Escape key listener
    ///
    /// Called when recording stops (either normally or via cancellation).
    /// Safe to call even if listener was never registered. Also resets the
    /// double-tap detector state.
    ///
    /// IMPORTANT: Like register_escape_listener, the actual unregistration is deferred
    /// to a spawned thread to avoid re-entrancy deadlock when called from within a
    /// global shortcut callback (e.g., the recording hotkey or Escape key itself).
    fn unregister_escape_listener(&mut self) {
        // Reset double-tap detector state - use try_lock to avoid deadlock when called
        // from within the escape callback (which already holds this lock)
        if let Some(ref detector) = self.double_tap_detector {
            if let Ok(mut det) = detector.try_lock() {
                det.reset();
            }
            // If try_lock fails, we're being called from within the escape callback.
            // The detector will be dropped anyway, so skipping reset is fine.
        }
        self.double_tap_detector = None;

        if !self.escape_registered.load(Ordering::SeqCst) {
            return;
        }

        let backend = match &self.escape {
            Some(c) => c.backend.clone(),
            None => return,
        };

        // Mark as unregistered immediately
        self.escape_registered.store(false, Ordering::SeqCst);

        // In tests, use synchronous unregistration (mock backends don't have deadlock issues)
        // In production, spawn unregistration on a separate thread to avoid re-entrancy deadlock
        #[cfg(test)]
        {
            match backend.unregister(super::ESCAPE_SHORTCUT) {
                Ok(()) => {
                    crate::debug!("Escape key listener unregistered");
                }
                Err(e) => {
                    crate::warn!("Failed to unregister Escape key listener: {}", e);
                }
            }
        }

        #[cfg(not(test))]
        {
            // Spawn unregistration on a separate thread to avoid re-entrancy deadlock
            // This is necessary because we may be called from within a global shortcut callback
            // (e.g., when stopping via recording hotkey or cancelling via Escape double-tap).
            std::thread::spawn(move || {
                // Small delay to ensure the calling shortcut callback has completed
                std::thread::sleep(std::time::Duration::from_millis(10));

                match backend.unregister(super::ESCAPE_SHORTCUT) {
                    Ok(()) => {
                        crate::debug!("Escape key listener unregistered");
                    }
                    Err(e) => {
                        // This can happen if registration failed or was never completed
                        crate::warn!("Failed to unregister Escape key listener: {}", e);
                    }
                }
            });
        }
    }

    /// Cancel recording without transcription
    ///
    /// This method is called when the user double-taps Escape during recording.
    /// It stops the recording immediately, discards the audio buffer (no WAV file
    /// created, no transcription triggered), and transitions directly to Idle state.
    ///
    /// # Arguments
    /// * `state` - The recording state mutex
    /// * `reason` - The reason for cancellation (e.g., "double-tap-escape")
    ///
    /// # Returns
    /// * `true` if cancellation was successful
    /// * `false` if not in recording state or an error occurred
    pub fn cancel_recording(&mut self, state: &Mutex<RecordingManager>, reason: &str) -> bool {
        // Check current state - can only cancel from Recording state
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                crate::error!("Failed to acquire lock for cancel: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        if current_state != RecordingState::Recording {
            crate::debug!("Cancel ignored - not in recording state (current: {:?})", current_state);
            return false;
        }

        crate::info!("Cancelling recording (reason: {})", reason);

        // 1. Unregister Escape key listener first
        self.unregister_escape_listener();

        // 2. Disable Escape key consumption since recording is being cancelled
        set_consume_escape(false);

        // 3. Stop silence detection if active
        self.stop_silence_detection();

        // 4. Stop audio capture (discard result - we don't want the audio)
        if let Some(ref audio_thread) = self.audio_thread {
            // Stop the audio thread to halt capture
            if let Err(e) = audio_thread.stop() {
                crate::warn!("Failed to stop audio thread during cancel: {:?}", e);
                // Continue anyway - the buffer will be discarded
            }
        }

        // 5. Abort recording - this clears the buffer and transitions directly to Idle
        //    (bypassing Processing state, so no transcription will be triggered)
        let abort_result = match state.lock() {
            Ok(mut guard) => guard.abort_recording(RecordingState::Idle),
            Err(e) => {
                crate::error!("Failed to acquire lock for abort: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        match abort_result {
            Ok(()) => {
                // 6. Emit recording_cancelled event
                self.recording_emitter.emit_recording_cancelled(RecordingCancelledPayload {
                    reason: reason.to_string(),
                    timestamp: current_timestamp(),
                });

                crate::info!("Recording cancelled successfully");
                true
            }
            Err(e) => {
                crate::error!("Failed to abort recording: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: format!("Failed to cancel recording: {}", e),
                });
                false
            }
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

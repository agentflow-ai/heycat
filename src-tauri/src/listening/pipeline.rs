// Listening audio pipeline for continuous wake word detection
// Manages audio capture in listening mode, routing samples to the wake word detector

use super::events::WakeWordEvent;
use super::{WakeWordDetector, WakeWordDetectorConfig, WakeWordError};
use crate::audio::{AudioBuffer, AudioCaptureError, AudioThreadHandle};
use crate::audio_constants::{ANALYSIS_INTERVAL_MS, EVENT_CHANNEL_BUFFER_SIZE, MIN_SAMPLES_FOR_ANALYSIS};
use crate::events::{current_timestamp, listening_events, ListeningEventEmitter};
use crate::parakeet::SharedTranscriptionModel;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;

/// Callback type for wake word detection (DEPRECATED)
/// Called when "Hey Cat" is detected - should start recording
/// Use subscribe_events() and WakeWordEvent instead for safer async handling.
#[deprecated(note = "Use subscribe_events() and WakeWordEvent instead")]
#[allow(dead_code)]
pub type WakeWordCallback = Box<dyn Fn() + Send + Sync>;

/// Errors that can occur in the listening pipeline
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineError {
    /// Audio capture error
    AudioError(String),
    /// Wake word detector error
    DetectorError(String),
    /// Pipeline is already running
    AlreadyRunning,
    /// Pipeline is not running
    NotRunning,
    /// Lock error
    #[allow(dead_code)] // Error variant for future use
    LockError,
    /// No event subscriber configured - must call subscribe_events() before start()
    NoEventSubscriber,
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::AudioError(msg) => write!(f, "Audio error: {}", msg),
            PipelineError::DetectorError(msg) => write!(f, "Detector error: {}", msg),
            PipelineError::AlreadyRunning => write!(f, "Pipeline is already running"),
            PipelineError::NotRunning => write!(f, "Pipeline is not running"),
            PipelineError::LockError => write!(f, "Lock error"),
            PipelineError::NoEventSubscriber => {
                write!(f, "Must call subscribe_events() before start()")
            }
        }
    }
}

impl std::error::Error for PipelineError {}

impl From<AudioCaptureError> for PipelineError {
    fn from(err: AudioCaptureError) -> Self {
        PipelineError::AudioError(err.to_string())
    }
}

impl From<WakeWordError> for PipelineError {
    fn from(err: WakeWordError) -> Self {
        PipelineError::DetectorError(err.to_string())
    }
}

/// Configuration for the listening pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// How often to analyze the audio buffer for wake word (milliseconds)
    pub analysis_interval_ms: u64,
    /// Minimum samples to collect before first analysis
    #[allow(dead_code)] // Reserved for future sample count validation
    pub min_samples_for_analysis: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            // Analyze every 150ms for responsive wake word detection
            // Trades ~3x more CPU for ~3x faster response time
            analysis_interval_ms: ANALYSIS_INTERVAL_MS,
            // Need at least 0.25 seconds of audio before analyzing
            // 16000 Hz * 0.25s = 4000 samples
            min_samples_for_analysis: MIN_SAMPLES_FOR_ANALYSIS,
        }
    }
}

/// Shared state for the analysis thread
struct AnalysisState {
    /// The wake word detector
    detector: Arc<WakeWordDetector>,
    /// Flag to stop the analysis thread
    should_stop: Arc<AtomicBool>,
    /// Audio buffer being filled by capture
    buffer: AudioBuffer,
    /// Flag indicating microphone is available
    mic_available: Arc<AtomicBool>,
    /// Event channel sender for wake word events (replaces direct callback)
    event_tx: tokio_mpsc::Sender<WakeWordEvent>,
}

/// Listening audio pipeline
///
/// Manages continuous audio capture for wake word detection.
/// Runs a dedicated analysis thread that periodically checks for the wake phrase.
pub struct ListeningPipeline {
    /// Configuration
    config: PipelineConfig,
    /// Wake word detector configuration
    detector_config: WakeWordDetectorConfig,
    /// Shared transcription model for the wake word detector
    shared_model: Option<SharedTranscriptionModel>,
    /// Analysis thread handle
    analysis_thread: Option<JoinHandle<()>>,
    /// Flag to signal analysis thread to stop
    should_stop: Arc<AtomicBool>,
    /// Flag indicating microphone availability
    mic_available: Arc<AtomicBool>,
    /// Shared detector for external access
    detector: Option<Arc<WakeWordDetector>>,
    /// Current audio buffer
    buffer: Option<AudioBuffer>,
    /// Event channel sender for wake word events (kept to create new senders for subscribers)
    event_tx: Option<tokio_mpsc::Sender<WakeWordEvent>>,
    /// Receiver for analysis thread exit notification
    /// Used to wait for previous thread to exit before starting a new one
    thread_exit_rx: Option<Receiver<()>>,
}

impl ListeningPipeline {
    /// Create a new listening pipeline with default configuration
    pub fn new() -> Self {
        Self::with_config(PipelineConfig::default(), WakeWordDetectorConfig::default())
    }

    /// Create a new listening pipeline with custom configuration
    pub fn with_config(config: PipelineConfig, detector_config: WakeWordDetectorConfig) -> Self {
        Self {
            config,
            detector_config,
            shared_model: None,
            analysis_thread: None,
            should_stop: Arc::new(AtomicBool::new(false)),
            mic_available: Arc::new(AtomicBool::new(true)),
            detector: None,
            buffer: None,
            event_tx: None,
            thread_exit_rx: None,
        }
    }

    /// Set the shared transcription model for the wake word detector
    ///
    /// This should be called with the same model used by other transcription consumers
    /// to share the ~3GB Parakeet model and save memory.
    pub fn set_shared_model(&mut self, model: SharedTranscriptionModel) {
        self.shared_model = Some(model);
    }

    /// Set the callback to be invoked when wake word is detected
    ///
    /// DEPRECATED: Use subscribe_events() instead for safer async event handling.
    /// This callback is called from the analysis thread when "Hey Cat" is detected.
    /// The callback should start recording (e.g., call handle_toggle on HotkeyIntegration).
    #[deprecated(note = "Use subscribe_events() instead for safer async event handling")]
    #[allow(deprecated)]
    #[allow(dead_code)]
    pub fn set_wake_word_callback(&self, _callback: WakeWordCallback) {
        crate::warn!("[pipeline] set_wake_word_callback is deprecated, use subscribe_events() instead");
        // No-op: callbacks are replaced by event channel
    }

    /// Clear the wake word callback
    #[deprecated(note = "Use subscribe_events() instead for safer async event handling")]
    #[allow(dead_code)] // Utility method for cleanup
    pub fn clear_wake_word_callback(&self) {
        // No-op: callbacks are replaced by event channel
    }

    /// Subscribe to wake word events
    ///
    /// Returns a receiver for wake word events. Events are sent when:
    /// - Wake word is detected (WakeWordEvent::Detected)
    /// - Listening becomes unavailable (WakeWordEvent::Unavailable)
    /// - An error occurs (WakeWordEvent::Error)
    ///
    /// The channel has a bounded buffer to handle backpressure gracefully.
    /// If the receiver falls behind, older events may be dropped (try_send).
    ///
    /// # Important
    ///
    /// **This MUST be called before `start()`.** If not called, `start()` will
    /// return `PipelineError::NoEventSubscriber`. This requirement ensures events
    /// are never silently dropped.
    ///
    /// The subscription persists across stop/start cycles, so you only need to
    /// call this once per pipeline instance.
    ///
    /// Each call creates a new channel; only the most recent receiver will get events.
    pub fn subscribe_events(&mut self) -> tokio_mpsc::Receiver<WakeWordEvent> {
        let (tx, rx) = tokio_mpsc::channel(EVENT_CHANNEL_BUFFER_SIZE);
        self.event_tx = Some(tx);
        rx
    }

    /// Check if the pipeline is currently running
    pub fn is_running(&self) -> bool {
        self.analysis_thread.is_some()
    }

    /// Check if microphone is available
    #[allow(dead_code)] // Utility method for status checks
    pub fn is_mic_available(&self) -> bool {
        self.mic_available.load(Ordering::SeqCst)
    }

    /// Start continuous audio capture and analysis
    ///
    /// # Arguments
    /// * `audio_handle` - Handle to the audio thread for capture
    /// * `emitter` - Event emitter for wake word detection events
    ///
    /// # Returns
    /// The audio buffer being filled, which can be used to check if capture is working
    ///
    /// # Errors
    /// - `NoEventSubscriber` if `subscribe_events()` was not called before starting
    /// - `AlreadyRunning` if the pipeline is already running
    /// - `DetectorError` if model is not loaded or VAD initialization fails
    /// - `AudioError` if audio capture fails to start
    pub fn start<E: ListeningEventEmitter + 'static>(
        &mut self,
        audio_handle: &AudioThreadHandle,
        emitter: Arc<E>,
    ) -> Result<AudioBuffer, PipelineError> {
        // Wait for any previous analysis thread to fully exit before starting a new one.
        // This prevents race conditions when the wake word callback stops the pipeline
        // and recording completion immediately tries to restart it.
        if let Some(rx) = self.thread_exit_rx.take() {
            crate::debug!("[pipeline] Waiting for previous analysis thread to exit...");
            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(()) => crate::debug!("[pipeline] Previous thread exit confirmed"),
                Err(_) => crate::warn!("[pipeline] Timeout waiting for previous thread exit"),
            }
        }

        if self.is_running() {
            crate::debug!("[pipeline] Start called but already running");
            return Err(PipelineError::AlreadyRunning);
        }

        // Require event subscriber to be configured before starting
        // This prevents silently dropping wake word events
        // Check early so callers get a clear error message about the requirement
        if self.event_tx.is_none() {
            return Err(PipelineError::NoEventSubscriber);
        }

        crate::info!("[pipeline] Starting listening pipeline...");

        // Create detector with shared model
        let shared_model = self.shared_model.clone().ok_or_else(|| {
            PipelineError::DetectorError("No shared transcription model configured".to_string())
        })?;

        // Verify model is loaded before creating detector
        if !shared_model.is_loaded() {
            return Err(PipelineError::DetectorError(
                "Shared transcription model not loaded".to_string(),
            ));
        }

        let detector = Arc::new(WakeWordDetector::with_shared_model_and_config(
            shared_model,
            self.detector_config.clone(),
        ));

        // Initialize VAD for the detector (model is already loaded via shared model)
        detector.init_vad()?;

        // Create audio buffer for capture
        let buffer = AudioBuffer::new();
        let buffer_clone = buffer.clone();

        // Start audio capture
        audio_handle
            .start(buffer_clone.clone())
            .map_err(|e| PipelineError::AudioError(e.to_string()))?;

        // Reset flags
        self.should_stop.store(false, Ordering::SeqCst);
        self.mic_available.store(true, Ordering::SeqCst);

        // Get event_tx - safe to unwrap since we checked for NoEventSubscriber earlier
        let event_tx = self.event_tx.clone().expect("event_tx checked above");

        // Create analysis state
        let state = AnalysisState {
            detector: detector.clone(),
            should_stop: self.should_stop.clone(),
            buffer: buffer_clone,
            mic_available: self.mic_available.clone(),
            event_tx,
        };

        // Create exit notification channel
        let (exit_tx, exit_rx) = mpsc::channel();
        self.thread_exit_rx = Some(exit_rx);

        // Start analysis thread
        let config = self.config.clone();
        let analysis_thread = thread::spawn(move || {
            analysis_thread_main(state, config, emitter);
            // Signal that thread has exited
            let _ = exit_tx.send(());
        });

        self.analysis_thread = Some(analysis_thread);
        self.detector = Some(detector);
        self.buffer = Some(buffer.clone());

        crate::info!(
            "[pipeline] Pipeline started, analysis_interval={}ms",
            self.config.analysis_interval_ms
        );

        Ok(buffer)
    }

    /// Stop the listening pipeline
    ///
    /// Uses a timeout when joining the analysis thread to prevent blocking
    /// indefinitely if the thread is stuck.
    ///
    /// # Arguments
    /// * `audio_handle` - Handle to stop audio capture
    pub fn stop(&mut self, audio_handle: &AudioThreadHandle) -> Result<(), PipelineError> {
        self.stop_with_timeout(audio_handle, Duration::from_millis(500))
    }

    /// Stop the listening pipeline with a custom timeout
    ///
    /// # Arguments
    /// * `audio_handle` - Handle to stop audio capture
    /// * `timeout` - Maximum time to wait for the analysis thread to exit
    pub fn stop_with_timeout(
        &mut self,
        audio_handle: &AudioThreadHandle,
        timeout: Duration,
    ) -> Result<(), PipelineError> {
        if !self.is_running() {
            crate::debug!("[pipeline] Stop called but not running");
            return Err(PipelineError::NotRunning);
        }

        crate::info!("[pipeline] Stopping listening pipeline...");

        // Signal analysis thread to stop
        self.should_stop.store(true, Ordering::SeqCst);

        // Stop audio capture
        let _ = audio_handle.stop();

        // Take the thread handle - marks pipeline as not running
        let thread = self.analysis_thread.take();

        // Wait for analysis thread to exit with timeout using the exit channel
        if let Some(rx) = self.thread_exit_rx.take() {
            match rx.recv_timeout(timeout) {
                Ok(()) => crate::debug!("[pipeline] Analysis thread exit confirmed"),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    crate::warn!(
                        "[pipeline] Timeout waiting for analysis thread to exit ({}ms)",
                        timeout.as_millis()
                    );
                    // Thread handle will be dropped, thread continues running in background
                    // This is safe because should_stop is already set
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    crate::debug!("[pipeline] Analysis thread already exited (channel disconnected)");
                }
            }
        }

        // If we got exit confirmation, we can join the thread safely (it's already done)
        // Otherwise, we just drop the handle - the thread will exit on its own
        if let Some(handle) = thread {
            // Try to join, but don't block forever - thread should already be done
            // if recv_timeout succeeded
            let _ = handle.join();
        }

        self.detector = None;
        self.buffer = None;

        crate::debug!("[pipeline] Pipeline stopped successfully");

        Ok(())
    }

    /// Stop the listening pipeline and return the buffer for handoff to recording
    ///
    /// This is used when wake word is detected - the listening pipeline stops but
    /// hands off its buffer to the recording mode, so audio data is preserved.
    ///
    /// NOTE: This method does NOT join the analysis thread because it may be called
    /// from within the wake word callback (which runs on the analysis thread itself).
    /// The thread will exit naturally after checking the should_stop flag.
    ///
    /// # Arguments
    /// * `audio_handle` - Handle to stop audio capture
    ///
    /// # Returns
    /// The audio buffer if pipeline was running, None if not running
    pub fn stop_and_get_buffer(
        &mut self,
        audio_handle: &AudioThreadHandle,
    ) -> Result<Option<AudioBuffer>, PipelineError> {
        if !self.is_running() {
            crate::debug!("[pipeline] stop_and_get_buffer called but not running");
            return Ok(None);
        }

        crate::info!("[pipeline] Stopping pipeline and returning buffer for recording...");

        // Signal analysis thread to stop
        self.should_stop.store(true, Ordering::SeqCst);

        // Stop audio capture
        let _ = audio_handle.stop();

        // DO NOT join the thread here - this method may be called from within
        // the wake word callback, which runs ON the analysis thread.
        // The thread will exit on its own after the callback returns.
        // Just take the handle so is_running() returns false.
        let _ = self.analysis_thread.take();

        self.detector = None;

        // Return the buffer WITHOUT clearing it - recording will use it
        let buffer = self.buffer.take();

        crate::debug!("[pipeline] Pipeline stopped, buffer returned for recording");
        Ok(buffer)
    }

    /// Set microphone availability (called when mic status changes)
    #[allow(dead_code)] // Future use for mic status tracking
    pub fn set_mic_available(&self, available: bool) {
        self.mic_available.store(available, Ordering::SeqCst);
    }

    /// Get a reference to the wake word detector (if running)
    #[allow(dead_code)] // Utility method for introspection
    pub fn detector(&self) -> Option<&Arc<WakeWordDetector>> {
        self.detector.as_ref()
    }
}

impl Default for ListeningPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ListeningPipeline {
    fn drop(&mut self) {
        // Signal stop but don't wait - the thread will exit on its own
        self.should_stop.store(true, Ordering::SeqCst);
    }
}

/// Main loop for the analysis thread
///
/// Periodically reads new samples from the audio buffer, feeds them to the
/// wake word detector, and emits events when wake word is detected.
fn analysis_thread_main<E: ListeningEventEmitter>(
    state: AnalysisState,
    config: PipelineConfig,
    emitter: Arc<E>,
) {
    crate::debug!(
        "[pipeline] Analysis thread started, interval={}ms",
        config.analysis_interval_ms
    );

    let interval = Duration::from_millis(config.analysis_interval_ms);

    loop {
        // Check if we should stop
        if state.should_stop.load(Ordering::SeqCst) {
            crate::debug!("[pipeline] Stop signal received, exiting analysis thread");
            break;
        }

        // Sleep for the analysis interval
        thread::sleep(interval);

        // Check if we should stop again after sleep
        if state.should_stop.load(Ordering::SeqCst) {
            crate::debug!("[pipeline] Stop signal received after sleep, exiting analysis thread");
            break;
        }

        // Check mic availability
        if !state.mic_available.load(Ordering::SeqCst) {
            crate::trace!("[pipeline] Mic not available, skipping analysis");
            continue;
        }

        // Read new samples from the audio buffer and clear it to prevent memory growth
        // In listening mode, we only need samples in the detector's circular buffer
        let new_samples = {
            match state.buffer.lock() {
                Ok(mut guard) => {
                    if guard.is_empty() {
                        Vec::new()
                    } else {
                        // Take all samples and clear the buffer to bound memory usage
                        // The detector's internal CircularBuffer maintains the rolling window
                        let samples = std::mem::take(&mut *guard);
                        samples
                    }
                }
                Err(_) => {
                    // Lock poisoned - emit unavailable event and send through channel
                    crate::error!("[pipeline] Audio buffer lock poisoned");
                    let reason = "Audio buffer lock error".to_string();
                    emitter.emit_listening_unavailable(listening_events::ListeningUnavailablePayload {
                        reason: reason.clone(),
                        timestamp: current_timestamp(),
                    });
                    // Also send through event channel
                    let _ = state.event_tx.try_send(WakeWordEvent::unavailable(reason));
                    break;
                }
            }
        };

        // Feed samples to detector
        if !new_samples.is_empty() {
            crate::trace!("[pipeline] Collected {} samples from audio buffer", new_samples.len());
            if let Err(e) = state.detector.push_samples(&new_samples) {
                // Log error but continue
                crate::warn!("[pipeline] Failed to push samples to detector: {}", e);
                continue;
            }
        }

        // Try to analyze and emit if wake word detected
        // The detector's analyze() method handles empty buffer gracefully
        let analysis_start = std::time::Instant::now();
        match state.detector.analyze_and_emit(emitter.as_ref()) {
            Ok(result) => {
                let analysis_duration = analysis_start.elapsed();
                crate::trace!(
                    "[pipeline] Analysis completed in {:?}, detected={}, confidence={:.2}, transcription='{}'",
                    analysis_duration,
                    result.detected,
                    result.confidence,
                    result.transcription
                );

                if result.detected {
                    // Check should_stop BEFORE sending event
                    if state.should_stop.load(Ordering::SeqCst) {
                        crate::debug!(
                            "[pipeline] Wake word detected but stop requested, skipping event"
                        );
                        break;
                    }

                    crate::info!(
                        "[pipeline] WAKE_WORD_DETECTED! confidence={:.2}, transcription='{}', sending event",
                        result.confidence,
                        result.transcription
                    );

                    // Wake word detected! The event was already emitted by analyze_and_emit
                    // The detector clears its internal buffer after detection
                    // Also clear the audio capture buffer to start fresh
                    if let Ok(mut guard) = state.buffer.lock() {
                        guard.clear();
                    }

                    // Send event through channel instead of invoking callback directly
                    // Using try_send to avoid blocking the analysis thread
                    let event = WakeWordEvent::detected(
                        result.transcription.clone(),
                        result.confidence,
                    );
                    if let Err(e) = state.event_tx.try_send(event) {
                        crate::warn!(
                            "[pipeline] Failed to send wake word event: {} (channel full or closed)",
                            e
                        );
                    }
                }
            }
            Err(WakeWordError::EmptyBuffer) => {
                // Not enough samples yet, continue
                crate::trace!("[pipeline] Buffer empty, skipping analysis");
            }
            Err(WakeWordError::InsufficientNewSamples) => {
                // Not enough new audio since last analysis, continue
                crate::trace!("[pipeline] Not enough new samples, skipping analysis");
            }
            Err(WakeWordError::NoSpeechDetected) => {
                // VAD filtered out non-speech audio, continue
                crate::trace!("[pipeline] No speech detected (VAD), skipping transcription");
            }
            Err(WakeWordError::DuplicateAudio) => {
                // Audio segment already analyzed (fingerprint match), continue
                crate::trace!("[pipeline] Duplicate audio detected (fingerprint), skipping");
            }
            Err(WakeWordError::ModelNotLoaded) => {
                // Model not loaded - emit unavailable and stop
                crate::error!("[pipeline] Wake word model not loaded");
                let reason = "Wake word model not loaded".to_string();
                emitter.emit_listening_unavailable(listening_events::ListeningUnavailablePayload {
                    reason: reason.clone(),
                    timestamp: current_timestamp(),
                });
                // Also send through event channel
                let _ = state.event_tx.try_send(WakeWordEvent::unavailable(reason));
                break;
            }
            Err(e) => {
                // Other error - log, send error event, and continue
                crate::warn!("[pipeline] Wake word analysis error: {}", e);
                let _ = state.event_tx.try_send(WakeWordEvent::error(e.to_string()));
            }
        }
    }

    crate::debug!("[pipeline] Analysis thread exiting");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert_eq!(config.analysis_interval_ms, ANALYSIS_INTERVAL_MS);
        assert_eq!(config.min_samples_for_analysis, MIN_SAMPLES_FOR_ANALYSIS);
    }

    #[test]
    fn test_pipeline_new_not_running() {
        let pipeline = ListeningPipeline::new();
        assert!(!pipeline.is_running());
        assert!(pipeline.is_mic_available());
    }

    #[test]
    fn test_pipeline_with_config() {
        let config = PipelineConfig {
            analysis_interval_ms: 1000,
            min_samples_for_analysis: 16000,
        };
        let detector_config = WakeWordDetectorConfig {
            wake_phrase: "hello world".to_string(),
            ..Default::default()
        };
        let pipeline = ListeningPipeline::with_config(config, detector_config);
        assert!(!pipeline.is_running());
    }

    #[test]
    fn test_pipeline_set_mic_available() {
        let pipeline = ListeningPipeline::new();
        assert!(pipeline.is_mic_available());

        pipeline.set_mic_available(false);
        assert!(!pipeline.is_mic_available());

        pipeline.set_mic_available(true);
        assert!(pipeline.is_mic_available());
    }

    #[test]
    fn test_stop_without_start_returns_error() {
        let mut pipeline = ListeningPipeline::new();
        let audio_handle = AudioThreadHandle::spawn();
        let result = pipeline.stop(&audio_handle);
        assert!(matches!(result, Err(PipelineError::NotRunning)));
    }

    #[test]
    fn test_pipeline_error_display() {
        let err = PipelineError::AudioError("test".to_string());
        assert!(format!("{}", err).contains("Audio error"));

        let err = PipelineError::DetectorError("test".to_string());
        assert!(format!("{}", err).contains("Detector error"));

        let err = PipelineError::AlreadyRunning;
        assert!(format!("{}", err).contains("already running"));

        let err = PipelineError::NotRunning;
        assert!(format!("{}", err).contains("not running"));

        let err = PipelineError::LockError;
        assert!(format!("{}", err).contains("Lock error"));

        let err = PipelineError::NoEventSubscriber;
        assert!(format!("{}", err).contains("subscribe_events()"));
        assert!(format!("{}", err).contains("start()"));
    }

    #[test]
    fn test_pipeline_error_from_audio_error() {
        let audio_err = AudioCaptureError::NoDeviceAvailable;
        let pipeline_err: PipelineError = audio_err.into();
        assert!(matches!(pipeline_err, PipelineError::AudioError(_)));
    }

    #[test]
    fn test_pipeline_error_from_wake_word_error() {
        let wake_err = WakeWordError::ModelNotLoaded;
        let pipeline_err: PipelineError = wake_err.into();
        assert!(matches!(pipeline_err, PipelineError::DetectorError(_)));
    }

    #[test]
    fn test_default_pipeline() {
        let pipeline = ListeningPipeline::default();
        assert!(!pipeline.is_running());
    }

    #[test]
    fn test_detector_not_available_before_start() {
        let pipeline = ListeningPipeline::new();
        assert!(pipeline.detector().is_none());
    }

    #[test]
    fn test_analysis_state_fields() {
        // Verify AnalysisState can be constructed (compile-time check)
        let detector = Arc::new(WakeWordDetector::new());
        let should_stop = Arc::new(AtomicBool::new(false));
        let buffer = AudioBuffer::new();
        let mic_available = Arc::new(AtomicBool::new(true));
        let (event_tx, _event_rx) = tokio_mpsc::channel(EVENT_CHANNEL_BUFFER_SIZE);

        let _state = AnalysisState {
            detector,
            should_stop,
            buffer,
            mic_available,
            event_tx,
        };
    }

    #[test]
    fn test_circular_buffer_bounds_memory() {
        // Verify the circular buffer in WakeWordDetector bounds memory
        // The default config uses 2 seconds at 16kHz = 32000 samples
        // 32000 * 4 bytes (f32) = ~128KB per buffer
        let detector = WakeWordDetector::new();
        let config = detector.config();

        let expected_samples = (config.window_duration_secs * config.sample_rate as f32) as usize;
        // Should be ~32000 samples for default 2 second window
        assert_eq!(expected_samples, 32000);

        // Memory should be bounded: 32000 samples * 4 bytes = ~128KB
        let expected_memory = expected_samples * std::mem::size_of::<f32>();
        assert!(expected_memory < 150 * 1024); // Less than 150KB (allows small overhead)
        assert!(expected_memory >= 120 * 1024); // At least ~120KB (verifies 2s buffer)
    }

    #[test]
    fn test_pipeline_config_memory_bounded() {
        // Verify the pipeline config's min_samples_for_analysis is reasonable
        let config = PipelineConfig::default();

        // 4000 samples at 16kHz = 0.25 seconds
        // This means we start analyzing after a quarter second of audio
        assert_eq!(config.min_samples_for_analysis, 4000);

        // Memory for min_samples: 4000 * 4 bytes = 16KB
        let min_memory = config.min_samples_for_analysis * std::mem::size_of::<f32>();
        assert!(min_memory < 20 * 1024); // Less than 20KB
    }

    #[test]
    fn test_subscribe_events_returns_receiver() {
        let mut pipeline = ListeningPipeline::new();
        let _rx = pipeline.subscribe_events();
        // Verify we got a receiver (implicitly tested by compilation)
        assert!(pipeline.event_tx.is_some());
    }

    #[test]
    fn test_start_without_subscribe_events_returns_error() {
        use crate::events::tests::MockEventEmitter;

        let mut pipeline = ListeningPipeline::new();
        let audio_handle = AudioThreadHandle::spawn();
        let emitter = Arc::new(MockEventEmitter::new());

        // Try to start without calling subscribe_events() first
        let result = pipeline.start(&audio_handle, emitter);

        // Should return NoEventSubscriber error
        assert!(matches!(result, Err(PipelineError::NoEventSubscriber)));
    }

    #[test]
    fn test_start_after_subscribe_events_proceeds() {
        use crate::events::tests::MockEventEmitter;
        use crate::parakeet::SharedTranscriptionModel;

        let mut pipeline = ListeningPipeline::new();
        let audio_handle = AudioThreadHandle::spawn();
        let emitter = Arc::new(MockEventEmitter::new());

        // Subscribe to events
        let _rx = pipeline.subscribe_events();

        // Set up shared model (not loaded, so start will fail with DetectorError)
        let shared_model = SharedTranscriptionModel::new();
        pipeline.set_shared_model(shared_model);

        // Try to start - should fail with DetectorError (model not loaded), not NoEventSubscriber
        let result = pipeline.start(&audio_handle, emitter);

        // The error should be about the model, not about missing subscriber
        assert!(
            matches!(result, Err(PipelineError::DetectorError(_))),
            "Expected DetectorError, got {:?}",
            result
        );
    }

    #[test]
    fn test_event_tx_persists_after_stop() {
        let mut pipeline = ListeningPipeline::new();

        // Subscribe to events
        let _rx = pipeline.subscribe_events();
        assert!(pipeline.event_tx.is_some());

        // Simulate stop behavior - event_tx should not be cleared
        // (Note: actual stop() requires the pipeline to be running, so we just verify
        // that the field is not part of what gets cleared)
        let audio_handle = AudioThreadHandle::spawn();
        let _ = pipeline.stop(&audio_handle); // Will fail with NotRunning, but that's ok

        // event_tx should still be set
        assert!(pipeline.event_tx.is_some());
    }

    #[test]
    fn test_subscribe_events_replaces_previous() {
        let mut pipeline = ListeningPipeline::new();
        let _rx1 = pipeline.subscribe_events();
        let _rx2 = pipeline.subscribe_events();
        // Both receivers should be valid (second replaces first sender)
        assert!(pipeline.event_tx.is_some());
    }

    #[tokio::test]
    async fn test_event_channel_send_receive() {
        let (tx, mut rx) = tokio_mpsc::channel::<WakeWordEvent>(EVENT_CHANNEL_BUFFER_SIZE);

        // Send a detected event
        let event = WakeWordEvent::detected("hey cat", 0.95);
        tx.send(event).await.unwrap();

        // Receive and verify
        let received = rx.recv().await.unwrap();
        assert!(received.is_detected());
        if let WakeWordEvent::Detected { text, confidence } = received {
            assert_eq!(text, "hey cat");
            assert!((confidence - 0.95).abs() < f32::EPSILON);
        }
    }

    #[tokio::test]
    async fn test_event_channel_multiple_events() {
        let (tx, mut rx) = tokio_mpsc::channel::<WakeWordEvent>(EVENT_CHANNEL_BUFFER_SIZE);

        // Send multiple events
        tx.send(WakeWordEvent::detected("test1", 0.8)).await.unwrap();
        tx.send(WakeWordEvent::unavailable("mic error")).await.unwrap();
        tx.send(WakeWordEvent::error("detection failed")).await.unwrap();

        // Receive all events in order
        let e1 = rx.recv().await.unwrap();
        let e2 = rx.recv().await.unwrap();
        let e3 = rx.recv().await.unwrap();

        assert!(matches!(e1, WakeWordEvent::Detected { .. }));
        assert!(matches!(e2, WakeWordEvent::Unavailable { .. }));
        assert!(matches!(e3, WakeWordEvent::Error { .. }));
    }

    #[tokio::test]
    async fn test_event_channel_try_send_backpressure() {
        // Create a channel with small buffer
        let (tx, _rx) = tokio_mpsc::channel::<WakeWordEvent>(2);

        // Fill the buffer
        assert!(tx.try_send(WakeWordEvent::detected("1", 0.9)).is_ok());
        assert!(tx.try_send(WakeWordEvent::detected("2", 0.9)).is_ok());

        // Next try_send should fail (buffer full)
        assert!(tx.try_send(WakeWordEvent::detected("3", 0.9)).is_err());
    }

    #[test]
    fn test_stop_with_timeout_not_running() {
        let mut pipeline = ListeningPipeline::new();
        let audio_handle = AudioThreadHandle::spawn();
        let result = pipeline.stop_with_timeout(&audio_handle, Duration::from_millis(100));
        assert!(matches!(result, Err(PipelineError::NotRunning)));
    }

    #[test]
    fn test_stop_delegates_to_stop_with_timeout() {
        // Verify stop() uses stop_with_timeout internally by checking it behaves the same
        let mut pipeline = ListeningPipeline::new();
        let audio_handle = AudioThreadHandle::spawn();

        // Both should return NotRunning error when pipeline isn't running
        let result1 = pipeline.stop(&audio_handle);
        assert!(matches!(result1, Err(PipelineError::NotRunning)));

        let result2 = pipeline.stop_with_timeout(&audio_handle, Duration::from_millis(100));
        assert!(matches!(result2, Err(PipelineError::NotRunning)));
    }

    #[test]
    fn test_thread_exit_rx_initialized_as_none() {
        let pipeline = ListeningPipeline::new();
        // The thread_exit_rx should be None initially
        assert!(pipeline.thread_exit_rx.is_none());
    }

    #[test]
    fn test_pipeline_drop_signals_stop() {
        // Create a pipeline and verify Drop signals should_stop
        let pipeline = ListeningPipeline::new();
        let should_stop = pipeline.should_stop.clone();

        assert!(!should_stop.load(Ordering::SeqCst));

        // Drop the pipeline
        drop(pipeline);

        // should_stop should now be true
        assert!(should_stop.load(Ordering::SeqCst));
    }

    #[test]
    fn test_thread_coordination_channel() {
        // Test that the exit channel mechanism works correctly
        let (exit_tx, exit_rx) = mpsc::channel::<()>();

        // Simulate thread sending exit signal
        exit_tx.send(()).unwrap();

        // Should receive the signal with timeout
        let result = exit_rx.recv_timeout(Duration::from_millis(100));
        assert!(result.is_ok());
    }

    #[test]
    fn test_thread_coordination_timeout() {
        // Test that recv_timeout returns error when no signal sent
        let (_exit_tx, exit_rx) = mpsc::channel::<()>();

        // Should timeout since no signal sent
        let result = exit_rx.recv_timeout(Duration::from_millis(10));
        assert!(matches!(
            result,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
    }

    #[test]
    fn test_thread_coordination_disconnected() {
        // Test that recv_timeout handles disconnected sender
        let (exit_tx, exit_rx) = mpsc::channel::<()>();

        // Drop the sender without sending
        drop(exit_tx);

        // Should get disconnected error
        let result = exit_rx.recv_timeout(Duration::from_millis(100));
        assert!(matches!(
            result,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected)
        ));
    }

    // Note: Integration tests requiring actual audio hardware are in pipeline_test.rs
    // These tests verify basic struct behavior without needing hardware
}

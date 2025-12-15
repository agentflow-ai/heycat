// Listening audio pipeline for continuous wake word detection
// Manages audio capture in listening mode, routing samples to the wake word detector

use super::{WakeWordDetector, WakeWordDetectorConfig, WakeWordError};
use crate::audio::{AudioBuffer, AudioCaptureError, AudioThreadHandle};
use crate::events::{current_timestamp, listening_events, ListeningEventEmitter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Callback type for wake word detection
/// Called when "Hey Cat" is detected - should start recording
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
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::AudioError(msg) => write!(f, "Audio error: {}", msg),
            PipelineError::DetectorError(msg) => write!(f, "Detector error: {}", msg),
            PipelineError::AlreadyRunning => write!(f, "Pipeline is already running"),
            PipelineError::NotRunning => write!(f, "Pipeline is not running"),
            PipelineError::LockError => write!(f, "Lock error"),
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
            // Analyze every 500ms by default
            analysis_interval_ms: 500,
            // Need at least 0.5 seconds of audio before analyzing
            // 16000 Hz * 0.5s = 8000 samples
            min_samples_for_analysis: 8000,
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
    /// Callback to invoke when wake word is detected
    wake_word_callback: Option<Arc<WakeWordCallback>>,
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
    /// Wake word callback - called when "Hey Cat" is detected
    wake_word_callback: Arc<Mutex<Option<Arc<WakeWordCallback>>>>,
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
            analysis_thread: None,
            should_stop: Arc::new(AtomicBool::new(false)),
            mic_available: Arc::new(AtomicBool::new(true)),
            detector: None,
            buffer: None,
            wake_word_callback: Arc::new(Mutex::new(None)),
        }
    }

    /// Set the callback to be invoked when wake word is detected
    ///
    /// This callback is called from the analysis thread when "Hey Cat" is detected.
    /// The callback should start recording (e.g., call handle_toggle on HotkeyIntegration).
    pub fn set_wake_word_callback(&self, callback: WakeWordCallback) {
        if let Ok(mut guard) = self.wake_word_callback.lock() {
            *guard = Some(Arc::new(callback));
        }
    }

    /// Clear the wake word callback
    #[allow(dead_code)] // Utility method for cleanup
    pub fn clear_wake_word_callback(&self) {
        if let Ok(mut guard) = self.wake_word_callback.lock() {
            *guard = None;
        }
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
    pub fn start<E: ListeningEventEmitter + 'static>(
        &mut self,
        audio_handle: &AudioThreadHandle,
        emitter: Arc<E>,
    ) -> Result<AudioBuffer, PipelineError> {
        if self.is_running() {
            crate::debug!("[pipeline] Start called but already running");
            return Err(PipelineError::AlreadyRunning);
        }

        crate::info!("[pipeline] Starting listening pipeline...");

        // Create detector and load model
        let detector = Arc::new(WakeWordDetector::with_config(self.detector_config.clone()));
        detector.load_model()?;

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

        // Get wake word callback if set
        let wake_word_callback = self.wake_word_callback.lock()
            .ok()
            .and_then(|guard| guard.clone());

        // Create analysis state
        let state = AnalysisState {
            detector: detector.clone(),
            should_stop: self.should_stop.clone(),
            buffer: buffer_clone,
            mic_available: self.mic_available.clone(),
            wake_word_callback,
        };

        // Start analysis thread
        let config = self.config.clone();
        let analysis_thread = thread::spawn(move || {
            analysis_thread_main(state, config, emitter);
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
    /// # Arguments
    /// * `audio_handle` - Handle to stop audio capture
    pub fn stop(&mut self, audio_handle: &AudioThreadHandle) -> Result<(), PipelineError> {
        if !self.is_running() {
            crate::debug!("[pipeline] Stop called but not running");
            return Err(PipelineError::NotRunning);
        }

        crate::info!("[pipeline] Stopping listening pipeline...");

        // Signal analysis thread to stop
        self.should_stop.store(true, Ordering::SeqCst);

        // Stop audio capture
        let _ = audio_handle.stop();

        // Wait for analysis thread to finish
        if let Some(thread) = self.analysis_thread.take() {
            let _ = thread.join();
        }

        self.detector = None;
        self.buffer = None;

        crate::debug!("[pipeline] Pipeline stopped successfully");

        Ok(())
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
                    // Lock poisoned - emit unavailable event
                    crate::error!("[pipeline] Audio buffer lock poisoned");
                    emitter.emit_listening_unavailable(listening_events::ListeningUnavailablePayload {
                        reason: "Audio buffer lock error".to_string(),
                        timestamp: current_timestamp(),
                    });
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
                    crate::info!(
                        "[pipeline] WAKE_WORD_DETECTED! confidence={:.2}, transcription='{}', invoking callback",
                        result.confidence,
                        result.transcription
                    );

                    // Wake word detected! The event was already emitted by analyze_and_emit
                    // The detector clears its internal buffer after detection
                    // Also clear the audio capture buffer to start fresh
                    if let Ok(mut guard) = state.buffer.lock() {
                        guard.clear();
                    }

                    // Invoke callback to start recording
                    if let Some(ref callback) = state.wake_word_callback {
                        callback();
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
            Err(WakeWordError::ModelNotLoaded) => {
                // Model not loaded - emit unavailable and stop
                crate::error!("[pipeline] Wake word model not loaded");
                emitter.emit_listening_unavailable(listening_events::ListeningUnavailablePayload {
                    reason: "Wake word model not loaded".to_string(),
                    timestamp: current_timestamp(),
                });
                break;
            }
            Err(e) => {
                // Other error - log and continue
                crate::warn!("[pipeline] Wake word analysis error: {}", e);
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
        assert_eq!(config.analysis_interval_ms, 500);
        assert_eq!(config.min_samples_for_analysis, 8000);
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

        let _state = AnalysisState {
            detector,
            should_stop,
            buffer,
            mic_available,
            wake_word_callback: None,
        };
    }

    #[test]
    fn test_circular_buffer_bounds_memory() {
        // Verify the circular buffer in WakeWordDetector bounds memory
        // The default config uses 3 seconds at 16kHz = 48000 samples
        // 48000 * 4 bytes (f32) = ~192KB per buffer
        let detector = WakeWordDetector::new();
        let config = detector.config();

        let expected_samples = (config.window_duration_secs * config.sample_rate as f32) as usize;
        // Should be ~48000 samples for default 3 second window
        assert_eq!(expected_samples, 48000);

        // Memory should be bounded: 48000 samples * 4 bytes = ~192KB
        let expected_memory = expected_samples * std::mem::size_of::<f32>();
        assert!(expected_memory < 250 * 1024); // Less than 250KB (allows small overhead)
        assert!(expected_memory >= 180 * 1024); // At least ~180KB (verifies 3s buffer)
    }

    #[test]
    fn test_pipeline_config_memory_bounded() {
        // Verify the pipeline config's min_samples_for_analysis is reasonable
        let config = PipelineConfig::default();

        // 8000 samples at 16kHz = 0.5 seconds
        // This means we start analyzing after half a second of audio
        assert_eq!(config.min_samples_for_analysis, 8000);

        // Memory for min_samples: 8000 * 4 bytes = 32KB
        let min_memory = config.min_samples_for_analysis * std::mem::size_of::<f32>();
        assert!(min_memory < 50 * 1024); // Less than 50KB
    }

    // Note: Integration tests requiring actual audio hardware are in pipeline_test.rs
    // These tests verify basic struct behavior without needing hardware
}

// Recording detection coordinator
// Manages silence and cancel phrase detection during recording phase

use super::cancel::{CancelPhraseDetector, CancelPhraseDetectorConfig, CancelPhraseError};
use super::silence::{SilenceConfig, SilenceDetectionResult, SilenceDetector, SilenceStopReason};
use crate::audio::AudioBuffer;
use crate::events::ListeningEventEmitter;
use crate::recording::{RecordingManager, RecordingState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Coordinator for silence and cancel phrase detection during recording
///
/// When recording starts (triggered by wake word or hotkey), this coordinator:
/// 1. Starts monitoring audio for silence to auto-stop recording
/// 2. Starts monitoring for cancel phrases ("cancel", "nevermind") to abort
/// 3. Feeds audio samples to both detectors
/// 4. Triggers appropriate actions based on detection results
pub struct RecordingDetectors {
    /// Silence detector configuration
    silence_config: SilenceConfig,
    /// Cancel phrase detector configuration
    cancel_config: CancelPhraseDetectorConfig,
    /// Detection thread handle
    detection_thread: Option<JoinHandle<()>>,
    /// Flag to stop the detection thread
    should_stop: Arc<AtomicBool>,
}

impl RecordingDetectors {
    /// Create a new recording detectors coordinator with default configuration
    pub fn new() -> Self {
        Self::with_config(SilenceConfig::default(), CancelPhraseDetectorConfig::default())
    }

    /// Create a new recording detectors coordinator with custom configuration
    pub fn with_config(silence_config: SilenceConfig, cancel_config: CancelPhraseDetectorConfig) -> Self {
        Self {
            silence_config,
            cancel_config,
            detection_thread: None,
            should_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if detection is currently running
    pub fn is_running(&self) -> bool {
        self.detection_thread.is_some()
    }

    /// Start monitoring for silence and cancel phrases
    ///
    /// # Arguments
    /// * `buffer` - Audio buffer being filled during recording
    /// * `recording_manager` - Manager for state transitions
    /// * `audio_thread` - Handle to stop audio capture
    /// * `emitter` - Event emitter for cancel events
    /// * `return_to_listening` - Whether to return to Listening state after stop
    pub fn start_monitoring<E: ListeningEventEmitter + 'static>(
        &mut self,
        buffer: AudioBuffer,
        recording_manager: Arc<Mutex<RecordingManager>>,
        audio_thread: Arc<crate::audio::AudioThreadHandle>,
        emitter: Arc<E>,
        return_to_listening: bool,
    ) -> Result<(), String> {
        if self.is_running() {
            crate::debug!("[coordinator] Start monitoring called but already running");
            return Err("Detection already running".to_string());
        }

        crate::info!(
            "[coordinator] Starting recording detectors, return_to_listening={}",
            return_to_listening
        );

        // Reset stop flag
        self.should_stop.store(false, Ordering::SeqCst);

        // Create detectors
        let mut silence_detector = SilenceDetector::with_config(self.silence_config.clone());
        silence_detector.reset();

        let cancel_detector = CancelPhraseDetector::with_config(self.cancel_config.clone());
        // Load cancel phrase model
        if let Err(e) = cancel_detector.load_model() {
            crate::warn!("Cancel phrase model not loaded, cancel detection disabled: {}", e);
        }
        // Start cancel detection session
        if let Err(e) = cancel_detector.start_session() {
            crate::warn!("Failed to start cancel detection session: {}", e);
        }

        let should_stop = self.should_stop.clone();

        // Spawn detection thread
        let thread_handle = thread::spawn(move || {
            detection_loop(
                buffer,
                silence_detector,
                cancel_detector,
                recording_manager,
                audio_thread,
                emitter,
                should_stop,
                return_to_listening,
            );
        });

        self.detection_thread = Some(thread_handle);
        crate::info!("[coordinator] Detection thread spawned");
        Ok(())
    }

    /// Stop monitoring
    pub fn stop_monitoring(&mut self) {
        crate::debug!("[coordinator] Stopping monitoring");
        self.should_stop.store(true, Ordering::SeqCst);

        if let Some(thread) = self.detection_thread.take() {
            let _ = thread.join();
        }
        crate::debug!("[coordinator] Monitoring stopped");
    }
}

impl Default for RecordingDetectors {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RecordingDetectors {
    fn drop(&mut self) {
        self.stop_monitoring();
    }
}

/// Main detection loop
///
/// Reads audio samples and feeds them to silence and cancel detectors.
/// Takes action based on detection results.
fn detection_loop<E: ListeningEventEmitter>(
    buffer: AudioBuffer,
    mut silence_detector: SilenceDetector,
    cancel_detector: CancelPhraseDetector,
    recording_manager: Arc<Mutex<RecordingManager>>,
    audio_thread: Arc<crate::audio::AudioThreadHandle>,
    emitter: Arc<E>,
    should_stop: Arc<AtomicBool>,
    return_to_listening: bool,
) {
    crate::debug!("[coordinator] Detection loop starting");

    // Detection interval: check every 100ms
    let interval = Duration::from_millis(100);

    // Track samples for batch processing
    let mut samples_since_last_check: Vec<f32> = Vec::new();
    let mut loop_count: u64 = 0;

    loop {
        loop_count += 1;

        // Check if we should stop
        if should_stop.load(Ordering::SeqCst) {
            crate::debug!("[coordinator] Stop signal received, exiting loop");
            break;
        }

        // Check if still recording
        let is_recording = recording_manager
            .lock()
            .map(|m| m.get_state() == RecordingState::Recording)
            .unwrap_or(false);

        if !is_recording {
            // Recording stopped by other means (hotkey, timeout, etc.)
            crate::debug!("[coordinator] No longer in Recording state, exiting loop");
            break;
        }

        // Read samples from buffer
        let new_samples = {
            match buffer.lock() {
                Ok(guard) => {
                    if guard.is_empty() {
                        Vec::new()
                    } else {
                        // Get a copy of samples without clearing (recording still needs them)
                        guard.clone()
                    }
                }
                Err(_) => {
                    crate::error!("[coordinator] Audio buffer lock poisoned in detection loop");
                    break;
                }
            }
        };

        // Accumulate samples
        if !new_samples.is_empty() {
            samples_since_last_check.extend(&new_samples);
            crate::trace!(
                "[coordinator] Loop {}: accumulated {} new samples (total: {})",
                loop_count,
                new_samples.len(),
                samples_since_last_check.len()
            );
        }

        // Process samples if we have enough (at least 100ms worth at 16kHz = 1600 samples)
        if samples_since_last_check.len() >= 1600 {
            crate::trace!(
                "[coordinator] Processing {} samples for detection",
                samples_since_last_check.len()
            );

            // Feed to cancel phrase detector (only during cancellation window)
            if cancel_detector.is_window_open() {
                crate::trace!(
                    "[coordinator] Cancel window open, remaining={:.2}s",
                    cancel_detector.remaining_window_secs()
                );
                let _ = cancel_detector.push_samples(&samples_since_last_check);

                // Try to analyze for cancel phrase
                match cancel_detector.analyze_and_abort(emitter.as_ref(), &recording_manager, return_to_listening) {
                    Ok(result) => {
                        if result.detected {
                            crate::info!(
                                "[coordinator] Cancel phrase detected: '{}', aborting recording",
                                result.phrase.as_ref().unwrap_or(&"unknown".to_string())
                            );
                            // Stop audio capture
                            let _ = audio_thread.stop();
                            break;
                        } else {
                            crate::trace!("[coordinator] No cancel phrase detected");
                        }
                    }
                    Err(CancelPhraseError::WindowExpired) => {
                        crate::debug!("[coordinator] Cancel detection window expired");
                    }
                    Err(CancelPhraseError::ModelNotLoaded) => {
                        // Model not loaded, skip cancel detection (only log once)
                    }
                    Err(e) => {
                        crate::debug!("[coordinator] Cancel phrase detection error: {}", e);
                    }
                }
            }

            // Feed to silence detector
            let silence_result = silence_detector.process_samples(&samples_since_last_check);

            match silence_result {
                SilenceDetectionResult::Stop(reason) => {
                    crate::info!(
                        "[coordinator] Silence detection STOP: {:?}, samples_processed={}",
                        reason,
                        samples_since_last_check.len()
                    );

                    // Stop audio capture
                    let _ = audio_thread.stop();

                    // Transition to appropriate state
                    if let Ok(mut manager) = recording_manager.lock() {
                        match reason {
                            SilenceStopReason::SilenceAfterSpeech => {
                                // Normal completion - transition to Processing
                                // The recording will be saved and transcribed
                                crate::info!("[coordinator] Transitioning to Processing (normal completion)");
                                if let Err(e) = manager.transition_to(RecordingState::Processing) {
                                    crate::error!("[coordinator] Failed to transition to Processing: {:?}", e);
                                }
                            }
                            SilenceStopReason::NoSpeechTimeout => {
                                // False activation - abort without saving
                                let target = if return_to_listening {
                                    RecordingState::Listening
                                } else {
                                    RecordingState::Idle
                                };
                                crate::info!("[coordinator] Aborting recording (no speech), target={:?}", target);
                                if let Err(e) = manager.abort_recording(target) {
                                    crate::error!("[coordinator] Failed to abort recording: {:?}", e);
                                }
                            }
                        }
                    }
                    break;
                }
                SilenceDetectionResult::Continue => {
                    // Keep recording
                    crate::trace!("[coordinator] Silence detection: continue");
                }
            }

            // Clear processed samples
            samples_since_last_check.clear();
        }

        // Sleep until next check
        thread::sleep(interval);
    }

    crate::debug!("[coordinator] Detection loop exited after {} iterations", loop_count);

    // Clean up cancel detector session
    let _ = cancel_detector.end_session();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_detectors_new() {
        let detectors = RecordingDetectors::new();
        assert!(!detectors.is_running());
    }

    #[test]
    fn test_recording_detectors_with_config() {
        let silence_config = SilenceConfig {
            silence_duration_ms: 1000,
            ..Default::default()
        };
        let cancel_config = CancelPhraseDetectorConfig {
            cancellation_window_secs: 5.0,
            ..Default::default()
        };
        let detectors = RecordingDetectors::with_config(silence_config, cancel_config);
        assert!(!detectors.is_running());
    }

    #[test]
    fn test_recording_detectors_default() {
        let detectors = RecordingDetectors::default();
        assert!(!detectors.is_running());
    }

    #[test]
    fn test_stop_without_start() {
        let mut detectors = RecordingDetectors::new();
        // Should not panic
        detectors.stop_monitoring();
        assert!(!detectors.is_running());
    }
}

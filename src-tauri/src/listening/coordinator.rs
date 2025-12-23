// Recording detection coordinator
// Manages silence detection during recording phase

use super::silence::{SilenceConfig, SilenceDetectionResult, SilenceDetector, SilenceStopReason};
use super::ListeningPipeline;
use crate::audio::{encode_wav, AudioBuffer, SystemFileWriter, TARGET_SAMPLE_RATE};
use crate::audio_constants::{DETECTION_INTERVAL_MS, MIN_DETECTION_SAMPLES};
use crate::events::{ListeningEventEmitter, RecordingEventEmitter, RecordingStoppedPayload};
use crate::recording::{RecordingManager, RecordingMetadata, RecordingState};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Coordinator for silence detection during recording
///
/// When recording starts (triggered by wake word or hotkey), this coordinator:
/// 1. Starts monitoring audio for silence to auto-stop recording
/// 2. Feeds audio samples to the silence detector
/// 3. Triggers appropriate actions based on detection results
pub struct RecordingDetectors {
    /// Silence detector configuration
    silence_config: SilenceConfig,
    /// Detection thread handle
    detection_thread: Option<JoinHandle<()>>,
    /// Flag to stop the detection thread
    should_stop: Arc<AtomicBool>,
    /// Directory for saving recordings (supports worktree isolation)
    recordings_dir: PathBuf,
}

impl RecordingDetectors {
    /// Create a new recording detectors coordinator with default configuration
    /// Uses the main repo recordings path (no worktree isolation)
    pub fn new() -> Self {
        Self::with_recordings_dir(
            crate::paths::get_recordings_dir(None)
                .unwrap_or_else(|_| PathBuf::from(".").join("heycat").join("recordings")),
        )
    }

    /// Create a new recording detectors coordinator with a specific recordings directory
    pub fn with_recordings_dir(recordings_dir: PathBuf) -> Self {
        Self::with_config_and_recordings_dir(SilenceConfig::default(), recordings_dir)
    }

    /// Create a new recording detectors coordinator with custom configuration
    pub fn with_config(silence_config: SilenceConfig) -> Self {
        Self::with_config_and_recordings_dir(
            silence_config,
            crate::paths::get_recordings_dir(None)
                .unwrap_or_else(|_| PathBuf::from(".").join("heycat").join("recordings")),
        )
    }

    /// Create a new recording detectors coordinator with custom configuration and recordings directory
    pub fn with_config_and_recordings_dir(
        silence_config: SilenceConfig,
        recordings_dir: PathBuf,
    ) -> Self {
        Self {
            silence_config,
            detection_thread: None,
            should_stop: Arc::new(AtomicBool::new(false)),
            recordings_dir,
        }
    }

    /// Check if detection is currently running
    ///
    /// Returns true only if the detection thread exists AND is still actively running.
    /// A finished thread that hasn't been joined yet returns false.
    pub fn is_running(&self) -> bool {
        match &self.detection_thread {
            Some(handle) => !handle.is_finished(),
            None => false,
        }
    }

    /// Start monitoring for silence and cancel phrases
    ///
    /// # Arguments
    /// * `buffer` - Audio buffer being filled during recording
    /// * `recording_manager` - Manager for state transitions
    /// * `audio_thread` - Handle to stop audio capture
    /// * `emitter` - Event emitter for cancel events
    /// * `return_to_listening` - Whether to return to Listening state after stop
    /// * `listening_pipeline` - Optional pipeline to restart if return_to_listening is true
    /// * `transcription_callback` - Optional callback to spawn transcription after recording saves
    #[allow(clippy::too_many_arguments)]
    pub fn start_monitoring<E: ListeningEventEmitter + RecordingEventEmitter + 'static>(
        &mut self,
        buffer: AudioBuffer,
        recording_manager: Arc<Mutex<RecordingManager>>,
        audio_thread: Arc<crate::audio::AudioThreadHandle>,
        emitter: Arc<E>,
        return_to_listening: bool,
        listening_pipeline: Option<Arc<Mutex<ListeningPipeline>>>,
        transcription_callback: Option<Box<dyn Fn(String) + Send + 'static>>,
    ) -> Result<(), String> {
        // Clean up any finished detection thread from previous session
        // This handles the case where the thread exited naturally (e.g., silence detected)
        // but the JoinHandle wasn't taken/joined yet
        if let Some(handle) = &self.detection_thread {
            if handle.is_finished() {
                if let Some(h) = self.detection_thread.take() {
                    let _ = h.join();
                    crate::debug!("[coordinator] Cleaned up finished detection thread");
                }
            }
        }

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

        // Create silence detector
        let mut silence_detector = SilenceDetector::with_config(self.silence_config.clone());
        silence_detector.reset();

        let should_stop = self.should_stop.clone();
        let recordings_dir = self.recordings_dir.clone();

        // Spawn detection thread
        let thread_handle = thread::spawn(move || {
            detection_loop(
                buffer,
                silence_detector,
                recording_manager,
                audio_thread,
                emitter,
                should_stop,
                return_to_listening,
                listening_pipeline,
                transcription_callback,
                recordings_dir,
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
/// Reads audio samples and feeds them to the silence detector.
/// Takes action based on detection results.
#[allow(clippy::too_many_arguments)]
fn detection_loop<E: ListeningEventEmitter + RecordingEventEmitter + 'static>(
    buffer: AudioBuffer,
    mut silence_detector: SilenceDetector,
    recording_manager: Arc<Mutex<RecordingManager>>,
    audio_thread: Arc<crate::audio::AudioThreadHandle>,
    emitter: Arc<E>,
    should_stop: Arc<AtomicBool>,
    return_to_listening: bool,
    listening_pipeline: Option<Arc<Mutex<ListeningPipeline>>>,
    transcription_callback: Option<Box<dyn Fn(String) + Send + 'static>>,
    recordings_dir: PathBuf,
) {
    crate::debug!("[coordinator] Detection loop starting");

    // Detection interval
    let interval = Duration::from_millis(DETECTION_INTERVAL_MS);

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

        // Drain NEW samples from ring buffer (lock-free read)
        // This also accumulates samples internally for WAV encoding
        let new_samples = buffer.drain_samples();

        // Accumulate samples for silence detection
        if !new_samples.is_empty() {
            samples_since_last_check.extend(&new_samples);
            crate::trace!(
                "[coordinator] Loop {}: accumulated {} new samples (total: {})",
                loop_count,
                new_samples.len(),
                samples_since_last_check.len()
            );
        }

        // Process samples if we have enough (at least 100ms worth at 16kHz)
        if samples_since_last_check.len() >= MIN_DETECTION_SAMPLES {
            crate::trace!(
                "[coordinator] Processing {} samples for detection",
                samples_since_last_check.len()
            );

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
                                // Normal completion - save recording and optionally restart listening
                                crate::info!("[coordinator] Recording complete, saving...");

                                // 1. Transition to Processing
                                if let Err(e) = manager.transition_to(RecordingState::Processing) {
                                    crate::error!("[coordinator] Failed to transition to Processing: {:?}", e);
                                    break;
                                }

                                // 2. Get samples and encode WAV
                                let sample_rate = manager.get_sample_rate().unwrap_or(TARGET_SAMPLE_RATE);
                                let (file_path, sample_count, duration_secs) = match manager.get_audio_buffer() {
                                    Ok(buf) => {
                                        match buf.lock() {
                                            Ok(samples) => {
                                                let count = samples.len();
                                                let duration = count as f64 / sample_rate as f64;
                                                let writer = SystemFileWriter::new(recordings_dir.clone());
                                                match encode_wav(&samples, sample_rate, &writer) {
                                                    Ok(path) => {
                                                        crate::info!("[coordinator] WAV saved to: {}", path);
                                                        (path, count, duration)
                                                    }
                                                    Err(e) => {
                                                        crate::error!("[coordinator] WAV encoding failed: {:?}", e);
                                                        (String::new(), count, duration)
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                crate::error!("[coordinator] Buffer lock failed: {:?}", e);
                                                (String::new(), 0, 0.0)
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        crate::error!("[coordinator] No audio buffer: {:?}", e);
                                        (String::new(), 0, 0.0)
                                    }
                                };

                                // 3. Emit recording_stopped event
                                let metadata = RecordingMetadata {
                                    duration_secs,
                                    file_path: file_path.clone(),
                                    sample_count,
                                    stop_reason: None,
                                };
                                emitter.emit_recording_stopped(RecordingStoppedPayload {
                                    metadata: metadata.clone(),
                                });
                                crate::info!("[coordinator] Recording stopped: {} samples, {:.2}s", sample_count, duration_secs);

                                // 4. Spawn transcription (same flow as hotkey recording)
                                if let Some(ref callback) = transcription_callback {
                                    if !file_path.is_empty() {
                                        crate::info!("[coordinator] Spawning transcription for: {}", file_path);
                                        callback(file_path.clone());
                                    }
                                }

                                // 5. Transition to final state and restart listening if needed
                                let target_state = if return_to_listening {
                                    RecordingState::Listening
                                } else {
                                    RecordingState::Idle
                                };
                                if let Err(e) = manager.transition_to(target_state) {
                                    crate::error!("[coordinator] Failed to transition to {:?}: {:?}", target_state, e);
                                }

                                // Drop the manager lock before restarting pipeline
                                drop(manager);

                                // 6. Restart listening pipeline if return_to_listening
                                if return_to_listening {
                                    if let Some(ref pipeline_arc) = listening_pipeline {
                                        if let Ok(mut pipeline) = pipeline_arc.lock() {
                                            crate::info!("[coordinator] Restarting listening pipeline...");
                                            match pipeline.start(&audio_thread, emitter.clone()) {
                                                Ok(_) => {
                                                    crate::info!("[coordinator] Listening pipeline restarted successfully");
                                                }
                                                Err(e) => {
                                                    crate::error!("[coordinator] Failed to restart listening pipeline: {:?}", e);
                                                }
                                            }
                                        }
                                    } else {
                                        crate::warn!("[coordinator] return_to_listening=true but no pipeline provided");
                                    }
                                }
                            }
                            SilenceStopReason::NoSpeechTimeout => {
                                // False activation - abort without saving
                                // Transition to target state (Listening or Idle) before dropping lock
                                let target_state = if return_to_listening {
                                    RecordingState::Listening
                                } else {
                                    RecordingState::Idle
                                };
                                crate::info!(
                                    "[coordinator] Aborting recording (no speech), transitioning to {:?}",
                                    target_state
                                );
                                if let Err(e) = manager.abort_recording(target_state) {
                                    crate::error!("[coordinator] Failed to abort recording: {:?}", e);
                                }

                                // Drop the manager lock before restarting pipeline
                                drop(manager);

                                // Restart listening pipeline if return_to_listening (even on timeout)
                                // NOTE: State is already Listening, no need to re-acquire manager lock
                                if return_to_listening {
                                    if let Some(ref pipeline_arc) = listening_pipeline {
                                        if let Ok(mut pipeline) = pipeline_arc.lock() {
                                            crate::info!("[coordinator] Restarting listening pipeline after false activation...");
                                            match pipeline.start(&audio_thread, emitter.clone()) {
                                                Ok(_) => {
                                                    crate::info!("[coordinator] Listening pipeline restarted successfully");
                                                }
                                                Err(e) => {
                                                    crate::error!("[coordinator] Failed to restart listening pipeline: {:?}", e);
                                                }
                                            }
                                        }
                                    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests removed per docs/TESTING.md:
    // - test_recording_detectors_new: Obvious default
    // - test_recording_detectors_default: Obvious default (duplicate)

    // ==================== Behavior Tests ====================

    #[test]
    fn test_recording_detectors_with_config() {
        let silence_config = SilenceConfig {
            silence_duration_ms: 1000,
            ..Default::default()
        };
        let detectors = RecordingDetectors::with_config(silence_config);
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

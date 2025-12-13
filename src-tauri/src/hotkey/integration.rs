// Hotkey-to-recording integration module
// Connects global hotkey to recording state with debouncing
// Uses unified command implementations for start/stop logic

use crate::audio::AudioThreadHandle;
use crate::commands::logic::{get_last_recording_buffer_impl, start_recording_impl, stop_recording_impl};
use crate::events::{
    current_timestamp, RecordingErrorPayload, RecordingEventEmitter, RecordingStartedPayload,
    RecordingStoppedPayload, TranscriptionCompletedPayload, TranscriptionErrorPayload,
    TranscriptionEventEmitter, TranscriptionStartedPayload,
};
use crate::model::check_model_exists;
use crate::recording::{RecordingManager, RecordingState};
use crate::whisper::{TranscriptionService, WhisperManager};
use crate::{debug, error, info, trace, warn};
use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Debounce duration for hotkey presses (200ms)
pub const DEBOUNCE_DURATION_MS: u64 = 200;

/// Handles hotkey toggle with debouncing and event emission
pub struct HotkeyIntegration<R: RecordingEventEmitter, T: TranscriptionEventEmitter> {
    last_toggle_time: Option<Instant>,
    debounce_duration: Duration,
    recording_emitter: R,
    /// Optional audio thread handle - when present, starts/stops capture on toggle
    audio_thread: Option<Arc<AudioThreadHandle>>,
    /// Optional WhisperManager for auto-transcription after recording stops
    whisper_manager: Option<Arc<WhisperManager>>,
    /// Transcription event emitter for emitting events from spawned thread
    transcription_emitter: Option<Arc<T>>,
    /// Reference to recording state for getting audio buffer in transcription thread
    recording_state: Option<Arc<Mutex<RecordingManager>>>,
}

impl<R: RecordingEventEmitter, T: TranscriptionEventEmitter + 'static> HotkeyIntegration<R, T> {
    /// Create a new HotkeyIntegration with default debounce duration
    pub fn new(recording_emitter: R) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(DEBOUNCE_DURATION_MS),
            recording_emitter,
            audio_thread: None,
            whisper_manager: None,
            transcription_emitter: None,
            recording_state: None,
        }
    }

    /// Add an audio thread handle (builder pattern)
    pub fn with_audio_thread(mut self, handle: Arc<AudioThreadHandle>) -> Self {
        self.audio_thread = Some(handle);
        self
    }

    /// Add WhisperManager for auto-transcription (builder pattern)
    pub fn with_whisper_manager(mut self, manager: Arc<WhisperManager>) -> Self {
        self.whisper_manager = Some(manager);
        self
    }

    /// Add transcription event emitter for emitting events from spawned thread (builder pattern)
    pub fn with_transcription_emitter(mut self, emitter: Arc<T>) -> Self {
        self.transcription_emitter = Some(emitter);
        self
    }

    /// Add recording state reference for transcription thread (builder pattern)
    pub fn with_recording_state(mut self, state: Arc<Mutex<RecordingManager>>) -> Self {
        self.recording_state = Some(state);
        self
    }

    /// Create with custom debounce duration (for testing)
    #[cfg(test)]
    pub fn with_debounce(recording_emitter: R, debounce_ms: u64) -> Self {
        Self {
            last_toggle_time: None,
            debounce_duration: Duration::from_millis(debounce_ms),
            recording_emitter,
            audio_thread: None,
            whisper_manager: None,
            transcription_emitter: None,
            recording_state: None,
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
                trace!("Toggle debounced");
                return false;
            }
        }

        self.last_toggle_time = Some(now);

        // Check current state to decide action
        let current_state = match state.lock() {
            Ok(guard) => guard.get_state(),
            Err(e) => {
                error!("Failed to acquire lock: {}", e);
                self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                    message: "Internal error: state lock poisoned".to_string(),
                });
                return false;
            }
        };

        debug!("Toggle received, current state: {:?}", current_state);

        match current_state {
            RecordingState::Idle => {
                info!("Starting recording...");
                // Check model availability before starting
                let model_available = check_model_exists().unwrap_or(false);
                // Use unified command implementation
                match start_recording_impl(state, self.audio_thread.as_deref(), model_available) {
                    Ok(()) => {
                        self.recording_emitter
                            .emit_recording_started(RecordingStartedPayload {
                                timestamp: current_timestamp(),
                            });
                        info!("Recording started, emitted recording_started event");
                        true
                    }
                    Err(e) => {
                        error!("Failed to start recording: {}", e);
                        self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Recording => {
                info!("Stopping recording...");
                // Use unified command implementation
                match stop_recording_impl(state, self.audio_thread.as_deref()) {
                    Ok(metadata) => {
                        info!(
                            "Recording stopped: {} samples, {:.2}s duration",
                            metadata.sample_count, metadata.duration_secs
                        );
                        self.recording_emitter
                            .emit_recording_stopped(RecordingStoppedPayload { metadata });
                        debug!("Emitted recording_stopped event");

                        // Auto-transcribe if whisper manager is configured
                        self.spawn_transcription();

                        true
                    }
                    Err(e) => {
                        error!("Failed to stop recording: {}", e);
                        self.recording_emitter.emit_recording_error(RecordingErrorPayload {
                            message: e,
                        });
                        false
                    }
                }
            }
            RecordingState::Processing => {
                // In Processing state - ignore toggle (busy)
                debug!("Toggle ignored - already processing");
                false
            }
        }
    }

    /// Spawn transcription in a separate thread
    ///
    /// Gets audio buffer, transcribes, copies to clipboard, and emits events.
    /// No-op if whisper manager, transcription emitter, or recording state is not configured.
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn spawn_transcription(&self) {
        // Check all required components are present
        let whisper_manager = match &self.whisper_manager {
            Some(wm) => wm.clone(),
            None => {
                debug!("Transcription skipped: no whisper manager configured");
                return;
            }
        };

        let transcription_emitter = match &self.transcription_emitter {
            Some(te) => te.clone(),
            None => {
                debug!("Transcription skipped: no transcription emitter configured");
                return;
            }
        };

        let recording_state = match &self.recording_state {
            Some(rs) => rs.clone(),
            None => {
                debug!("Transcription skipped: no recording state configured");
                return;
            }
        };

        // Check if model is loaded
        if !whisper_manager.is_loaded() {
            info!("Transcription skipped: whisper model not loaded");
            return;
        }

        info!("Spawning transcription thread...");

        std::thread::spawn(move || {
            // Emit transcription_started event
            let start_time = Instant::now();
            transcription_emitter.emit_transcription_started(TranscriptionStartedPayload {
                timestamp: current_timestamp(),
            });

            // Get audio buffer
            let samples = match get_last_recording_buffer_impl(&recording_state) {
                Ok(audio_data) => audio_data.samples,
                Err(e) => {
                    error!("Failed to get recording buffer: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: format!("Failed to get recording buffer: {}", e),
                    });
                    return;
                }
            };

            debug!("Transcribing {} samples...", samples.len());

            // Perform transcription
            match whisper_manager.transcribe(&samples) {
                Ok(text) => {
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    info!(
                        "Transcription completed in {}ms: {} chars",
                        duration_ms,
                        text.len()
                    );

                    // Copy to clipboard
                    match Clipboard::new() {
                        Ok(mut clipboard) => {
                            if let Err(e) = clipboard.set_text(&text) {
                                warn!("Failed to copy to clipboard: {}", e);
                            } else {
                                debug!("Transcribed text copied to clipboard");
                            }
                        }
                        Err(e) => {
                            warn!("Failed to access clipboard: {}", e);
                        }
                    }

                    // Emit transcription_completed event
                    transcription_emitter.emit_transcription_completed(TranscriptionCompletedPayload {
                        text,
                        duration_ms,
                    });

                    // Reset whisper state to idle
                    if let Err(e) = whisper_manager.reset_to_idle() {
                        warn!("Failed to reset whisper state: {}", e);
                    }
                }
                Err(e) => {
                    error!("Transcription failed: {}", e);
                    transcription_emitter.emit_transcription_error(TranscriptionErrorPayload {
                        error: e.to_string(),
                    });

                    // Reset whisper state to idle on error
                    if let Err(reset_err) = whisper_manager.reset_to_idle() {
                        warn!("Failed to reset whisper state: {}", reset_err);
                    }
                }
            }
        });
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

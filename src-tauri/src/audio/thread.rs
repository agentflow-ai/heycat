// Dedicated audio thread for capturing audio
//
// This module provides a thread-safe interface to audio capture.
// SwiftBackend handles audio via AVFoundation on a dedicated thread
// and communicates via channels.

use super::{AudioBuffer, AudioCaptureBackend, AudioCaptureError, StopReason, SwiftBackend};
use super::diagnostics::QualityWarning;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Response from a Start command
pub type StartResponse = Result<u32, AudioCaptureError>;

/// Result of stopping a recording (includes reason if auto-stopped)
#[derive(Debug, Clone)]
pub struct StopResult {
    /// Why the recording stopped (None = user initiated)
    pub reason: Option<StopReason>,
    /// Quality warnings generated during the recording
    pub warnings: Vec<QualityWarning>,
    /// Raw audio data (if debug mode was enabled) with device sample rate
    pub raw_audio: Option<(Vec<f32>, u32)>,
    /// Path to the captured WAV file with duration in ms
    /// Caller should move/rename this to the final location (instant, no I/O)
    pub capture_file: Option<(String, u64)>,
}

/// Commands sent to the audio thread
pub enum AudioCommand {
    /// Start capturing audio into the provided buffer
    /// Includes a response channel to return the sample rate or error
    Start {
        buffer: AudioBuffer,
        response_tx: Sender<StartResponse>,
        device_name: Option<String>,
    },
    /// Stop capturing audio and return result via channel
    Stop(Option<Sender<StopResult>>),
    /// Shutdown the audio thread (used in tests)
    #[allow(dead_code)]
    Shutdown,
}

/// Handle to the audio capture thread
///
/// This handle is Send + Sync and can be safely shared across threads.
/// Commands are sent via channel to the dedicated audio thread.
/// When dropped, the audio thread is gracefully shutdown.
pub struct AudioThreadHandle {
    sender: Sender<AudioCommand>,
    thread: Option<JoinHandle<()>>,
}

impl AudioThreadHandle {
    /// Spawn a new audio capture thread
    pub fn spawn() -> Self {
        let (sender, receiver) = mpsc::channel();

        let thread = thread::spawn(move || {
            audio_thread_main(receiver);
        });

        Self {
            sender,
            thread: Some(thread),
        }
    }

    /// Start audio capture into the provided buffer using a specific device
    ///
    /// Returns the actual sample rate of the audio device on success.
    /// If the specified device is not found, falls back to the default device.
    /// Blocks until the audio thread responds.
    ///
    /// # Arguments
    /// * `buffer` - The audio buffer to capture samples into
    /// * `device_name` - Optional device name; None uses the default device
    #[must_use = "this returns a Result that should be handled"]
    pub fn start_with_device(
        &self,
        buffer: AudioBuffer,
        device_name: Option<String>,
    ) -> Result<u32, AudioThreadError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.sender
            .send(AudioCommand::Start {
                buffer,
                response_tx,
                device_name,
            })
            .map_err(|_| AudioThreadError::ThreadDisconnected)?;

        // Wait for response from audio thread
        response_rx
            .recv()
            .map_err(|_| AudioThreadError::ThreadDisconnected)?
            .map_err(AudioThreadError::CaptureError)
    }

    /// Stop audio capture and return the stop result
    #[must_use = "this returns a Result that should be handled"]
    pub fn stop(&self) -> Result<StopResult, AudioThreadError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.sender
            .send(AudioCommand::Stop(Some(response_tx)))
            .map_err(|_| AudioThreadError::ThreadDisconnected)?;

        // Wait for response with the stop reason
        response_rx
            .recv()
            .map_err(|_| AudioThreadError::ThreadDisconnected)
    }

    /// Shutdown the audio thread gracefully (used in tests)
    #[allow(dead_code)]
    pub fn shutdown(&self) -> Result<(), AudioThreadError> {
        self.sender
            .send(AudioCommand::Shutdown)
            .map_err(|_| AudioThreadError::ThreadDisconnected)
    }
}

impl Drop for AudioThreadHandle {
    /// Gracefully shutdown the audio thread when the handle is dropped.
    ///
    /// Sends a Shutdown command and waits for the thread to exit.
    /// This ensures clean resource cleanup when the application closes.
    fn drop(&mut self) {
        // Send shutdown command - ignore errors if thread already exited
        let _ = self.sender.send(AudioCommand::Shutdown);

        // Wait for thread to finish
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Errors from audio thread operations
#[derive(Debug, Clone, PartialEq)]
pub enum AudioThreadError {
    /// The audio thread has disconnected
    ThreadDisconnected,
    /// Audio capture failed
    CaptureError(AudioCaptureError),
}

impl std::fmt::Display for AudioThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioThreadError::ThreadDisconnected => write!(f, "Audio thread disconnected"),
            AudioThreadError::CaptureError(e) => write!(f, "Audio capture error: {}", e),
        }
    }
}

impl std::error::Error for AudioThreadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AudioThreadError::CaptureError(e) => Some(e),
            _ => None,
        }
    }
}

/// Main loop for the audio thread
///
/// Creates SwiftBackend (AVFoundation) and processes commands.
/// This runs on a dedicated thread for consistent audio handling.
#[cfg_attr(coverage_nightly, coverage(off))]
fn audio_thread_main(receiver: Receiver<AudioCommand>) {
    crate::info!("Audio thread started, creating SwiftBackend (AVFoundation)...");
    let mut backend = SwiftBackend::new();
    crate::debug!("SwiftBackend created, waiting for commands...");

    // Track the stop signal receiver when recording is active
    let mut stop_signal_rx: Option<Receiver<StopReason>> = None;
    // Track pending stop reason from auto-stop
    let mut pending_stop_reason: Option<StopReason> = None;

    loop {
        // Check for stop signals from callbacks (non-blocking)
        if let Some(ref rx) = stop_signal_rx {
            if let Ok(reason) = rx.try_recv() {
                crate::info!("Received auto-stop signal: {:?}", reason);
                // Auto-stop the recording
                match backend.stop() {
                    Ok(()) => crate::debug!("Auto-stopped successfully"),
                    Err(e) => crate::error!("Auto-stop failed: {:?}", e),
                }
                // Store the reason for when Stop command arrives
                pending_stop_reason = Some(reason);
                stop_signal_rx = None;
            }
        }

        // Wait for commands (with timeout to allow checking stop signals)
        let command = if stop_signal_rx.is_some() {
            // Recording active - use timeout to allow periodic signal checks
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(cmd) => cmd,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        } else {
            // Not recording - block until command
            match receiver.recv() {
                Ok(cmd) => cmd,
                Err(_) => break,
            }
        };

        match command {
            AudioCommand::Start {
                buffer,
                response_tx,
                device_name,
            } => {
                crate::debug!("Received START command, device={:?}", device_name);
                // Create stop signal channel for callbacks
                let (stop_tx, stop_rx) = mpsc::channel();
                stop_signal_rx = Some(stop_rx);
                pending_stop_reason = None;

                let result = backend.start(
                    buffer,
                    Some(stop_tx),
                    device_name,
                );
                match &result {
                    Ok(sample_rate) => {
                        crate::info!("Audio capture started at {} Hz", sample_rate)
                    }
                    Err(e) => {
                        crate::error!("Audio capture failed to start: {:?}", e);
                        stop_signal_rx = None; // Clear receiver on failure
                    }
                }
                // Send response back - ignore if receiver dropped
                let _ = response_tx.send(result);
            }
            AudioCommand::Stop(response_tx) => {
                crate::debug!("Received STOP command");
                // Use pending reason if auto-stopped, otherwise None (user-initiated)
                let reason = pending_stop_reason.take();
                if reason.is_none() {
                    // Only stop if not already auto-stopped
                    match backend.stop() {
                        Ok(()) => crate::debug!("Audio capture stopped successfully"),
                        Err(e) => crate::error!("Audio capture failed to stop: {:?}", e),
                    }
                }
                stop_signal_rx = None;

                // Get warnings, raw audio, and capture file from backend
                let warnings = backend.take_warnings();
                let raw_audio = backend.take_raw_audio();
                let capture_file = backend.take_capture_file();

                // Send stop result back
                if let Some(tx) = response_tx {
                    let _ = tx.send(StopResult { reason, warnings, raw_audio, capture_file });
                }
            }
            AudioCommand::Shutdown => {
                crate::debug!("Received SHUTDOWN command");
                let _ = backend.stop();
                break;
            }
        }
    }
    crate::info!("Audio thread exiting");
}

#[cfg(test)]
#[path = "thread_test.rs"]
mod tests;

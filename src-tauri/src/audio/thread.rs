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

                // Get warnings and raw audio from backend (captured during stop)
                let warnings = backend.take_warnings();
                let raw_audio = backend.take_raw_audio();

                // Send stop result back
                if let Some(tx) = response_tx {
                    let _ = tx.send(StopResult { reason, warnings, raw_audio });
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
mod tests {
    use super::*;

    #[test]
    fn test_audio_thread_handle_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AudioThreadHandle>();
    }

    #[test]
    fn test_spawn_and_shutdown() {
        let handle = AudioThreadHandle::spawn();
        assert!(handle.shutdown().is_ok());
    }

    #[test]
    fn test_drop_shuts_down_thread() {
        // Spawn a thread and immediately drop it
        let handle = AudioThreadHandle::spawn();
        drop(handle);
        // If we get here without hanging, the Drop impl worked correctly
    }

    /// Test that start and stop commands work
    /// Excluded from coverage because hardware availability varies
    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_start_stop_commands() {
        let handle = AudioThreadHandle::spawn();
        let buffer = AudioBuffer::new();

        // Start returns sample rate on success (or CaptureError if no device)
        let result = handle.start_with_device(buffer, None);
        // Either succeeds with sample rate or fails with CaptureError (no device in CI)
        match result {
            Ok(sample_rate) => assert!(sample_rate > 0),
            Err(AudioThreadError::CaptureError(_)) => {} // Expected in CI without audio device
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        // Stop should succeed
        assert!(handle.stop().is_ok());

        // Shutdown
        assert!(handle.shutdown().is_ok());
    }

    /// Test AudioCommand::Start includes device_name field
    #[test]
    fn test_audio_command_start_includes_device() {
        let buffer = AudioBuffer::new();
        let (response_tx, _response_rx) = mpsc::channel::<StartResponse>();

        // Test with Some device name
        let cmd_with_device = AudioCommand::Start {
            buffer: buffer.clone(),
            response_tx: response_tx.clone(),
            device_name: Some("Test Microphone".to_string()),
        };

        // Verify the command can hold device_name (compile-time check)
        match cmd_with_device {
            AudioCommand::Start { device_name, .. } => {
                assert_eq!(device_name, Some("Test Microphone".to_string()));
            }
            _ => panic!("Expected Start command"),
        }

        // Test with None device name
        let (response_tx2, _) = mpsc::channel::<StartResponse>();
        let cmd_without_device = AudioCommand::Start {
            buffer,
            response_tx: response_tx2,
            device_name: None,
        };

        match cmd_without_device {
            AudioCommand::Start { device_name, .. } => {
                assert!(device_name.is_none());
            }
            _ => panic!("Expected Start command"),
        }
    }

    /// Test start_with_device sends correct command
    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_start_with_device_passes_device_name() {
        let handle = AudioThreadHandle::spawn();
        let buffer = AudioBuffer::new();

        // Start with a non-existent device - should fall back to default
        let result = handle.start_with_device(buffer, Some("NonExistent Device".to_string()));

        // Either succeeds with sample rate (fallback to default) or fails with CaptureError
        match result {
            Ok(sample_rate) => assert!(sample_rate > 0),
            Err(AudioThreadError::CaptureError(_)) => {} // Expected in CI without audio device
            Err(e) => panic!("Unexpected error: {:?}", e),
        }

        // Stop and shutdown
        let _ = handle.stop();
        assert!(handle.shutdown().is_ok());
    }

    // test_start_uses_default_device removed: start() method removed (unused convenience wrapper)
}

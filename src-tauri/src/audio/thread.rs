// Dedicated audio thread for capturing audio
//
// This module provides a thread-safe interface to audio capture.
// CpalBackend contains cpal::Stream which is NOT Send+Sync, so we isolate
// it on a dedicated thread and communicate via channels.

use super::{AudioBuffer, AudioCaptureBackend, AudioCaptureError, CpalBackend};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Response from a Start command
pub type StartResponse = Result<u32, AudioCaptureError>;

/// Commands sent to the audio thread
pub enum AudioCommand {
    /// Start capturing audio into the provided buffer
    /// Includes a response channel to return the sample rate or error
    Start(AudioBuffer, Sender<StartResponse>),
    /// Stop capturing audio
    Stop,
    /// Shutdown the audio thread (used in tests)
    #[allow(dead_code)]
    Shutdown,
}

/// Handle to the audio capture thread
///
/// This handle is Send + Sync and can be safely shared across threads.
/// Commands are sent via channel to the dedicated audio thread.
pub struct AudioThreadHandle {
    sender: Sender<AudioCommand>,
    _thread: JoinHandle<()>,
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
            _thread: thread,
        }
    }

    /// Start audio capture into the provided buffer
    ///
    /// Returns the actual sample rate of the audio device on success.
    /// Blocks until the audio thread responds.
    pub fn start(&self, buffer: AudioBuffer) -> Result<u32, AudioThreadError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.sender
            .send(AudioCommand::Start(buffer, response_tx))
            .map_err(|_| AudioThreadError::ThreadDisconnected)?;

        // Wait for response from audio thread
        response_rx
            .recv()
            .map_err(|_| AudioThreadError::ThreadDisconnected)?
            .map_err(AudioThreadError::CaptureError)
    }

    /// Stop audio capture
    pub fn stop(&self) -> Result<(), AudioThreadError> {
        self.sender
            .send(AudioCommand::Stop)
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

/// Errors from audio thread operations
#[derive(Debug, Clone, PartialEq)]
pub enum AudioThreadError {
    /// The audio thread has disconnected
    ThreadDisconnected,
    /// Audio capture failed
    CaptureError(AudioCaptureError),
}

/// Main loop for the audio thread
///
/// Creates CpalBackend and processes commands.
/// This runs on a dedicated thread where CpalBackend can safely live.
#[cfg_attr(coverage_nightly, coverage(off))]
fn audio_thread_main(receiver: Receiver<AudioCommand>) {
    eprintln!("[audio-thread] Audio thread started, creating CpalBackend...");
    let mut backend = CpalBackend::new();
    eprintln!("[audio-thread] CpalBackend created, waiting for commands...");

    while let Ok(command) = receiver.recv() {
        match command {
            AudioCommand::Start(buffer, response_tx) => {
                eprintln!("[audio-thread] Received START command");
                let result = backend.start(buffer);
                match &result {
                    Ok(sample_rate) => {
                        eprintln!(
                            "[audio-thread] Audio capture started successfully at {} Hz",
                            sample_rate
                        )
                    }
                    Err(e) => eprintln!("[audio-thread] Audio capture failed to start: {:?}", e),
                }
                // Send response back - ignore if receiver dropped
                let _ = response_tx.send(result);
            }
            AudioCommand::Stop => {
                eprintln!("[audio-thread] Received STOP command");
                match backend.stop() {
                    Ok(()) => eprintln!("[audio-thread] Audio capture stopped successfully"),
                    Err(e) => eprintln!("[audio-thread] Audio capture failed to stop: {:?}", e),
                }
            }
            AudioCommand::Shutdown => {
                eprintln!("[audio-thread] Received SHUTDOWN command");
                let _ = backend.stop();
                break;
            }
        }
    }
    eprintln!("[audio-thread] Audio thread exiting");
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

    /// Test that start and stop commands work
    /// Excluded from coverage because hardware availability varies
    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn test_start_stop_commands() {
        let handle = AudioThreadHandle::spawn();
        let buffer = AudioBuffer::new();

        // Start returns sample rate on success (or CaptureError if no device)
        let result = handle.start(buffer);
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
}

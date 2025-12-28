// Audio monitoring module for real-time level metering
//
// Provides audio level monitoring for device testing in settings UI.
// Uses the unified SharedAudioEngine which provides both level monitoring
// and audio capture through a single AVAudioEngine instance.
//
// The engine stays running even when monitoring is "stopped" - this allows
// seamless transition between level monitoring and audio capture without
// device conflicts.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::swift::{self, AudioEngineResult};

/// Commands sent to the audio monitor thread
enum MonitorCommand {
    /// Initialize the audio engine without starting level forwarding.
    /// Used to pre-warm the engine at app startup for instant audio settings UI.
    Init {
        device_name: Option<String>,
        response_tx: Sender<Result<(), String>>,
    },
    /// Start monitoring with device name and level sender
    Start {
        device_name: Option<String>,
        level_tx: Sender<u8>,
        response_tx: Sender<Result<(), String>>,
    },
    /// Stop monitoring (with optional response channel for synchronous stop)
    Stop {
        response_tx: Option<Sender<()>>,
    },
    /// Shutdown the thread
    Shutdown,
}

/// Handle to the audio monitor thread
///
/// This handle is Send + Sync and can be safely shared via Tauri state.
/// The actual AVFoundation monitoring runs on a dedicated thread.
pub struct AudioMonitorHandle {
    command_tx: Sender<MonitorCommand>,
    thread: Option<JoinHandle<()>>,
}

impl AudioMonitorHandle {
    /// Spawn a new audio monitor thread
    pub fn spawn() -> Self {
        let (command_tx, command_rx) = mpsc::channel();

        let thread = thread::spawn(move || {
            monitor_thread_main(command_rx);
        });

        Self {
            command_tx,
            thread: Some(thread),
        }
    }

    /// Start monitoring audio levels for the specified device
    ///
    /// Returns Ok(level_receiver) on success - the receiver will receive level values (0-100).
    /// Call stop() when done monitoring.
    pub fn start(&self, device_name: Option<String>) -> Result<Receiver<u8>, String> {
        let (level_tx, level_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        self.command_tx
            .send(MonitorCommand::Start {
                device_name,
                level_tx,
                response_tx,
            })
            .map_err(|_| "Monitor thread disconnected".to_string())?;

        // Wait for response
        response_rx
            .recv()
            .map_err(|_| "Monitor thread disconnected".to_string())??;

        Ok(level_rx)
    }

    /// Stop monitoring audio levels (synchronous - waits for Swift to release device)
    pub fn stop(&self) -> Result<(), String> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(MonitorCommand::Stop {
                response_tx: Some(response_tx),
            })
            .map_err(|_| "Monitor thread disconnected".to_string())?;

        // Wait for confirmation that Swift has stopped and released the device
        response_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|_| "Monitor stop timed out".to_string())
    }

    /// Shutdown the monitor thread
    #[allow(dead_code)]
    pub fn shutdown(&self) -> Result<(), String> {
        self.command_tx
            .send(MonitorCommand::Shutdown)
            .map_err(|_| "Monitor thread disconnected".to_string())
    }

    /// Initialize the audio engine without starting level forwarding.
    ///
    /// Call this at app startup to pre-warm the engine so audio settings UI is instant.
    /// This is idempotent - if the engine is already running, it returns Ok immediately.
    pub fn init(&self, device_name: Option<String>) -> Result<(), String> {
        let (response_tx, response_rx) = mpsc::channel();

        self.command_tx
            .send(MonitorCommand::Init {
                device_name,
                response_tx,
            })
            .map_err(|_| "Monitor thread disconnected".to_string())?;

        // Wait for response
        response_rx
            .recv()
            .map_err(|_| "Monitor thread disconnected".to_string())?
    }
}

impl Drop for AudioMonitorHandle {
    fn drop(&mut self) {
        // Send shutdown command - ignore errors if thread already exited
        let _ = self.command_tx.send(MonitorCommand::Shutdown);

        // Wait for thread to finish
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Main loop for the monitor thread
///
/// Uses the SharedAudioEngine for level monitoring. The engine provides
/// level values continuously once started, even during audio capture.
#[cfg_attr(coverage_nightly, coverage(off))]
fn monitor_thread_main(command_rx: Receiver<MonitorCommand>) {
    crate::debug!("Audio monitor thread started (SharedAudioEngine)");

    // Track the level sender for the current monitoring session
    let mut level_tx: Option<Sender<u8>> = None;

    loop {
        // If monitoring, poll for level every 50ms
        let timeout = if level_tx.is_some() {
            Duration::from_millis(50)
        } else {
            Duration::from_secs(60) // Long timeout when not monitoring
        };

        match command_rx.recv_timeout(timeout) {
            Ok(MonitorCommand::Init {
                device_name,
                response_tx,
            }) => {
                crate::debug!("Monitor: Received INIT command");

                // Start engine if not already running (pre-warm for instant UI)
                if !swift::audio_engine_is_running() {
                    match swift::audio_engine_start(device_name.as_deref()) {
                        AudioEngineResult::Ok => {
                            crate::info!("Audio engine pre-initialized at startup");
                            let _ = response_tx.send(Ok(()));
                        }
                        AudioEngineResult::Failed(error) => {
                            crate::warn!("Failed to pre-initialize audio engine: {}", error);
                            let _ = response_tx.send(Err(error));
                        }
                    }
                } else {
                    crate::debug!("Audio engine already running, skipping init");
                    let _ = response_tx.send(Ok(()));
                }
                // Note: level_tx stays None - we're just pre-warming, not monitoring yet
            }
            Ok(MonitorCommand::Start {
                device_name,
                level_tx: new_level_tx,
                response_tx,
            }) => {
                crate::debug!("Monitor: Received START command");

                // Ensure engine is running (may already be running for capture)
                if !swift::audio_engine_is_running() {
                    match swift::audio_engine_start(device_name.as_deref()) {
                        AudioEngineResult::Ok => {
                            level_tx = Some(new_level_tx);
                            let _ = response_tx.send(Ok(()));
                            crate::info!("Audio monitor started via SharedAudioEngine");
                        }
                        AudioEngineResult::Failed(error) => {
                            let _ = response_tx.send(Err(error));
                        }
                    }
                } else if device_name.is_some() {
                    // Engine running, but device may need switching
                    if let AudioEngineResult::Failed(error) = swift::audio_engine_set_device(device_name.as_deref()) {
                        crate::warn!("Failed to set device: {}", error);
                        // Don't fail - continue with current device
                    }
                    level_tx = Some(new_level_tx);
                    let _ = response_tx.send(Ok(()));
                    crate::info!("Audio monitor attached to running SharedAudioEngine");
                } else {
                    // Engine already running, just attach
                    level_tx = Some(new_level_tx);
                    let _ = response_tx.send(Ok(()));
                    crate::info!("Audio monitor attached to running SharedAudioEngine");
                }
            }
            Ok(MonitorCommand::Stop { response_tx }) => {
                crate::debug!("Monitor: Received STOP command");
                // Don't stop the engine - just stop sending levels
                // Engine stays running for potential capture or other monitoring
                level_tx = None;
                // Signal that stop is complete
                if let Some(tx) = response_tx {
                    let _ = tx.send(());
                }
            }
            Ok(MonitorCommand::Shutdown) => {
                crate::debug!("Monitor: Received SHUTDOWN command");
                // On shutdown, stop the engine entirely
                swift::audio_engine_stop();
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Timeout - poll for level if monitoring
                if let Some(ref tx) = level_tx {
                    if swift::audio_engine_is_running() {
                        let level = swift::audio_engine_get_level();
                        // Send level - ignore if receiver dropped
                        if tx.send(level).is_err() {
                            // Receiver dropped, stop sending levels
                            // Don't stop the engine - it may be used for capture
                            crate::debug!("Monitor: Level receiver dropped, stopping level polling");
                            level_tx = None;
                        }
                    } else {
                        // Engine stopped unexpectedly
                        crate::warn!("Monitor: SharedAudioEngine stopped unexpectedly");
                        level_tx = None;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                crate::debug!("Monitor: Command channel closed, exiting");
                swift::audio_engine_stop();
                break;
            }
        }
    }

    crate::debug!("Audio monitor thread exiting");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_monitor_handle_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AudioMonitorHandle>();
    }

    #[test]
    fn test_spawn_and_drop() {
        let handle = AudioMonitorHandle::spawn();
        drop(handle);
        // If we get here without hanging, the Drop impl worked correctly
    }

    #[test]
    fn test_stop_without_start() {
        let handle = AudioMonitorHandle::spawn();
        // Stop when not started should be fine
        assert!(handle.stop().is_ok());
    }

    #[test]
    fn test_shutdown() {
        let handle = AudioMonitorHandle::spawn();
        assert!(handle.shutdown().is_ok());
    }

    #[test]
    fn test_engine_running_query() {
        // Ensure clean state
        swift::audio_engine_stop();

        // Should not be running after stop
        assert!(!swift::audio_engine_is_running(), "Engine should not be running after stop");
    }

    #[test]
    fn test_init_without_device() {
        let handle = AudioMonitorHandle::spawn();
        // Init without device should succeed (pre-warms the engine)
        assert!(handle.init(None).is_ok());
    }

    #[test]
    fn test_init_is_idempotent() {
        let handle = AudioMonitorHandle::spawn();
        // First init
        assert!(handle.init(None).is_ok());
        // Second init should also succeed (engine already running)
        assert!(handle.init(None).is_ok());
    }

    #[test]
    fn test_start_after_init_works() {
        let handle = AudioMonitorHandle::spawn();
        // Pre-warm the engine
        assert!(handle.init(None).is_ok());
        // Start monitoring - should attach to running engine instantly
        let result = handle.start(None);
        assert!(result.is_ok());
        // Clean up
        assert!(handle.stop().is_ok());
    }
}

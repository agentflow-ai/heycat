// Audio monitoring module for real-time level metering
//
// Provides audio level monitoring for device testing in settings UI.
// Uses a dedicated thread (like AudioThreadHandle) to isolate the cpal::Stream
// which is not Send+Sync. Communication is via channels.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::debug;

/// Commands sent to the audio monitor thread
enum MonitorCommand {
    /// Start monitoring with device name and level sender
    Start {
        device_name: Option<String>,
        level_tx: Sender<u8>,
        response_tx: Sender<Result<(), String>>,
    },
    /// Stop monitoring
    Stop,
    /// Shutdown the thread
    Shutdown,
}

/// Handle to the audio monitor thread
///
/// This handle is Send + Sync and can be safely shared via Tauri state.
/// The actual cpal::Stream lives on a dedicated thread.
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

    /// Stop monitoring audio levels
    pub fn stop(&self) -> Result<(), String> {
        self.command_tx
            .send(MonitorCommand::Stop)
            .map_err(|_| "Monitor thread disconnected".to_string())
    }

    /// Shutdown the monitor thread
    #[allow(dead_code)]
    pub fn shutdown(&self) -> Result<(), String> {
        self.command_tx
            .send(MonitorCommand::Shutdown)
            .map_err(|_| "Monitor thread disconnected".to_string())
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
#[cfg_attr(coverage_nightly, coverage(off))]
fn monitor_thread_main(command_rx: Receiver<MonitorCommand>) {
    debug!("Audio monitor thread started");

    // Stream is Option because we start/stop monitoring.
    // The stream is kept alive by holding it - dropping stops monitoring.
    // We use _stream to indicate it's intentionally unused except for its lifetime.
    let mut _stream: Option<cpal::Stream> = None;

    loop {
        match command_rx.recv() {
            Ok(MonitorCommand::Start {
                device_name,
                level_tx,
                response_tx,
            }) => {
                debug!("Monitor: Received START command");

                // Stop existing stream first by dropping it
                _stream = None;

                // Create new stream
                match create_monitor_stream(device_name, level_tx) {
                    Ok(s) => {
                        _stream = Some(s);
                        let _ = response_tx.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = response_tx.send(Err(e));
                    }
                }
            }
            Ok(MonitorCommand::Stop) => {
                debug!("Monitor: Received STOP command");
                _stream = None; // Drop stream to stop monitoring
            }
            Ok(MonitorCommand::Shutdown) => {
                debug!("Monitor: Received SHUTDOWN command");
                _stream = None;
                break;
            }
            Err(_) => {
                debug!("Monitor: Command channel closed, exiting");
                break;
            }
        }
    }

    debug!("Audio monitor thread exiting");
}

/// Create a monitoring stream for the specified device
#[cfg_attr(coverage_nightly, coverage(off))]
fn create_monitor_stream(
    device_name: Option<String>,
    level_tx: Sender<u8>,
) -> Result<cpal::Stream, String> {
    let host = cpal::default_host();

    // Find the requested device or use default
    let device = if let Some(ref name) = device_name {
        host.input_devices()
            .map_err(|e| format!("Failed to enumerate devices: {}", e))?
            .find(|d| d.name().map(|n| n == *name).unwrap_or(false))
            .ok_or_else(|| format!("Device not found: {}", name))?
    } else {
        host.default_input_device()
            .ok_or_else(|| "No default input device available".to_string())?
    };

    let config = device
        .default_input_config()
        .map_err(|e| format!("Failed to get device config: {}", e))?;

    debug!(
        "Starting audio monitor on {:?} at {} Hz",
        device.name(),
        config.sample_rate().0
    );

    // Track sample count for throttling (~20 emissions per second)
    let sample_rate = config.sample_rate().0 as usize;
    let samples_per_emission = sample_rate / 20; // ~50ms worth of samples
    let mut sample_count: usize = 0;
    let mut accumulated_sum_squares: f32 = 0.0;

    let stream = device
        .build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Accumulate sum of squares for RMS calculation
                for &sample in data {
                    accumulated_sum_squares += sample * sample;
                    sample_count += 1;
                }

                // Emit level when we have enough samples
                if sample_count >= samples_per_emission {
                    let rms = (accumulated_sum_squares / sample_count as f32).sqrt();

                    // Convert to 0-100 scale with headroom adjustment
                    // Normal speech is typically -20 to -10 dBFS, we scale to make it visible
                    let level = (rms * 300.0).min(100.0) as u8;

                    // Send level - ignore if receiver dropped (monitoring stopped)
                    let _ = level_tx.send(level);

                    // Reset accumulators
                    sample_count = 0;
                    accumulated_sum_squares = 0.0;
                }
            },
            |err| {
                crate::warn!("Audio monitor stream error: {}", err);
            },
            None,
        )
        .map_err(|e| format!("Failed to build input stream: {}", e))?;

    stream
        .play()
        .map_err(|e| format!("Failed to start stream: {}", e))?;

    Ok(stream)
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
}

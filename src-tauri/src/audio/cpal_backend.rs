// cpal-based audio capture backend
// This code interacts with hardware and is excluded from coverage measurement
//
// Note: All impl blocks here are excluded from coverage because they
// interact with hardware and cannot be unit tested.
#![cfg_attr(coverage_nightly, coverage(off))]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;

use super::{AudioBuffer, AudioCaptureBackend, AudioCaptureError, CaptureState, StopReason, MAX_BUFFER_SAMPLES};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

/// Audio capture backend using cpal for platform-specific audio capture
pub struct CpalBackend {
    state: CaptureState,
    stream: Option<Stream>,
}

impl CpalBackend {
    /// Create a new cpal backend
    pub fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            stream: None,
        }
    }
}

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCaptureBackend for CpalBackend {
    fn start(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<Sender<StopReason>>,
    ) -> Result<u32, AudioCaptureError> {
        eprintln!("[cpal] Starting audio capture...");

        // Get the default audio host
        let host = cpal::default_host();
        eprintln!("[cpal] Host: {:?}", host.id());

        // Get the default input device
        let device = host.default_input_device().ok_or_else(|| {
            eprintln!("[cpal] ERROR: No input device available!");
            AudioCaptureError::NoDeviceAvailable
        })?;
        eprintln!(
            "[cpal] Input device: {:?}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        // Get the default input config
        let config = device.default_input_config().map_err(|e| {
            eprintln!("[cpal] ERROR: Failed to get input config: {}", e);
            AudioCaptureError::DeviceError(e.to_string())
        })?;
        let actual_sample_rate = config.sample_rate().0;
        eprintln!(
            "[cpal] Config: {} Hz, {:?}, {} channels",
            actual_sample_rate,
            config.sample_format(),
            config.channels()
        );

        // Create an error handler closure
        let err_fn = |err: cpal::StreamError| {
            eprintln!("Audio stream error: {}", err);
        };

        // Shared flag to ensure we only signal once
        let signaled = std::sync::Arc::new(AtomicBool::new(false));

        // Build the input stream based on sample format
        // Each callback checks buffer size to prevent unbounded memory growth
        // and signals if buffer is full or lock fails
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let buffer_clone = buffer.clone();
                let signal_clone = stop_signal.clone();
                let signaled_clone = signaled.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        match buffer_clone.lock() {
                            Ok(mut guard) => {
                                let remaining = MAX_BUFFER_SAMPLES.saturating_sub(guard.len());
                                if remaining > 0 {
                                    let to_add = data.len().min(remaining);
                                    guard.extend_from_slice(&data[..to_add]);
                                } else if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    // Buffer full - signal once
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::BufferFull);
                                    }
                                }
                            }
                            Err(_) => {
                                // Lock poisoned - signal once
                                if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::LockError);
                                    }
                                }
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let buffer_clone = buffer.clone();
                let signal_clone = stop_signal.clone();
                let signaled_clone = signaled.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        match buffer_clone.lock() {
                            Ok(mut guard) => {
                                let remaining = MAX_BUFFER_SAMPLES.saturating_sub(guard.len());
                                if remaining > 0 {
                                    // Convert i16 samples to f32 normalized to [-1.0, 1.0]
                                    guard.extend(
                                        data.iter()
                                            .take(remaining)
                                            .map(|&s| s as f32 / i16::MAX as f32),
                                    );
                                } else if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    // Buffer full - signal once
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::BufferFull);
                                    }
                                }
                            }
                            Err(_) => {
                                // Lock poisoned - signal once
                                if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::LockError);
                                    }
                                }
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let buffer_clone = buffer.clone();
                let signal_clone = stop_signal;
                let signaled_clone = signaled;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        match buffer_clone.lock() {
                            Ok(mut guard) => {
                                let remaining = MAX_BUFFER_SAMPLES.saturating_sub(guard.len());
                                if remaining > 0 {
                                    // Convert u16 samples to f32 normalized to [-1.0, 1.0]
                                    guard.extend(
                                        data.iter()
                                            .take(remaining)
                                            .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0),
                                    );
                                } else if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    // Buffer full - signal once
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::BufferFull);
                                    }
                                }
                            }
                            Err(_) => {
                                // Lock poisoned - signal once
                                if !signaled_clone.swap(true, Ordering::SeqCst) {
                                    if let Some(ref sender) = signal_clone {
                                        let _ = sender.send(StopReason::LockError);
                                    }
                                }
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            _ => {
                return Err(AudioCaptureError::DeviceError(
                    "Unsupported sample format".to_string(),
                ))
            }
        }
        .map_err(|e| {
            eprintln!("[cpal] ERROR: Failed to build input stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        // Start the stream
        stream.play().map_err(|e| {
            eprintln!("[cpal] ERROR: Failed to start stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        eprintln!("[cpal] Audio stream started successfully!");
        self.stream = Some(stream);
        self.state = CaptureState::Capturing;
        Ok(actual_sample_rate)
    }

    fn stop(&mut self) -> Result<(), AudioCaptureError> {
        eprintln!("[cpal] Stopping audio capture...");
        if let Some(stream) = self.stream.take() {
            // Stream will be dropped here, stopping capture
            drop(stream);
            eprintln!("[cpal] Audio stream stopped");
        } else {
            eprintln!("[cpal] No active stream to stop");
        }
        self.state = CaptureState::Stopped;
        Ok(())
    }
}

// cpal-based audio capture backend
// This code interacts with hardware and is excluded from coverage measurement
//
// Note: All impl blocks here are excluded from coverage because they
// interact with hardware and cannot be unit tested.
#![cfg_attr(coverage_nightly, coverage(off))]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, Stream};
use rubato::{FftFixedIn, Resampler};

use super::{AudioBuffer, AudioCaptureBackend, AudioCaptureError, CaptureState, StopReason, MAX_RESAMPLE_BUFFER_SAMPLES, TARGET_SAMPLE_RATE};
use crate::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

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

/// Find an audio input device by name
///
/// Searches through all input devices and returns the one matching the given name.
/// Returns None if no device with that name is found.
fn find_device_by_name(name: &str) -> Option<cpal::Device> {
    let host = cpal::default_host();
    host.input_devices()
        .ok()?
        .find(|d| d.name().map(|n| n == name).unwrap_or(false))
}

/// Try to find a supported config with the target sample rate
fn find_config_with_sample_rate(
    device: &cpal::Device,
    target_rate: u32,
) -> Option<cpal::SupportedStreamConfig> {
    if let Ok(configs) = device.supported_input_configs() {
        for config_range in configs {
            let min_rate = config_range.min_sample_rate().0;
            let max_rate = config_range.max_sample_rate().0;
            if min_rate <= target_rate && target_rate <= max_rate {
                return Some(config_range.with_sample_rate(SampleRate(target_rate)));
            }
        }
    }
    None
}

/// Create a resampler for converting from source rate to target rate
fn create_resampler(
    source_rate: u32,
    target_rate: u32,
    chunk_size: usize,
) -> Result<FftFixedIn<f32>, AudioCaptureError> {
    FftFixedIn::new(
        source_rate as usize,
        target_rate as usize,
        chunk_size,
        1, // sub_chunks - use 1 for simplicity
        1, // channels - mono
    )
    .map_err(|e| AudioCaptureError::DeviceError(format!("Failed to create resampler: {}", e)))
}

/// Shared state for audio processing callback
/// Captures all the Arc-wrapped resources needed by the callback
struct CallbackState {
    buffer: AudioBuffer,
    stop_signal: Option<Sender<StopReason>>,
    signaled: Arc<AtomicBool>,
    resampler: Option<Arc<Mutex<FftFixedIn<f32>>>>,
    resample_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_size: usize,
}

impl CallbackState {
    /// Process f32 audio samples - handles resampling and buffer management
    ///
    /// This is the core audio processing logic, extracted to avoid duplication
    /// across F32, I16, and U16 sample format callbacks.
    fn process_samples(&self, f32_samples: &[f32]) {
        let samples_to_add = if let Some(ref resampler) = self.resampler {
            // Accumulate samples and resample when we have enough
            let mut resample_buf = match self.resample_buffer.lock() {
                Ok(buf) => buf,
                Err(_) => return,
            };

            // Signal stop if resample buffer overflows - data loss is unacceptable
            if resample_buf.len() + f32_samples.len() > MAX_RESAMPLE_BUFFER_SAMPLES {
                error!("Resample buffer overflow: resampling can't keep up with audio input");
                if !self.signaled.swap(true, Ordering::SeqCst) {
                    if let Some(ref sender) = self.stop_signal {
                        let _ = sender.send(StopReason::ResampleOverflow);
                    }
                }
                return;
            }
            resample_buf.extend_from_slice(f32_samples);

            // Process full chunks using pre-allocated buffer
            let mut resampled = Vec::new();
            while resample_buf.len() >= self.chunk_size {
                // Use pre-allocated chunk buffer to avoid allocation
                if let Ok(mut chunk_buf) = self.chunk_buffer.lock() {
                    chunk_buf.copy_from_slice(&resample_buf[..self.chunk_size]);
                    resample_buf.drain(..self.chunk_size);
                    if let Ok(mut r) = resampler.lock() {
                        if let Ok(output) = r.process(&[chunk_buf.as_slice()], None) {
                            if !output.is_empty() {
                                resampled.extend_from_slice(&output[0]);
                            }
                        }
                    }
                } else {
                    // Fallback to allocation if chunk buffer lock fails
                    let chunk: Vec<f32> = resample_buf.drain(..self.chunk_size).collect();
                    if let Ok(mut r) = resampler.lock() {
                        if let Ok(output) = r.process(&[chunk], None) {
                            if !output.is_empty() {
                                resampled.extend_from_slice(&output[0]);
                            }
                        }
                    }
                }
            }
            resampled
        } else {
            // No resampling needed
            f32_samples.to_vec()
        };

        // Use lock-free ring buffer for reduced contention
        // Check if buffer is full before pushing
        if self.buffer.is_full() {
            if !self.signaled.swap(true, Ordering::SeqCst) {
                if let Some(ref sender) = self.stop_signal {
                    let _ = sender.send(StopReason::BufferFull);
                }
            }
            return;
        }

        // Push samples to ring buffer (lock-free)
        let pushed = self.buffer.push_samples(&samples_to_add);
        if pushed < samples_to_add.len() {
            // Buffer became full during push
            if !self.signaled.swap(true, Ordering::SeqCst) {
                if let Some(ref sender) = self.stop_signal {
                    let _ = sender.send(StopReason::BufferFull);
                }
            }
        }
    }
}

impl AudioCaptureBackend for CpalBackend {
    fn start(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<Sender<StopReason>>,
        device_name: Option<String>,
    ) -> Result<u32, AudioCaptureError> {
        info!("Starting audio capture (target: {}Hz)...", TARGET_SAMPLE_RATE);

        // Get the default audio host
        let host = cpal::default_host();
        debug!("Host: {:?}", host.id());

        // Find the requested device or fall back to default
        let device = if let Some(ref name) = device_name {
            match find_device_by_name(name) {
                Some(d) => {
                    info!("Using requested device: {}", name);
                    d
                }
                None => {
                    warn!(
                        "Requested device '{}' not found, falling back to default",
                        name
                    );
                    host.default_input_device().ok_or_else(|| {
                        error!("No input device available!");
                        AudioCaptureError::NoDeviceAvailable
                    })?
                }
            }
        } else {
            host.default_input_device().ok_or_else(|| {
                error!("No input device available!");
                AudioCaptureError::NoDeviceAvailable
            })?
        };
        debug!(
            "Input device: {:?}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        // Try to get a config with 16kHz sample rate, fall back to default
        let (config, needs_resampling) = if let Some(config_16k) = find_config_with_sample_rate(&device, TARGET_SAMPLE_RATE) {
            info!("Device supports {}Hz natively", TARGET_SAMPLE_RATE);
            (config_16k, false)
        } else {
            let default_config = device.default_input_config().map_err(|e| {
                error!("Failed to get input config: {}", e);
                AudioCaptureError::DeviceError(e.to_string())
            })?;
            warn!(
                "Device doesn't support {}Hz, will resample from {}Hz",
                TARGET_SAMPLE_RATE,
                default_config.sample_rate().0
            );
            (default_config, true)
        };

        let device_sample_rate = config.sample_rate().0;
        debug!(
            "Config: {} Hz, {:?}, {} channels",
            device_sample_rate,
            config.sample_format(),
            config.channels()
        );

        // Create resampler if needed
        let resampler: Option<Arc<Mutex<FftFixedIn<f32>>>> = if needs_resampling {
            // Use a chunk size suitable for real-time processing
            let chunk_size = 1024;
            let r = create_resampler(device_sample_rate, TARGET_SAMPLE_RATE, chunk_size)?;
            Some(Arc::new(Mutex::new(r)))
        } else {
            None
        };

        // Shared flag to ensure we only signal once
        let signaled = std::sync::Arc::new(AtomicBool::new(false));

        // Create error handler that signals stop on stream errors
        let err_signal = stop_signal.clone();
        let err_signaled = signaled.clone();
        let err_fn = move |err: cpal::StreamError| {
            error!("Audio stream error: {}", err);
            // Signal stop so recording doesn't continue with garbage data
            if !err_signaled.swap(true, Ordering::SeqCst) {
                if let Some(ref sender) = err_signal {
                    let _ = sender.send(StopReason::StreamError);
                }
            }
        };

        // Buffer for accumulating samples before resampling
        let resample_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let chunk_size = 1024usize;

        // Pre-allocated chunk buffer to avoid allocations in hot path
        let chunk_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0f32; chunk_size]));

        // Create shared callback state - all callbacks use the same processing logic
        let callback_state = Arc::new(CallbackState {
            buffer,
            stop_signal,
            signaled,
            resampler,
            resample_buffer,
            chunk_buffer,
            chunk_size,
        });

        // Build the input stream based on sample format
        // Each callback converts to f32 and delegates to CallbackState::process_samples
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let state = callback_state.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        // F32 samples are already in the correct format
                        state.process_samples(data);
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let state = callback_state.clone();
                device.build_input_stream(
                    &config.into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        // Convert i16 samples to f32 normalized to [-1.0, 1.0]
                        let f32_samples: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                        state.process_samples(&f32_samples);
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let state = callback_state;
                device.build_input_stream(
                    &config.into(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        // Convert u16 samples to f32 normalized to [-1.0, 1.0]
                        let f32_samples: Vec<f32> = data.iter().map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0).collect();
                        state.process_samples(&f32_samples);
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
            error!("Failed to build input stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        // Start the stream
        stream.play().map_err(|e| {
            error!("Failed to start stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        info!("Audio stream started successfully at {}Hz (output: {}Hz)", device_sample_rate, TARGET_SAMPLE_RATE);
        self.stream = Some(stream);
        self.state = CaptureState::Capturing;
        // Always return TARGET_SAMPLE_RATE since we resample if needed
        Ok(TARGET_SAMPLE_RATE)
    }

    fn stop(&mut self) -> Result<(), AudioCaptureError> {
        debug!("Stopping audio capture...");
        if let Some(stream) = self.stream.take() {
            // Stream will be dropped here, stopping capture
            drop(stream);
            debug!("Audio stream stopped");
        } else {
            debug!("No active stream to stop");
        }
        self.state = CaptureState::Stopped;
        Ok(())
    }
}

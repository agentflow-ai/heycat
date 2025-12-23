// cpal-based audio capture backend
// This code interacts with hardware and is excluded from coverage measurement
//
// Note: All impl blocks here are excluded from coverage because they
// interact with hardware and cannot be unit tested.
#![cfg_attr(coverage_nightly, coverage(off))]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, Stream, StreamConfig};
use rubato::{FftFixedIn, Resampler};

use super::{AudioBuffer, AudioCaptureBackend, AudioCaptureError, CaptureState, StopReason, MAX_RESAMPLE_BUFFER_SAMPLES, TARGET_SAMPLE_RATE};
use super::denoiser::{DtlnDenoiser, SharedDenoiser};
use super::preprocessing::PreprocessingChain;
use crate::audio_constants::{PREFERRED_BUFFER_SIZE, RESAMPLE_CHUNK_SIZE};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

/// Audio capture backend using cpal for platform-specific audio capture
pub struct CpalBackend {
    state: CaptureState,
    stream: Option<Stream>,
    /// Stores callback state reference for diagnostic logging on stop
    callback_state: Option<Arc<CallbackState>>,
}

impl CpalBackend {
    /// Create a new cpal backend
    pub fn new() -> Self {
        Self {
            state: CaptureState::Idle,
            stream: None,
            callback_state: None,
        }
    }

    /// Start audio capture with optional shared denoiser
    ///
    /// This is the main implementation. When a shared denoiser is provided,
    /// it's used directly (eliminating the ~2s model loading delay).
    /// Otherwise, falls back to loading the denoiser inline.
    ///
    /// # Arguments
    /// * `buffer` - Audio buffer to fill with captured samples
    /// * `stop_signal` - Optional channel to signal auto-stop events
    /// * `device_name` - Optional device name; falls back to default
    /// * `shared_denoiser` - Optional pre-loaded shared denoiser
    pub fn start_with_denoiser(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<Sender<StopReason>>,
        device_name: Option<String>,
        shared_denoiser: Option<Arc<SharedDenoiser>>,
    ) -> Result<u32, AudioCaptureError> {
        self.start_internal(buffer, stop_signal, device_name, shared_denoiser)
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

/// Get the effective buffer size, checking for environment variable override.
///
/// For troubleshooting audio issues, the buffer size can be overridden by setting
/// the `HEYCAT_AUDIO_BUFFER_SIZE` environment variable to a value between 64 and 2048.
/// If not set or invalid, uses `PREFERRED_BUFFER_SIZE` (256 samples).
fn get_effective_buffer_size() -> u32 {
    if let Ok(env_value) = std::env::var("HEYCAT_AUDIO_BUFFER_SIZE") {
        if let Ok(size) = env_value.parse::<u32>() {
            if (64..=2048).contains(&size) {
                crate::info!(
                    "Using buffer size {} from HEYCAT_AUDIO_BUFFER_SIZE environment variable",
                    size
                );
                return size;
            } else {
                crate::warn!(
                    "HEYCAT_AUDIO_BUFFER_SIZE={} is out of range (64-2048), using default {}",
                    size,
                    PREFERRED_BUFFER_SIZE
                );
            }
        } else {
            crate::warn!(
                "HEYCAT_AUDIO_BUFFER_SIZE='{}' is not a valid number, using default {}",
                env_value,
                PREFERRED_BUFFER_SIZE
            );
        }
    }
    PREFERRED_BUFFER_SIZE
}

/// Create a StreamConfig with the preferred buffer size.
///
/// Attempts to use `BufferSize::Fixed(buffer_size)` for consistent timing.
/// The actual buffer size used by the driver may differ from the requested size.
/// Buffer size can be overridden via `HEYCAT_AUDIO_BUFFER_SIZE` env var for troubleshooting.
fn create_stream_config_with_buffer_size(base_config: cpal::SupportedStreamConfig) -> (StreamConfig, u32) {
    let buffer_size = get_effective_buffer_size();
    let mut config: StreamConfig = base_config.into();
    config.buffer_size = BufferSize::Fixed(buffer_size);
    crate::info!(
        "Requesting buffer size: {} samples (~{:.1}ms at {}Hz)",
        buffer_size,
        buffer_size as f32 / config.sample_rate.0 as f32 * 1000.0,
        config.sample_rate.0
    );
    (config, buffer_size)
}

/// Mix multi-channel audio to mono.
///
/// Performs proper channel mixing with -3dB gain compensation to prevent clipping
/// when summing channels. For stereo: `mono = (left + right) / 2 * 0.707`.
///
/// # Arguments
/// * `samples` - Interleaved multi-channel samples (e.g., [L0, R0, L1, R1, ...])
/// * `channels` - Number of channels in the input
///
/// # Returns
/// Mono samples (one sample per frame)
fn mix_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels == 1 {
        // Already mono - return as-is
        return samples.to_vec();
    }

    let channels = channels as usize;
    let frame_count = samples.len() / channels;
    let mut mono = Vec::with_capacity(frame_count);

    // -3dB gain compensation when summing channels (sqrt(0.5) ≈ 0.707)
    // This prevents clipping when coherent signals are summed
    const GAIN_COMPENSATION: f32 = 0.7071067811865476; // 1/sqrt(2)

    for frame in 0..frame_count {
        let frame_start = frame * channels;
        let sum: f32 = samples[frame_start..frame_start + channels].iter().sum();
        let avg = sum / channels as f32;
        mono.push(avg * GAIN_COMPENSATION);
    }

    mono
}

/// Shared state for audio processing callback
/// Captures all the Arc-wrapped resources needed by the callback
struct CallbackState {
    buffer: AudioBuffer,
    stop_signal: Option<Sender<StopReason>>,
    signaled: Arc<AtomicBool>,
    /// Stop flag - when true, callbacks should not process new samples
    /// This prevents samples from being pushed to denoiser after stop is initiated
    stop_flag: Arc<AtomicBool>,
    resampler: Option<Arc<Mutex<FftFixedIn<f32>>>>,
    resample_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_size: usize,
    /// Tracks total input samples received from device (for diagnostic logging)
    input_sample_count: Arc<AtomicUsize>,
    /// Tracks total output samples after resampling (for diagnostic logging)
    output_sample_count: Arc<AtomicUsize>,
    /// Device sample rate (for ratio calculation in diagnostics)
    device_sample_rate: u32,
    /// Number of channels from the audio device (for stereo-to-mono mixing)
    channel_count: u16,
    /// Voice-optimized preprocessing (highpass + pre-emphasis)
    preprocessing: Arc<Mutex<PreprocessingChain>>,
    /// Optional noise suppression denoiser (None if failed to load)
    denoiser: Option<Arc<Mutex<DtlnDenoiser>>>,
}

impl CallbackState {
    /// Process f32 audio samples - handles channel mixing, resampling, and buffer management
    ///
    /// This is the core audio processing logic, extracted to avoid duplication
    /// across F32, I16, and U16 sample format callbacks.
    ///
    /// Processing order:
    /// 1. Channel mixing (stereo → mono)
    /// 2. Resampling (if needed)
    /// 3. Noise suppression (if enabled)
    /// 4. Ring buffer storage
    fn process_samples(&self, f32_samples: &[f32]) {
        // Check stop flag FIRST - don't process samples after stop is initiated
        // This prevents stale samples from corrupting the shared denoiser state
        if self.stop_flag.load(Ordering::SeqCst) {
            return;
        }

        // Track input samples for diagnostic logging (raw device samples)
        self.input_sample_count.fetch_add(f32_samples.len(), Ordering::Relaxed);

        // Step 1: Mix multi-channel to mono (if needed)
        // This must happen before resampling since resampler is configured for mono
        let mono_samples = if self.channel_count > 1 {
            mix_to_mono(f32_samples, self.channel_count)
        } else {
            f32_samples.to_vec()
        };

        // Step 2: Apply voice-optimized preprocessing (highpass + pre-emphasis)
        // This runs at device sample rate for best filter accuracy
        let preprocessed = match self.preprocessing.lock() {
            Ok(mut pp) => pp.process(&mono_samples),
            Err(_) => mono_samples, // Skip preprocessing if lock fails
        };

        // Step 3: Resample to target rate (if needed)
        let samples_to_add = if let Some(ref resampler) = self.resampler {
            // Accumulate samples and resample when we have enough
            let mut resample_buf = match self.resample_buffer.lock() {
                Ok(buf) => buf,
                Err(_) => return,
            };

            // Signal stop if resample buffer overflows - data loss is unacceptable
            if resample_buf.len() + preprocessed.len() > MAX_RESAMPLE_BUFFER_SAMPLES {
                crate::error!("Resample buffer overflow: resampling can't keep up with audio input");
                if !self.signaled.swap(true, Ordering::SeqCst) {
                    if let Some(ref sender) = self.stop_signal {
                        let _ = sender.send(StopReason::ResampleOverflow);
                    }
                }
                return;
            }
            resample_buf.extend_from_slice(&preprocessed);

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
            preprocessed
        };

        // Apply noise suppression if available
        let processed_samples = if let Some(ref denoiser) = self.denoiser {
            match denoiser.lock() {
                Ok(mut d) => d.process(&samples_to_add),
                Err(_) => samples_to_add, // Skip denoising if lock fails
            }
        } else {
            samples_to_add
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

        // Track output samples for diagnostic logging
        self.output_sample_count.fetch_add(processed_samples.len(), Ordering::Relaxed);

        // Push samples to ring buffer (lock-free)
        let pushed = self.buffer.push_samples(&processed_samples);
        if pushed < processed_samples.len() {
            // Buffer became full during push
            if !self.signaled.swap(true, Ordering::SeqCst) {
                if let Some(ref sender) = self.stop_signal {
                    let _ = sender.send(StopReason::BufferFull);
                }
            }
        }
    }

    /// Flush any remaining samples in the resample buffer, resampler delay buffer, and denoiser
    ///
    /// Called from stop() after the stream is dropped but before CallbackState is dropped.
    /// This ensures:
    /// 1. Residual samples that didn't fill a complete chunk are processed via process_partial
    /// 2. The resampler's internal delay buffer (output_delay frames) is flushed
    /// 3. Flushed resampler samples pass through the denoiser (if present)
    /// 4. The denoiser's internal buffers are flushed
    ///
    /// The FFT resampler holds samples internally during processing. Without flushing the delay
    /// buffer, each recording loses ~100-500 samples, causing progressive audio speedup.
    fn flush_residuals(&self) {
        let mut resampled_residuals = Vec::new();

        // Step 1: Flush resampler residuals (if we have a resampler)
        if let Some(ref resampler) = self.resampler {
            let mut resample_buf = match self.resample_buffer.lock() {
                Ok(buf) => buf,
                Err(_) => return,
            };

            if let Ok(mut r) = resampler.lock() {
                // Process any remaining samples using process_partial
                let residual_count = resample_buf.len();
                if residual_count > 0 {
                    crate::debug!("Flushing {} residual samples via process_partial", residual_count);
                    if let Ok(output) = r.process_partial(Some(&[resample_buf.as_slice()]), None) {
                        if !output.is_empty() && !output[0].is_empty() {
                            resampled_residuals.extend_from_slice(&output[0]);
                            crate::debug!("Residual flush produced {} output samples", output[0].len());
                        }
                    }
                    resample_buf.clear();
                }

                // Flush the resampler's internal delay buffer (CRITICAL)
                let delay = r.output_delay();
                crate::debug!("Flushing resampler delay buffer (delay={} frames)", delay);
                if let Ok(output) = r.process_partial(None::<&[&[f32]]>, None) {
                    if !output.is_empty() && !output[0].is_empty() {
                        resampled_residuals.extend_from_slice(&output[0]);
                        crate::debug!("Flushed {} samples from delay buffer", output[0].len());
                    }
                }
            }
        }

        // Step 2: Pass resampled residuals through denoiser (if present)
        if !resampled_residuals.is_empty() {
            if let Some(ref denoiser) = self.denoiser {
                if let Ok(mut d) = denoiser.lock() {
                    let processed = d.process(&resampled_residuals);
                    if !processed.is_empty() {
                        self.output_sample_count.fetch_add(processed.len(), Ordering::Relaxed);
                        self.buffer.push_samples(&processed);
                    }
                }
            } else {
                // No denoiser, push directly to buffer
                self.output_sample_count.fetch_add(resampled_residuals.len(), Ordering::Relaxed);
                self.buffer.push_samples(&resampled_residuals);
            }
        }

        // Step 3: Flush denoiser's internal buffers
        if let Some(ref denoiser) = self.denoiser {
            if let Ok(mut d) = denoiser.lock() {
                let flushed = d.flush();
                if !flushed.is_empty() {
                    self.output_sample_count.fetch_add(flushed.len(), Ordering::Relaxed);
                    self.buffer.push_samples(&flushed);
                }
            }
        }

        crate::info!("[FLUSH] complete: output_sample_count={}",
            self.output_sample_count.load(Ordering::Relaxed));
    }

    /// Log sample count diagnostics when recording stops
    fn log_sample_diagnostics(&self) {
        let input = self.input_sample_count.load(Ordering::Relaxed);
        let output = self.output_sample_count.load(Ordering::Relaxed);

        if input == 0 {
            crate::debug!("Sample diagnostics: No samples recorded");
            return;
        }

        let actual_ratio = output as f64 / input as f64;
        let expected_ratio = TARGET_SAMPLE_RATE as f64 / self.device_sample_rate as f64;
        let ratio_error = ((actual_ratio - expected_ratio) / expected_ratio * 100.0).abs();

        crate::info!(
            "Sample diagnostics: input={}, output={}, actual_ratio={:.6}, expected_ratio={:.6}, error={:.2}%",
            input, output, actual_ratio, expected_ratio, ratio_error
        );
    }
}

impl AudioCaptureBackend for CpalBackend {
    fn start(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<Sender<StopReason>>,
        device_name: Option<String>,
    ) -> Result<u32, AudioCaptureError> {
        // Delegate to internal implementation without shared denoiser
        self.start_internal(buffer, stop_signal, device_name, None)
    }

    fn stop(&mut self) -> Result<(), AudioCaptureError> {
        crate::info!("========================================");
        crate::info!("[STOP] RECORDING SESSION STOPPING");
        crate::info!("========================================");

        // Set stop flag FIRST - prevents callbacks from processing new samples
        // This must happen BEFORE dropping the stream to prevent race conditions
        if let Some(ref callback_state) = self.callback_state {
            callback_state.stop_flag.store(true, Ordering::SeqCst);
            crate::debug!("[STOP] Stop flag set - callbacks will no longer process samples");
        }

        // Now stop the stream - any in-flight callbacks will see stop_flag=true
        if let Some(stream) = self.stream.take() {
            // Stream will be dropped here, stopping capture
            drop(stream);
            // Brief wait for any in-flight callbacks to check the flag and exit
            std::thread::sleep(std::time::Duration::from_millis(10));
            crate::info!("[STOP] Audio stream stopped");
        } else {
            crate::debug!("No active stream to stop");
        }

        // Now flush any residual samples and log diagnostics
        // This must happen after stream is stopped but before callback_state is dropped
        if let Some(ref callback_state) = self.callback_state {
            callback_state.flush_residuals();
            callback_state.log_sample_diagnostics();
        }

        // Clear callback state
        self.callback_state = None;
        self.state = CaptureState::Stopped;
        Ok(())
    }
}

impl CpalBackend {
    /// Internal start implementation that accepts optional shared denoiser
    fn start_internal(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<Sender<StopReason>>,
        device_name: Option<String>,
        shared_denoiser: Option<Arc<SharedDenoiser>>,
    ) -> Result<u32, AudioCaptureError> {
        crate::info!("========================================");
        crate::info!("[START] NEW RECORDING SESSION STARTING");
        crate::info!("========================================");
        crate::info!("Starting audio capture (target: {}Hz)...", TARGET_SAMPLE_RATE);

        // Get the default audio host
        let host = cpal::default_host();
        crate::debug!("Host: {:?}", host.id());

        // Find the requested device or fall back to default
        let device = if let Some(ref name) = device_name {
            match find_device_by_name(name) {
                Some(d) => {
                    crate::info!("Using requested device: {}", name);
                    d
                }
                None => {
                    crate::warn!(
                        "Requested device '{}' not found, falling back to default",
                        name
                    );
                    host.default_input_device().ok_or_else(|| {
                        crate::error!("No input device available!");
                        AudioCaptureError::NoDeviceAvailable
                    })?
                }
            }
        } else {
            host.default_input_device().ok_or_else(|| {
                crate::error!("No input device available!");
                AudioCaptureError::NoDeviceAvailable
            })?
        };
        crate::debug!(
            "Input device: {:?}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        // Try to get a config with 16kHz sample rate, fall back to default
        let (config, needs_resampling) = if let Some(config_16k) = find_config_with_sample_rate(&device, TARGET_SAMPLE_RATE) {
            crate::info!("Device supports {}Hz natively", TARGET_SAMPLE_RATE);
            (config_16k, false)
        } else {
            let default_config = device.default_input_config().map_err(|e| {
                crate::error!("Failed to get input config: {}", e);
                AudioCaptureError::DeviceError(e.to_string())
            })?;
            crate::warn!(
                "Device doesn't support {}Hz, will resample from {}Hz",
                TARGET_SAMPLE_RATE,
                default_config.sample_rate().0
            );
            (default_config, true)
        };

        let device_sample_rate = config.sample_rate().0;
        let channel_count = config.channels();
        crate::debug!(
            "Config: {} Hz, {:?}, {} channels",
            device_sample_rate,
            config.sample_format(),
            channel_count
        );

        // Log channel mixing information
        if channel_count > 1 {
            crate::info!(
                "Device has {} channels - will mix to mono with -3dB gain compensation",
                channel_count
            );
        }

        // Create resampler if needed
        let resampler: Option<Arc<Mutex<FftFixedIn<f32>>>> = if needs_resampling {
            let r = create_resampler(device_sample_rate, TARGET_SAMPLE_RATE, RESAMPLE_CHUNK_SIZE)?;
            crate::info!(
                "Resampler created: {}Hz -> {}Hz, output_delay={} frames",
                device_sample_rate,
                TARGET_SAMPLE_RATE,
                r.output_delay()
            );
            Some(Arc::new(Mutex::new(r)))
        } else {
            None
        };

        // Shared flag to ensure we only signal once
        let signaled = std::sync::Arc::new(AtomicBool::new(false));

        // Clone stop_signal and signaled before moving into CallbackState
        // These clones are needed for the fallback error handlers in buffer size retry logic
        let stop_signal_for_state = stop_signal.clone();
        let signaled_for_state = signaled.clone();

        // Create error handler that signals stop on stream errors
        let err_signal = stop_signal.clone();
        let err_signaled = signaled.clone();
        let err_fn = move |err: cpal::StreamError| {
            crate::error!("Audio stream error: {}", err);
            // Signal stop so recording doesn't continue with garbage data
            if !err_signaled.swap(true, Ordering::SeqCst) {
                if let Some(ref sender) = err_signal {
                    let _ = sender.send(StopReason::StreamError);
                }
            }
        };

        // Buffer for accumulating samples before resampling
        let resample_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

        // Pre-allocated chunk buffer to avoid allocations in hot path
        let chunk_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(vec![0.0f32; RESAMPLE_CHUNK_SIZE]));

        // Initialize noise suppression denoiser
        // Use shared denoiser if provided, otherwise no denoising (user disabled or unavailable)
        let denoiser: Option<Arc<Mutex<DtlnDenoiser>>> = if let Some(shared) = shared_denoiser {
            // Reset the shared denoiser's LSTM states for this new recording
            shared.reset();
            Some(shared.inner())
        } else {
            // No denoiser - either disabled by user setting or shared denoiser unavailable
            crate::debug!("No denoiser - noise suppression disabled or unavailable");
            None
        };

        // Create preprocessing chain at device sample rate for best filter accuracy
        let preprocessing = PreprocessingChain::new(device_sample_rate);
        crate::info!(
            "Preprocessing chain initialized at {}Hz (highpass {}Hz, pre-emphasis α={})",
            device_sample_rate,
            crate::audio_constants::HIGHPASS_CUTOFF_HZ,
            crate::audio_constants::PRE_EMPHASIS_ALPHA
        );

        // Create shared callback state - all callbacks use the same processing logic
        let callback_state = Arc::new(CallbackState {
            buffer,
            stop_signal: stop_signal_for_state,
            signaled: signaled_for_state,
            stop_flag: Arc::new(AtomicBool::new(false)),
            resampler,
            resample_buffer,
            chunk_buffer,
            chunk_size: RESAMPLE_CHUNK_SIZE,
            input_sample_count: Arc::new(AtomicUsize::new(0)),
            output_sample_count: Arc::new(AtomicUsize::new(0)),
            device_sample_rate,
            channel_count,
            preprocessing: Arc::new(Mutex::new(preprocessing)),
            denoiser,
        });

        // Create stream config with preferred buffer size
        let (stream_config, effective_buffer_size) = create_stream_config_with_buffer_size(config.clone());
        let sample_format = config.sample_format();

        // Build the input stream based on sample format
        // Each callback converts to f32 and delegates to CallbackState::process_samples
        // Try with fixed buffer size first, fall back to default if platform rejects it
        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                // Try fixed buffer size first
                let state = callback_state.clone();
                let err_signal = stop_signal.clone();
                let err_signaled = signaled.clone();
                let result = device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        state.process_samples(data);
                    },
                    move |err: cpal::StreamError| {
                        crate::error!("Audio stream error: {}", err);
                        if !err_signaled.swap(true, Ordering::SeqCst) {
                            if let Some(ref sender) = err_signal {
                                let _ = sender.send(StopReason::StreamError);
                            }
                        }
                    },
                    None,
                );
                match result {
                    Ok(stream) => {
                        crate::info!("Stream created with fixed buffer size: {} samples", effective_buffer_size);
                        Ok(stream)
                    }
                    Err(e) => {
                        crate::warn!(
                            "Fixed buffer size {} rejected ({}), falling back to platform default",
                            effective_buffer_size, e
                        );
                        let default_config: StreamConfig = config.clone().into();
                        let state = callback_state.clone();
                        device.build_input_stream(
                            &default_config,
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                state.process_samples(data);
                            },
                            err_fn,
                            None,
                        )
                    }
                }
            }
            cpal::SampleFormat::I16 => {
                // Try fixed buffer size first
                let state = callback_state.clone();
                let err_signal = stop_signal.clone();
                let err_signaled = signaled.clone();
                let result = device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let f32_samples: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                        state.process_samples(&f32_samples);
                    },
                    move |err: cpal::StreamError| {
                        crate::error!("Audio stream error: {}", err);
                        if !err_signaled.swap(true, Ordering::SeqCst) {
                            if let Some(ref sender) = err_signal {
                                let _ = sender.send(StopReason::StreamError);
                            }
                        }
                    },
                    None,
                );
                match result {
                    Ok(stream) => {
                        crate::info!("Stream created with fixed buffer size: {} samples", effective_buffer_size);
                        Ok(stream)
                    }
                    Err(e) => {
                        crate::warn!(
                            "Fixed buffer size {} rejected ({}), falling back to platform default",
                            effective_buffer_size, e
                        );
                        let default_config: StreamConfig = config.clone().into();
                        let state = callback_state.clone();
                        device.build_input_stream(
                            &default_config,
                            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                                let f32_samples: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                                state.process_samples(&f32_samples);
                            },
                            err_fn,
                            None,
                        )
                    }
                }
            }
            cpal::SampleFormat::U16 => {
                // Try fixed buffer size first
                let state = callback_state.clone();
                let err_signal = stop_signal.clone();
                let err_signaled = signaled.clone();
                let result = device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let f32_samples: Vec<f32> = data.iter().map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0).collect();
                        state.process_samples(&f32_samples);
                    },
                    move |err: cpal::StreamError| {
                        crate::error!("Audio stream error: {}", err);
                        if !err_signaled.swap(true, Ordering::SeqCst) {
                            if let Some(ref sender) = err_signal {
                                let _ = sender.send(StopReason::StreamError);
                            }
                        }
                    },
                    None,
                );
                match result {
                    Ok(stream) => {
                        crate::info!("Stream created with fixed buffer size: {} samples", effective_buffer_size);
                        Ok(stream)
                    }
                    Err(e) => {
                        crate::warn!(
                            "Fixed buffer size {} rejected ({}), falling back to platform default",
                            effective_buffer_size, e
                        );
                        let default_config: StreamConfig = config.clone().into();
                        let state = callback_state.clone();
                        device.build_input_stream(
                            &default_config,
                            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                                let f32_samples: Vec<f32> = data.iter().map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0).collect();
                                state.process_samples(&f32_samples);
                            },
                            err_fn,
                            None,
                        )
                    }
                }
            }
            _ => {
                return Err(AudioCaptureError::DeviceError(
                    "Unsupported sample format".to_string(),
                ))
            }
        }
        .map_err(|e| {
            crate::error!("Failed to build input stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        // Start the stream
        stream.play().map_err(|e| {
            crate::error!("Failed to start stream: {}", e);
            AudioCaptureError::StreamError(e.to_string())
        })?;

        crate::info!("Audio stream started successfully at {}Hz (output: {}Hz)", device_sample_rate, TARGET_SAMPLE_RATE);
        self.stream = Some(stream);
        self.callback_state = Some(callback_state);
        self.state = CaptureState::Capturing;
        // Always return TARGET_SAMPLE_RATE since we resample if needed
        Ok(TARGET_SAMPLE_RATE)
    }
}

#[cfg(test)]
mod resampler_tests {
    use super::*;
    use rubato::Resampler;

    const SOURCE_RATE: usize = 48000;
    const TARGET_RATE: usize = 16000;
    const CHUNK_SIZE: usize = RESAMPLE_CHUNK_SIZE;

    /// Test that the FFT resampler eventually produces output after enough input.
    /// The FFT resampler has internal latency and may not produce output on every call.
    #[test]
    fn test_resampler_produces_output_after_warmup() {
        let mut resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        let mut total_output = 0usize;

        // Process multiple chunks to warm up the resampler
        for _ in 0..10 {
            let chunk: Vec<f32> = vec![0.5f32; CHUNK_SIZE];
            let output = resampler.process(&[&chunk], None).unwrap();
            total_output += output[0].len();
        }

        // After enough input, we should have some output
        assert!(
            total_output > 0,
            "Resampler should produce output after processing multiple chunks"
        );
    }

    /// Test that sample ratio converges to expected value over many chunks.
    /// The FFT resampler has internal latency that can cause ratio drift on small samples.
    #[test]
    fn test_sample_ratio_converges() {
        let mut resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();
        let expected_ratio = TARGET_RATE as f64 / SOURCE_RATE as f64;

        let mut total_input = 0usize;
        let mut total_output = 0usize;

        // Process many chunks to get past initial latency effects
        for _ in 0..100 {
            let chunk: Vec<f32> = vec![0.5f32; CHUNK_SIZE];
            let output = resampler.process(&[&chunk], None).unwrap();

            total_input += CHUNK_SIZE;
            total_output += output[0].len();
        }

        // With enough samples, ratio should be close to expected
        // Allow 5% tolerance due to FFT windowing effects
        let actual_ratio = total_output as f64 / total_input as f64;
        let ratio_error = ((actual_ratio - expected_ratio) / expected_ratio * 100.0).abs();
        assert!(
            ratio_error < 5.0,
            "Ratio error {:.2}% exceeds 5%",
            ratio_error
        );
    }

    /// Test edge case: flush when buffer is empty (should be a no-op)
    #[test]
    fn test_flush_with_empty_buffer() {
        let resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        // Create CallbackState with empty resample buffer
        let callback_state = CallbackState {
            buffer: AudioBuffer::new(),
            stop_signal: None,
            signaled: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            resampler: Some(Arc::new(Mutex::new(resampler))),
            resample_buffer: Arc::new(Mutex::new(Vec::new())),
            chunk_buffer: Arc::new(Mutex::new(vec![0.0f32; CHUNK_SIZE])),
            chunk_size: CHUNK_SIZE,
            input_sample_count: Arc::new(AtomicUsize::new(0)),
            output_sample_count: Arc::new(AtomicUsize::new(0)),
            device_sample_rate: SOURCE_RATE as u32,
            channel_count: 1, // Tests use mono
            preprocessing: Arc::new(Mutex::new(PreprocessingChain::new(SOURCE_RATE as u32))),
            denoiser: None,
        };

        // Flush should be a no-op when buffer is empty
        callback_state.flush_residuals();

        // Verify no samples were output
        assert_eq!(
            callback_state.output_sample_count.load(Ordering::Relaxed),
            0
        );
    }

    /// Test that resample buffer is cleared after flush
    #[test]
    fn test_buffer_cleared_after_flush() {
        // First warm up the resampler so it has internal state
        let mut resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        // Process several chunks to warm up
        for _ in 0..5 {
            let chunk: Vec<f32> = vec![0.5f32; CHUNK_SIZE];
            let _ = resampler.process(&[&chunk], None).unwrap();
        }

        // Create CallbackState with residual samples in buffer
        let residual_samples: Vec<f32> = vec![0.5f32; 500];
        let callback_state = CallbackState {
            buffer: AudioBuffer::new(),
            stop_signal: None,
            signaled: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            resampler: Some(Arc::new(Mutex::new(resampler))),
            resample_buffer: Arc::new(Mutex::new(residual_samples)),
            chunk_buffer: Arc::new(Mutex::new(vec![0.0f32; CHUNK_SIZE])),
            chunk_size: CHUNK_SIZE,
            input_sample_count: Arc::new(AtomicUsize::new(5 * CHUNK_SIZE + 500)),
            output_sample_count: Arc::new(AtomicUsize::new(0)),
            device_sample_rate: SOURCE_RATE as u32,
            channel_count: 1, // Tests use mono
            preprocessing: Arc::new(Mutex::new(PreprocessingChain::new(SOURCE_RATE as u32))),
            denoiser: None,
        };

        // Flush the residual samples
        callback_state.flush_residuals();

        // Verify buffer is now empty
        let resample_buf = callback_state.resample_buffer.lock().unwrap();
        assert!(
            resample_buf.is_empty(),
            "Resample buffer should be empty after flush"
        );

        // The flush should process the residual samples through the resampler.
        // Due to FFT latency, we may or may not get output, so just verify the buffer is cleared.
    }

    /// Test that flush processes residuals without panicking
    #[test]
    fn test_flush_residuals_does_not_panic() {
        // Test with various residual sizes
        for residual_size in [1, 100, 500, 1023] {
            let residual_samples: Vec<f32> = vec![0.5f32; residual_size];
            let callback_state = CallbackState {
                buffer: AudioBuffer::new(),
                stop_signal: None,
                signaled: Arc::new(AtomicBool::new(false)),
                stop_flag: Arc::new(AtomicBool::new(false)),
                resampler: Some(Arc::new(Mutex::new(
                    FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap(),
                ))),
                resample_buffer: Arc::new(Mutex::new(residual_samples)),
                chunk_buffer: Arc::new(Mutex::new(vec![0.0f32; CHUNK_SIZE])),
                chunk_size: CHUNK_SIZE,
                input_sample_count: Arc::new(AtomicUsize::new(residual_size)),
                output_sample_count: Arc::new(AtomicUsize::new(0)),
                device_sample_rate: SOURCE_RATE as u32,
                channel_count: 1, // Tests use mono
                preprocessing: Arc::new(Mutex::new(PreprocessingChain::new(SOURCE_RATE as u32))),
                denoiser: None,
            };

            // Should not panic
            callback_state.flush_residuals();

            // Buffer should be cleared
            let resample_buf = callback_state.resample_buffer.lock().unwrap();
            assert!(
                resample_buf.is_empty(),
                "Buffer should be empty after flush with {} residuals",
                residual_size
            );
        }
    }

    /// Test that process_partial(None) extracts samples from the delay buffer.
    /// The FFT resampler holds output_delay() frames internally that must be flushed.
    #[test]
    fn test_process_partial_extracts_delay_buffer() {
        let mut resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        // The resampler has an internal delay buffer
        let delay = resampler.output_delay();
        assert!(delay > 0, "Resampler should have non-zero output delay");

        // Process some chunks to fill the internal state
        let mut total_output = 0usize;
        for _ in 0..10 {
            let chunk: Vec<f32> = vec![0.5f32; CHUNK_SIZE];
            let output = resampler.process(&[&chunk], None).unwrap();
            total_output += output[0].len();
        }

        // Now flush with process_partial(None) - this should extract remaining samples
        let flush_output = resampler.process_partial(None::<&[&[f32]]>, None).unwrap();
        let flushed_samples = if !flush_output.is_empty() {
            flush_output[0].len()
        } else {
            0
        };

        // After flushing, we should have gotten some samples from the delay buffer
        // The exact number depends on the resampler's internal state, but it should be > 0
        assert!(
            flushed_samples > 0,
            "process_partial(None) should extract samples from delay buffer, got {}",
            flushed_samples
        );
    }

    /// Test that sample ratio converges toward expected value after proper flushing.
    /// With proper flushing, the ratio error should be lower than without flushing.
    ///
    /// Note: Due to FFT resampler internal buffering characteristics, the exact ratio
    /// depends on total sample count. The key behavior is that flushing extracts
    /// additional samples that would otherwise be lost.
    #[test]
    fn test_sample_ratio_improves_with_flush() {
        // Create two resamplers to compare with vs without flushing
        let mut resampler_with_flush =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();
        let mut resampler_without_flush =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        let expected_ratio = TARGET_RATE as f64 / SOURCE_RATE as f64;

        let mut total_input = 0usize;
        let mut output_with_flush = 0usize;
        let mut output_without_flush = 0usize;

        // Process chunks (simulating a recording)
        for _ in 0..100 {
            let chunk: Vec<f32> = vec![0.5f32; CHUNK_SIZE];
            let out1 = resampler_with_flush.process(&[&chunk], None).unwrap();
            let out2 = resampler_without_flush.process(&[&chunk], None).unwrap();
            total_input += CHUNK_SIZE;
            output_with_flush += out1[0].len();
            output_without_flush += out2[0].len();
        }

        // Add some residual samples via process_partial (for with_flush)
        let residual: Vec<f32> = vec![0.5f32; 500];
        total_input += 500;
        let residual_output = resampler_with_flush.process_partial(Some(&[&residual[..]]), None).unwrap();
        if !residual_output.is_empty() {
            output_with_flush += residual_output[0].len();
        }

        // Flush the delay buffer with process_partial(None)
        let flush_output = resampler_with_flush.process_partial(None::<&[&[f32]]>, None).unwrap();
        if !flush_output.is_empty() {
            output_with_flush += flush_output[0].len();
        }

        // Calculate ratio errors
        let ratio_with_flush = output_with_flush as f64 / total_input as f64;
        let ratio_without_flush = output_without_flush as f64 / (total_input - 500) as f64; // without residual

        let error_with_flush = ((ratio_with_flush - expected_ratio) / expected_ratio * 100.0).abs();
        let error_without_flush = ((ratio_without_flush - expected_ratio) / expected_ratio * 100.0).abs();

        // The key assertion: flushing should get us more output samples
        // (or at least not make things worse)
        assert!(
            output_with_flush > output_without_flush,
            "Flushing should produce more output samples: with_flush={}, without_flush={}",
            output_with_flush,
            output_without_flush
        );

        // The ratio with flushing should be at least as good or better
        // (Note: exact tolerance depends on sample count, so we just verify improvement)
        assert!(
            error_with_flush <= error_without_flush + 0.5,
            "Flushing should not significantly degrade ratio: with_flush error={:.3}%, without_flush error={:.3}%",
            error_with_flush,
            error_without_flush
        );
    }

    // =========================================================================
    // Regression tests for stop flag (audio-glitch.bug.md fix)
    // =========================================================================

    /// Regression test: stop_flag prevents sample processing
    ///
    /// Bug: Audio callbacks continued processing after stop() was called,
    /// pushing stale samples to the SharedDenoiser and corrupting LSTM state.
    ///
    /// Fix: Added stop_flag atomic that callbacks check before processing.
    #[test]
    fn test_stop_flag_prevents_sample_processing() {
        let resampler =
            FftFixedIn::<f32>::new(SOURCE_RATE, TARGET_RATE, CHUNK_SIZE, 1, 1).unwrap();

        let callback_state = CallbackState {
            buffer: AudioBuffer::new(),
            stop_signal: None,
            signaled: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            resampler: Some(Arc::new(Mutex::new(resampler))),
            resample_buffer: Arc::new(Mutex::new(Vec::new())),
            chunk_buffer: Arc::new(Mutex::new(vec![0.0f32; CHUNK_SIZE])),
            chunk_size: CHUNK_SIZE,
            input_sample_count: Arc::new(AtomicUsize::new(0)),
            output_sample_count: Arc::new(AtomicUsize::new(0)),
            device_sample_rate: SOURCE_RATE as u32,
            channel_count: 1, // Tests use mono
            preprocessing: Arc::new(Mutex::new(PreprocessingChain::new(SOURCE_RATE as u32))),
            denoiser: None,
        };

        // Process some samples normally
        let samples: Vec<f32> = vec![0.5f32; 1000];
        callback_state.process_samples(&samples);

        let input_before_stop = callback_state.input_sample_count.load(Ordering::Relaxed);
        assert_eq!(input_before_stop, 1000, "Should have processed 1000 samples");

        // Set the stop flag
        callback_state.stop_flag.store(true, Ordering::SeqCst);

        // Try to process more samples - should be ignored
        callback_state.process_samples(&samples);

        let input_after_stop = callback_state.input_sample_count.load(Ordering::Relaxed);
        assert_eq!(
            input_after_stop, 1000,
            "Stop flag should prevent further sample processing"
        );
    }

    // =========================================================================
    // Channel mixing tests
    // =========================================================================

    /// Test that mono input (1 channel) passes through unchanged
    #[test]
    fn test_mix_to_mono_preserves_mono() {
        let mono_samples = vec![0.5f32, -0.3, 0.8, -0.1];
        let result = mix_to_mono(&mono_samples, 1);

        assert_eq!(result.len(), mono_samples.len());
        assert_eq!(result, mono_samples);
    }

    /// Test that stereo input (2 channels) is correctly mixed to mono
    #[test]
    fn test_mix_to_mono_stereo_mixing() {
        // Interleaved stereo: [L0, R0, L1, R1, ...]
        let stereo_samples = vec![0.5f32, 0.5, -0.3, -0.3, 0.8, 0.8, -0.1, -0.1];
        let result = mix_to_mono(&stereo_samples, 2);

        // Should have half the samples (one per frame)
        assert_eq!(result.len(), 4);

        // When L == R, the result should be L * 0.707 (gain compensation)
        let expected_gain = 0.7071067811865476f32;
        assert!((result[0] - 0.5 * expected_gain).abs() < 0.0001);
        assert!((result[1] - (-0.3) * expected_gain).abs() < 0.0001);
        assert!((result[2] - 0.8 * expected_gain).abs() < 0.0001);
        assert!((result[3] - (-0.1) * expected_gain).abs() < 0.0001);
    }

    /// Test that stereo 0dB sine wave results in approximately -3dB mono output
    #[test]
    fn test_mix_to_mono_gain_compensation() {
        // Full-scale stereo signal (both channels at 1.0)
        let stereo_samples = vec![1.0f32, 1.0];
        let result = mix_to_mono(&stereo_samples, 2);

        assert_eq!(result.len(), 1);

        // -3dB = 0.707..., so (1.0 + 1.0) / 2 * 0.707 ≈ 0.707
        let expected = 0.7071067811865476f32;
        assert!(
            (result[0] - expected).abs() < 0.0001,
            "Expected ~{}, got {}",
            expected,
            result[0]
        );
    }

    /// Test that multi-channel input (4+ channels) is handled without panics
    #[test]
    fn test_mix_to_mono_multichannel() {
        // 4-channel audio: [Ch0, Ch1, Ch2, Ch3, Ch0, Ch1, Ch2, Ch3, ...]
        let multichannel_samples = vec![
            0.25f32, 0.25, 0.25, 0.25, // Frame 0: all channels at 0.25
            0.5, 0.5, 0.5, 0.5, // Frame 1: all channels at 0.5
        ];
        let result = mix_to_mono(&multichannel_samples, 4);

        // Should have 2 output samples (one per frame)
        assert_eq!(result.len(), 2);

        // Average of four 0.25 values = 0.25, with -3dB gain = 0.25 * 0.707 ≈ 0.177
        let expected_frame0 = 0.25 * 0.7071067811865476f32;
        let expected_frame1 = 0.5 * 0.7071067811865476f32;
        assert!(
            (result[0] - expected_frame0).abs() < 0.0001,
            "Frame 0: expected ~{}, got {}",
            expected_frame0,
            result[0]
        );
        assert!(
            (result[1] - expected_frame1).abs() < 0.0001,
            "Frame 1: expected ~{}, got {}",
            expected_frame1,
            result[1]
        );
    }

    /// Test that mixed output maintains correct sample count (input_samples / channels)
    #[test]
    fn test_mix_to_mono_sample_count() {
        // 100 stereo frames = 200 samples
        let stereo_samples: Vec<f32> = (0..200).map(|i| (i as f32) * 0.001).collect();
        let result = mix_to_mono(&stereo_samples, 2);
        assert_eq!(result.len(), 100);

        // 100 mono frames = 100 samples
        let mono_samples: Vec<f32> = (0..100).map(|i| (i as f32) * 0.001).collect();
        let result = mix_to_mono(&mono_samples, 1);
        assert_eq!(result.len(), 100);

        // 100 quad-channel frames = 400 samples
        let quad_samples: Vec<f32> = (0..400).map(|i| (i as f32) * 0.001).collect();
        let result = mix_to_mono(&quad_samples, 4);
        assert_eq!(result.len(), 100);
    }
}

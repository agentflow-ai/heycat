// Audio capture module for microphone recording

use ringbuf::{
    traits::{Consumer, Observer, Producer, Split},
    HeapRb,
};
use std::sync::{Arc, Mutex};

mod cpal_backend;
pub use cpal_backend::CpalBackend;

mod device;
pub use device::{list_input_devices, AudioInputDevice};

mod error;
pub use error::AudioDeviceError;

pub mod monitor;
pub use monitor::AudioMonitorHandle;

pub mod thread;
pub use thread::AudioThreadHandle;

pub mod wav;
pub use wav::{encode_wav, parse_duration_from_file, SystemFileWriter};

pub mod denoiser;
pub use denoiser::SharedDenoiser;

pub mod preprocessing;
pub use preprocessing::PreprocessingChain;

#[cfg(test)]
mod mod_test;

#[cfg(test)]
mod wav_test;

/// Thread-safe buffer for storing audio samples using lock-free ring buffer
///
/// Uses a SPSC ring buffer for low-contention audio capture:
/// - Producer (audio callback) writes via `push_samples()` - lock-free
/// - Consumer (detection loop) reads via `drain_samples()` - lock-free
/// - Accumulated samples are stored for WAV encoding
pub struct AudioBuffer {
    /// Ring buffer producer for lock-free writes
    producer: Arc<Mutex<RingProducer>>,
    /// Ring buffer consumer for lock-free reads
    consumer: Arc<Mutex<RingConsumer>>,
    /// Accumulated samples for WAV encoding (populated by drain_samples)
    accumulated: Arc<Mutex<Vec<f32>>>,
}

impl AudioBuffer {
    /// Create a new empty audio buffer with default capacity
    pub fn new() -> Self {
        Self::with_capacity(MAX_BUFFER_SAMPLES)
    }

    /// Create a new audio buffer with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let rb = HeapRb::<f32>::new(capacity);
        let (producer, consumer) = rb.split();
        Self {
            producer: Arc::new(Mutex::new(producer)),
            consumer: Arc::new(Mutex::new(consumer)),
            accumulated: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Push samples to the buffer (used by audio callback)
    ///
    /// Returns the number of samples actually written.
    /// If buffer is full, returns 0.
    pub fn push_samples(&self, samples: &[f32]) -> usize {
        match self.producer.lock() {
            Ok(mut prod) => prod.push_slice(samples),
            Err(_) => 0,
        }
    }

    /// Drain available samples from ring buffer into accumulated storage
    ///
    /// Returns a copy of the newly drained samples.
    /// This should be called periodically by the consumer.
    pub fn drain_samples(&self) -> Vec<f32> {
        let mut drained = Vec::new();

        // Read from ring buffer
        if let Ok(mut cons) = self.consumer.lock() {
            let available = cons.occupied_len();
            if available > 0 {
                drained.resize(available, 0.0);
                cons.pop_slice(&mut drained);
            }
        }

        // Accumulate for WAV encoding
        if !drained.is_empty() {
            if let Ok(mut acc) = self.accumulated.lock() {
                acc.extend_from_slice(&drained);
            }
        }

        drained
    }

    /// Get accumulated sample count (for buffer full detection)
    pub fn accumulated_len(&self) -> usize {
        self.accumulated.lock().map(|a| a.len()).unwrap_or(0)
    }

    /// Get remaining capacity before buffer is full
    #[allow(dead_code)]
    pub fn remaining_capacity(&self) -> usize {
        MAX_BUFFER_SAMPLES.saturating_sub(self.accumulated_len())
    }

    /// Check if buffer has reached maximum capacity
    pub fn is_full(&self) -> bool {
        self.accumulated_len() >= MAX_BUFFER_SAMPLES
    }

    /// Lock the accumulated buffer for direct access (WAV encoding, etc.)
    ///
    /// Note: This only accesses accumulated samples, not samples still in ring buffer.
    /// Call `drain_samples()` first to ensure all samples are accumulated.
    pub fn lock(&self) -> std::sync::LockResult<std::sync::MutexGuard<'_, Vec<f32>>> {
        self.accumulated.lock()
    }
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for AudioBuffer {
    fn clone(&self) -> Self {
        Self {
            producer: Arc::clone(&self.producer),
            consumer: Arc::clone(&self.consumer),
            accumulated: Arc::clone(&self.accumulated),
        }
    }
}

impl std::fmt::Debug for AudioBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBuffer")
            .field("accumulated_len", &self.accumulated_len())
            .finish()
    }
}

/// Type alias for ring buffer producer half
type RingProducer = ringbuf::HeapProd<f32>;

/// Type alias for ring buffer consumer half
type RingConsumer = ringbuf::HeapCons<f32>;

/// Target sample rate for audio capture (16 kHz for speech recognition models)
pub const TARGET_SAMPLE_RATE: u32 = 16000;

/// Maximum buffer size in samples (~10 minutes at 16kHz = 9.6M samples)
/// This prevents unlimited memory growth during long recordings.
/// At 16kHz mono, this is approximately 38MB of f32 data.
pub const MAX_BUFFER_SAMPLES: usize = 16000 * 60 * 10;

/// Maximum resampling buffer size in samples (~3 seconds at 48kHz)
/// This limits memory growth if resampling can't keep up with input rate.
/// Typically source rates are 44.1kHz or 48kHz, so 3 seconds = ~144k samples.
pub const MAX_RESAMPLE_BUFFER_SAMPLES: usize = 48000 * 3;

/// State of the audio capture process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureState {
    /// Not capturing audio
    Idle,
    /// Actively capturing audio
    Capturing,
    /// Capture stopped (audio data available)
    Stopped,
}

impl Default for CaptureState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Errors that can occur during audio capture
#[derive(Debug, Clone, PartialEq)]
pub enum AudioCaptureError {
    /// No audio input device is available
    NoDeviceAvailable,
    /// Error with the audio device
    DeviceError(String),
    /// Error with the audio stream
    StreamError(String),
}

impl std::fmt::Display for AudioCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioCaptureError::NoDeviceAvailable => write!(f, "No audio input device available"),
            AudioCaptureError::DeviceError(msg) => write!(f, "Audio device error: {}", msg),
            AudioCaptureError::StreamError(msg) => write!(f, "Audio stream error: {}", msg),
        }
    }
}

impl std::error::Error for AudioCaptureError {}

/// Reason why audio capture was stopped automatically
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum StopReason {
    /// Buffer reached maximum capacity (~10 minutes)
    BufferFull,
    /// Lock poisoning error in audio callback (legacy, kept for serialization compatibility)
    #[allow(dead_code)]
    LockError,
    /// Audio stream error (device disconnected, etc.)
    StreamError,
    /// Resample buffer overflow (resampling can't keep up)
    ResampleOverflow,
    /// Silence detected after speech (user finished talking)
    #[allow(dead_code)] // Used by silence detection in listening module
    SilenceAfterSpeech,
    /// No speech detected after wake word (false activation timeout)
    #[allow(dead_code)] // Used by silence detection in listening module
    NoSpeechTimeout,
}

/// Trait for audio capture backends (allows mocking in tests)
pub trait AudioCaptureBackend {
    /// Start capturing audio into the provided buffer
    /// Returns the actual sample rate of the audio device
    ///
    /// # Arguments
    /// * `buffer` - The audio buffer to capture samples into
    /// * `stop_signal` - Optional sender to signal stop (e.g., buffer full, lock error)
    /// * `device_name` - Optional device name to use; falls back to default if not found
    ///
    /// Note: Production code uses `CpalBackend::start_with_denoiser()` directly for
    /// SharedDenoiser support. This trait method is kept for API completeness and
    /// future mock implementations.
    #[allow(dead_code)]
    fn start(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<std::sync::mpsc::Sender<StopReason>>,
        device_name: Option<String>,
    ) -> Result<u32, AudioCaptureError>;

    /// Stop capturing audio
    fn stop(&mut self) -> Result<(), AudioCaptureError>;
}

//! Core DTLN denoiser implementation
//!
//! Processes audio frames through the two-stage DTLN pipeline:
//! 1. Magnitude masking in frequency domain (Model 1)
//! 2. Time-domain refinement (Model 2)

use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;
use tract_onnx::prelude::*;

use super::{DtlnModels, RunnableModel};

/// Frame size for DTLN processing (32ms at 16kHz)
pub const FRAME_SIZE: usize = 512;

/// Frame shift / hop size (8ms at 16kHz, 75% overlap)
pub const FRAME_SHIFT: usize = 128;

/// Number of FFT bins (FRAME_SIZE / 2 + 1 for real FFT)
pub const FFT_BINS: usize = FRAME_SIZE / 2 + 1; // 257

/// LSTM hidden state size (determined by model architecture)
const LSTM_UNITS: usize = 128;

/// DTLN real-time denoiser
///
/// Processes audio in chunks, maintaining LSTM states between frames
/// for temporal continuity. Uses overlap-add for smooth output.
pub struct DtlnDenoiser {
    /// Model 1: Magnitude masking (frequency domain)
    model_1: RunnableModel,
    /// Model 2: Time-domain refinement
    model_2: RunnableModel,

    /// Input frame buffer (accumulates samples until full frame)
    input_buffer: Vec<f32>,
    /// Output buffer for overlap-add
    output_buffer: Vec<f32>,

    /// Combined LSTM state for Model 1 (shape: 1, 2, 128, 2)
    /// Contains both hidden and cell states for the LSTM
    state_1: Tensor,
    /// Combined LSTM state for Model 2 (shape: 1, 2, 128, 2)
    state_2: Tensor,

    /// Hann window coefficients
    window: Vec<f32>,

    /// FFT planner for forward FFT
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    /// FFT planner for inverse FFT
    ifft: std::sync::Arc<dyn rustfft::Fft<f32>>,
}

impl DtlnDenoiser {
    /// Create a new DTLN denoiser from loaded models
    ///
    /// # Arguments
    /// * `models` - Pre-loaded DTLN ONNX models
    ///
    /// # Returns
    /// * `DtlnDenoiser` ready to process audio
    pub fn new(models: DtlnModels) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FRAME_SIZE);
        let ifft = planner.plan_fft_inverse(FRAME_SIZE);

        Self {
            model_1: models.model_1,
            model_2: models.model_2,
            input_buffer: Vec::with_capacity(FRAME_SIZE),
            output_buffer: vec![0.0; FRAME_SIZE],
            state_1: Self::zeros_lstm_state(),
            state_2: Self::zeros_lstm_state(),
            window: Self::hann_window(FRAME_SIZE),
            fft,
            ifft,
        }
    }

    /// Process audio samples through the denoiser
    ///
    /// Accumulates input samples and produces denoised output when enough
    /// samples are available for frame processing.
    ///
    /// # Arguments
    /// * `samples` - Input audio samples (16kHz mono f32)
    ///
    /// # Returns
    /// * Denoised audio samples (may be fewer than input due to latency)
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        let mut output = Vec::new();

        // Add samples to input buffer
        self.input_buffer.extend_from_slice(samples);

        // Process complete frames
        while self.input_buffer.len() >= FRAME_SIZE {
            // Extract frame
            let frame: Vec<f32> = self.input_buffer[..FRAME_SIZE].to_vec();

            // Process frame through DTLN pipeline
            let processed = self.process_frame(&frame);

            // Overlap-add: add processed frame to output buffer
            for (i, &sample) in processed.iter().enumerate() {
                self.output_buffer[i] += sample;
            }

            // Extract output samples (first FRAME_SHIFT samples are ready)
            output.extend_from_slice(&self.output_buffer[..FRAME_SHIFT]);

            // Shift output buffer
            self.output_buffer.copy_within(FRAME_SHIFT.., 0);
            for i in (FRAME_SIZE - FRAME_SHIFT)..FRAME_SIZE {
                self.output_buffer[i] = 0.0;
            }

            // Shift input buffer by hop size
            self.input_buffer.drain(..FRAME_SHIFT);
        }

        output
    }

    /// Process a single frame through the DTLN pipeline
    fn process_frame(&mut self, frame: &[f32]) -> Vec<f32> {
        // Apply window
        let windowed: Vec<f32> = frame
            .iter()
            .zip(self.window.iter())
            .map(|(&s, &w)| s * w)
            .collect();

        // FFT
        let mut fft_buffer: Vec<Complex<f32>> = windowed
            .iter()
            .map(|&s| Complex::new(s, 0.0))
            .collect();
        self.fft.process(&mut fft_buffer);

        // Extract magnitude and phase (first 257 bins for real FFT)
        let magnitude: Vec<f32> = fft_buffer[..FFT_BINS]
            .iter()
            .map(|c| c.norm())
            .collect();
        let phase: Vec<f32> = fft_buffer[..FFT_BINS]
            .iter()
            .map(|c| c.arg())
            .collect();

        // Model 1: Magnitude masking
        let (masked_magnitude, new_state_1) = self.run_model_1(&magnitude);
        self.state_1 = new_state_1;

        // Reconstruct complex spectrum with masked magnitude and original phase
        let reconstructed: Vec<Complex<f32>> = masked_magnitude
            .iter()
            .zip(phase.iter())
            .map(|(&mag, &ph)| Complex::from_polar(mag, ph))
            .collect();

        // IFFT - need to reconstruct full spectrum (conjugate symmetric)
        let mut ifft_buffer: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); FRAME_SIZE];
        for (i, &c) in reconstructed.iter().enumerate() {
            ifft_buffer[i] = c;
            if i > 0 && i < FFT_BINS - 1 {
                ifft_buffer[FRAME_SIZE - i] = c.conj();
            }
        }
        self.ifft.process(&mut ifft_buffer);

        // Normalize IFFT output and extract real part
        let scale = 1.0 / FRAME_SIZE as f32;
        let time_domain: Vec<f32> = ifft_buffer
            .iter()
            .map(|c| c.re * scale)
            .collect();

        // Model 2: Time-domain refinement
        let (refined, new_state_2) = self.run_model_2(&time_domain);
        self.state_2 = new_state_2;

        // Apply synthesis window for overlap-add
        refined
            .iter()
            .zip(self.window.iter())
            .map(|(&s, &w)| s * w)
            .collect()
    }

    /// Run Model 1 (magnitude masking)
    fn run_model_1(&self, magnitude: &[f32]) -> (Vec<f32>, Tensor) {
        // Prepare input tensor: shape (1, 1, 257)
        let input_tensor: Tensor = tract_ndarray::Array3::from_shape_fn((1, 1, FFT_BINS), |(_, _, i)| {
            magnitude.get(i).copied().unwrap_or(0.0)
        })
        .into();

        // Run inference with LSTM state
        let result = self
            .model_1
            .run(tvec![
                input_tensor.into(),
                self.state_1.clone().into(),
            ])
            .expect("Model 1 inference failed");

        // Extract outputs: [0] = mask (sigmoid output), [1] = new state
        let mask_output = result[0].to_array_view::<f32>().expect("Invalid mask output");
        let new_state = result[1].clone().into_tensor();

        // Apply mask to magnitude
        let masked: Vec<f32> = magnitude
            .iter()
            .enumerate()
            .map(|(i, &m)| m * mask_output.as_slice().unwrap().get(i).copied().unwrap_or(1.0))
            .collect();

        (masked, new_state)
    }

    /// Run Model 2 (time-domain refinement)
    fn run_model_2(&self, time_domain: &[f32]) -> (Vec<f32>, Tensor) {
        // Prepare input tensor: shape (1, 1, 512)
        let input_tensor: Tensor = tract_ndarray::Array3::from_shape_fn((1, 1, FRAME_SIZE), |(_, _, i)| {
            time_domain.get(i).copied().unwrap_or(0.0)
        })
        .into();

        // Run inference with LSTM state
        let result = self
            .model_2
            .run(tvec![
                input_tensor.into(),
                self.state_2.clone().into(),
            ])
            .expect("Model 2 inference failed");

        // Extract outputs: [0] = refined audio, [1] = new state
        let output = result[0].to_array_view::<f32>().expect("Invalid output");
        let new_state = result[1].clone().into_tensor();

        let refined: Vec<f32> = output.as_slice().unwrap().to_vec();

        (refined, new_state)
    }

    /// Reset the denoiser state for a new audio stream
    ///
    /// Clears input/output buffers and resets LSTM states to zeros.
    /// Call this when starting to process a new, unrelated audio stream.
    pub fn reset(&mut self) {
        self.input_buffer.clear();
        self.output_buffer.fill(0.0);
        self.state_1 = Self::zeros_lstm_state();
        self.state_2 = Self::zeros_lstm_state();
    }

    /// Create a Hann window of given size
    fn hann_window(size: usize) -> Vec<f32> {
        (0..size)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / size as f32).cos()))
            .collect()
    }

    /// Create a zero-initialized LSTM state tensor
    /// Shape: (1, 2, 128, 2) - batch, num_directions, hidden_size, h_and_c
    fn zeros_lstm_state() -> Tensor {
        tract_ndarray::Array4::<f32>::zeros((1, 2, LSTM_UNITS, 2)).into()
    }
}

impl std::fmt::Debug for DtlnDenoiser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DtlnDenoiser")
            .field("input_buffer_len", &self.input_buffer.len())
            .field("output_buffer_len", &self.output_buffer.len())
            .finish_non_exhaustive()
    }
}

//! Voice-optimized audio preprocessing filters.
//!
//! This module provides filters to improve speech quality in the audio pipeline:
//! - **Highpass filter**: Removes low-frequency rumble (HVAC, traffic, handling noise)
//! - **Pre-emphasis filter**: Boosts higher frequencies for speech clarity
//!
//! Both filters are stateful IIR filters that preserve state between audio callbacks.

use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz, Type, Q_BUTTERWORTH_F32};
use crate::audio_constants::{HIGHPASS_CUTOFF_HZ, PRE_EMPHASIS_ALPHA};

/// Highpass filter for removing low-frequency rumble.
///
/// Uses a 2nd-order Butterworth IIR filter for smooth frequency response.
/// The filter operates at the device's native sample rate for best results.
pub struct HighpassFilter {
    filter: DirectForm2Transposed<f32>,
    enabled: bool,
}

impl HighpassFilter {
    /// Create a new highpass filter at the given sample rate.
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz (e.g., 16000, 48000)
    pub fn new(sample_rate: u32) -> Self {
        let coeffs = Coefficients::<f32>::from_params(
            Type::HighPass,
            sample_rate.hz(),
            HIGHPASS_CUTOFF_HZ.hz(),
            Q_BUTTERWORTH_F32,
        )
        .expect("Failed to create highpass filter coefficients");

        Self {
            filter: DirectForm2Transposed::<f32>::new(coeffs),
            enabled: true,
        }
    }

    /// Create a new highpass filter with a custom cutoff frequency.
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `cutoff_hz` - Cutoff frequency in Hz
    #[allow(dead_code)]
    pub fn with_cutoff(sample_rate: u32, cutoff_hz: f32) -> Self {
        let coeffs = Coefficients::<f32>::from_params(
            Type::HighPass,
            sample_rate.hz(),
            cutoff_hz.hz(),
            Q_BUTTERWORTH_F32,
        )
        .expect("Failed to create highpass filter coefficients");

        Self {
            filter: DirectForm2Transposed::<f32>::new(coeffs),
            enabled: true,
        }
    }

    /// Enable or disable the filter.
    ///
    /// When disabled, `process()` returns the input unchanged.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the filter is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Reset the filter state.
    ///
    /// Call this between recording sessions to prevent state carryover.
    pub fn reset(&mut self) {
        self.filter.reset_state();
    }

    /// Process a buffer of samples through the highpass filter.
    ///
    /// # Arguments
    /// * `samples` - Input samples (mono audio)
    ///
    /// # Returns
    /// Filtered samples (same length as input)
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        if !self.enabled {
            return samples.to_vec();
        }

        samples.iter().map(|&s| self.filter.run(s)).collect()
    }

    /// Process samples in-place for better performance.
    ///
    /// # Arguments
    /// * `samples` - Samples to filter in-place
    pub fn process_inplace(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            *sample = self.filter.run(*sample);
        }
    }
}

/// Pre-emphasis filter for boosting high frequencies.
///
/// Implements the standard pre-emphasis filter used in ASR:
/// `y[n] = x[n] - alpha * x[n-1]`
///
/// This first-order FIR filter boosts frequencies above ~300Hz,
/// improving speech clarity by emphasizing consonants.
pub struct PreEmphasisFilter {
    prev_sample: f32,
    alpha: f32,
    enabled: bool,
}

impl PreEmphasisFilter {
    /// Create a new pre-emphasis filter with the default coefficient (0.97).
    pub fn new() -> Self {
        Self {
            prev_sample: 0.0,
            alpha: PRE_EMPHASIS_ALPHA,
            enabled: true,
        }
    }

    /// Create a new pre-emphasis filter with a custom coefficient.
    ///
    /// # Arguments
    /// * `alpha` - Pre-emphasis coefficient (typically 0.95-0.97)
    #[allow(dead_code)]
    pub fn with_alpha(alpha: f32) -> Self {
        Self {
            prev_sample: 0.0,
            alpha,
            enabled: true,
        }
    }

    /// Enable or disable the filter.
    ///
    /// When disabled, `process()` returns the input unchanged.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the filter is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Reset the filter state.
    ///
    /// Call this between recording sessions to prevent state carryover.
    pub fn reset(&mut self) {
        self.prev_sample = 0.0;
    }

    /// Process a buffer of samples through the pre-emphasis filter.
    ///
    /// # Arguments
    /// * `samples` - Input samples (mono audio)
    ///
    /// # Returns
    /// Pre-emphasized samples (same length as input)
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        if !self.enabled {
            return samples.to_vec();
        }

        let mut output = Vec::with_capacity(samples.len());
        for &sample in samples {
            let emphasized = sample - self.alpha * self.prev_sample;
            self.prev_sample = sample;
            output.push(emphasized);
        }
        output
    }

    /// Process samples in-place for better performance.
    ///
    /// # Arguments
    /// * `samples` - Samples to filter in-place
    pub fn process_inplace(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }

        for sample in samples.iter_mut() {
            let original = *sample;
            *sample = original - self.alpha * self.prev_sample;
            self.prev_sample = original;
        }
    }
}

impl Default for PreEmphasisFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Combined preprocessing chain for voice audio.
///
/// Applies filters in the correct order:
/// 1. Highpass filter (remove rumble)
/// 2. Pre-emphasis filter (boost clarity)
pub struct PreprocessingChain {
    highpass: HighpassFilter,
    pre_emphasis: PreEmphasisFilter,
}

impl PreprocessingChain {
    /// Create a new preprocessing chain at the given sample rate.
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(sample_rate: u32) -> Self {
        Self {
            highpass: HighpassFilter::new(sample_rate),
            pre_emphasis: PreEmphasisFilter::new(),
        }
    }

    /// Enable or disable the highpass filter.
    pub fn set_highpass_enabled(&mut self, enabled: bool) {
        self.highpass.set_enabled(enabled);
    }

    /// Enable or disable the pre-emphasis filter.
    pub fn set_pre_emphasis_enabled(&mut self, enabled: bool) {
        self.pre_emphasis.set_enabled(enabled);
    }

    /// Reset all filter states.
    ///
    /// Call this between recording sessions.
    pub fn reset(&mut self) {
        self.highpass.reset();
        self.pre_emphasis.reset();
    }

    /// Process samples through the complete preprocessing chain.
    ///
    /// # Arguments
    /// * `samples` - Input samples (mono audio)
    ///
    /// # Returns
    /// Preprocessed samples
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        // Apply highpass first, then pre-emphasis
        let filtered = self.highpass.process(samples);
        self.pre_emphasis.process(&filtered)
    }

    /// Process samples in-place through the preprocessing chain.
    ///
    /// More efficient for large buffers as it avoids allocation.
    pub fn process_inplace(&mut self, samples: &mut [f32]) {
        self.highpass.process_inplace(samples);
        self.pre_emphasis.process_inplace(samples);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const TEST_SAMPLE_RATE: u32 = 16000;

    /// Generate a sine wave at the given frequency
    fn generate_sine(frequency: f32, sample_rate: u32, num_samples: usize, amplitude: f32) -> Vec<f32> {
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                amplitude * (2.0 * PI * frequency * t).sin()
            })
            .collect()
    }

    /// Calculate RMS (root mean square) of a signal
    fn rms(samples: &[f32]) -> f32 {
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    // =========================================================================
    // Highpass Filter Tests
    // =========================================================================

    #[test]
    fn test_highpass_removes_low_frequency() {
        let mut filter = HighpassFilter::new(TEST_SAMPLE_RATE);

        // Generate 50Hz tone (well below 80Hz cutoff)
        let input = generate_sine(50.0, TEST_SAMPLE_RATE, 4000, 1.0);
        let output = filter.process(&input);

        // Skip first 500 samples (filter settling time)
        let input_rms = rms(&input[500..]);
        let output_rms = rms(&output[500..]);

        // 50Hz is close to the 80Hz cutoff, so attenuation is moderate (about -6dB)
        // 2nd-order Butterworth has 12dB/octave rolloff. 50Hz is ~0.68 octaves below 80Hz
        // Expected attenuation: ~8dB or ~0.4x
        let attenuation = output_rms / input_rms;
        assert!(
            attenuation < 0.5,
            "50Hz should be attenuated below cutoff, got attenuation ratio: {}",
            attenuation
        );
    }

    #[test]
    fn test_highpass_passes_speech_frequencies() {
        let mut filter = HighpassFilter::new(TEST_SAMPLE_RATE);

        // Generate 200Hz tone (well above 80Hz cutoff)
        let input = generate_sine(200.0, TEST_SAMPLE_RATE, 4000, 1.0);
        let output = filter.process(&input);

        // Skip first 500 samples (filter settling time)
        let input_rms = rms(&input[500..]);
        let output_rms = rms(&output[500..]);

        // 200Hz should pass with minimal attenuation (< 1dB ≈ 0.89x)
        let ratio = output_rms / input_rms;
        assert!(
            ratio > 0.85,
            "200Hz should pass with minimal attenuation, got ratio: {}",
            ratio
        );
    }

    #[test]
    fn test_highpass_bypass() {
        let mut filter = HighpassFilter::new(TEST_SAMPLE_RATE);
        filter.set_enabled(false);

        let input = generate_sine(50.0, TEST_SAMPLE_RATE, 1000, 1.0);
        let output = filter.process(&input);

        // Bypassed filter should return identical output
        assert_eq!(input, output);
    }

    #[test]
    fn test_highpass_reset() {
        let mut filter = HighpassFilter::new(TEST_SAMPLE_RATE);

        // Process some samples to build up state
        let _ = filter.process(&generate_sine(100.0, TEST_SAMPLE_RATE, 1000, 1.0));

        // Reset and process a new signal
        filter.reset();
        let input = vec![0.0; 100];
        let output = filter.process(&input);

        // After reset, zero input should produce near-zero output
        let max_output = output.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(
            max_output < 0.001,
            "Reset filter should produce near-zero output for zero input, got max: {}",
            max_output
        );
    }

    // =========================================================================
    // Pre-emphasis Filter Tests
    // =========================================================================

    #[test]
    fn test_pre_emphasis_boosts_high_frequencies() {
        let mut filter = PreEmphasisFilter::new();

        // Compare 100Hz (low) vs 1000Hz (high)
        let low_freq = generate_sine(100.0, TEST_SAMPLE_RATE, 2000, 1.0);
        let high_freq = generate_sine(1000.0, TEST_SAMPLE_RATE, 2000, 1.0);

        // Process both through separate filter instances
        let mut filter_low = PreEmphasisFilter::new();
        let mut filter_high = PreEmphasisFilter::new();

        let low_output = filter_low.process(&low_freq);
        let high_output = filter_high.process(&high_freq);

        // Skip first 100 samples for settling
        let low_rms = rms(&low_output[100..]);
        let high_rms = rms(&high_output[100..]);

        // High frequency should be boosted relative to low frequency
        // (1000Hz should have higher RMS than 100Hz after pre-emphasis)
        assert!(
            high_rms > low_rms,
            "Pre-emphasis should boost high frequencies: 1kHz RMS={}, 100Hz RMS={}",
            high_rms,
            low_rms
        );
    }

    #[test]
    fn test_pre_emphasis_formula() {
        let mut filter = PreEmphasisFilter::new();

        // Simple test case
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let output = filter.process(&input);

        // y[n] = x[n] - 0.97 * x[n-1], with x[-1] = 0
        let expected = vec![
            1.0 - 0.97 * 0.0,  // 1.0
            2.0 - 0.97 * 1.0,  // 1.03
            3.0 - 0.97 * 2.0,  // 1.06
            4.0 - 0.97 * 3.0,  // 1.09
            5.0 - 0.97 * 4.0,  // 1.12
        ];

        for (i, (out, exp)) in output.iter().zip(expected.iter()).enumerate() {
            assert!(
                (out - exp).abs() < 0.0001,
                "Sample {}: expected {}, got {}",
                i,
                exp,
                out
            );
        }
    }

    #[test]
    fn test_pre_emphasis_bypass() {
        let mut filter = PreEmphasisFilter::new();
        filter.set_enabled(false);

        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let output = filter.process(&input);

        assert_eq!(input, output);
    }

    #[test]
    fn test_pre_emphasis_reset() {
        let mut filter = PreEmphasisFilter::new();

        // Process some samples to set prev_sample
        let _ = filter.process(&vec![1.0, 2.0, 3.0]);

        // Reset
        filter.reset();

        // After reset, first sample should use prev_sample = 0
        let output = filter.process(&vec![1.0]);
        assert_eq!(output[0], 1.0); // 1.0 - 0.97 * 0.0 = 1.0
    }

    // =========================================================================
    // Preprocessing Chain Tests
    // =========================================================================

    #[test]
    fn test_chain_applies_both_filters() {
        let mut chain = PreprocessingChain::new(TEST_SAMPLE_RATE);

        // Generate a signal with both low and high frequency components
        let low_freq = generate_sine(50.0, TEST_SAMPLE_RATE, 2000, 1.0);
        let high_freq = generate_sine(1000.0, TEST_SAMPLE_RATE, 2000, 1.0);
        let mixed: Vec<f32> = low_freq.iter().zip(high_freq.iter()).map(|(l, h)| l + h).collect();

        let output = chain.process(&mixed);

        // Output should be different from input (filters applied)
        assert_ne!(mixed, output);

        // Low frequency should be attenuated, high frequency preserved/boosted
        // We can't easily separate them, but the RMS should be lower than the combined input
        // due to the highpass removing the 50Hz component
        let input_rms = rms(&mixed[500..]);
        let output_rms = rms(&output[500..]);

        // The output should be smaller because the 50Hz was removed
        assert!(
            output_rms < input_rms,
            "Chain should remove low frequencies: input RMS={}, output RMS={}",
            input_rms,
            output_rms
        );
    }

    #[test]
    fn test_chain_reset() {
        let mut chain = PreprocessingChain::new(TEST_SAMPLE_RATE);

        // Process some samples
        let _ = chain.process(&generate_sine(500.0, TEST_SAMPLE_RATE, 1000, 1.0));

        // Reset and verify clean state
        chain.reset();
        let input = vec![0.0; 100];
        let output = chain.process(&input);

        let max_output = output.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        assert!(
            max_output < 0.001,
            "Reset chain should produce near-zero output for zero input"
        );
    }

    #[test]
    fn test_chain_individual_bypass() {
        let input = generate_sine(100.0, TEST_SAMPLE_RATE, 500, 1.0);

        // Both enabled
        let mut chain_both = PreprocessingChain::new(TEST_SAMPLE_RATE);
        let output_both = chain_both.process(&input);

        // Only highpass
        let mut chain_hp_only = PreprocessingChain::new(TEST_SAMPLE_RATE);
        chain_hp_only.set_pre_emphasis_enabled(false);
        let output_hp = chain_hp_only.process(&input);

        // Only pre-emphasis
        let mut chain_pe_only = PreprocessingChain::new(TEST_SAMPLE_RATE);
        chain_pe_only.set_highpass_enabled(false);
        let output_pe = chain_pe_only.process(&input);

        // All three outputs should be different
        assert_ne!(output_both, output_hp);
        assert_ne!(output_both, output_pe);
        assert_ne!(output_hp, output_pe);
    }

    #[test]
    fn test_inplace_processing() {
        let original = generate_sine(500.0, TEST_SAMPLE_RATE, 1000, 1.0);

        // Test highpass inplace
        let mut samples_hp = original.clone();
        let mut filter_hp = HighpassFilter::new(TEST_SAMPLE_RATE);
        filter_hp.process_inplace(&mut samples_hp);
        let expected_hp = HighpassFilter::new(TEST_SAMPLE_RATE).process(&original);

        for (i, (inplace, regular)) in samples_hp.iter().zip(expected_hp.iter()).enumerate() {
            assert!(
                (inplace - regular).abs() < 0.0001,
                "Highpass inplace mismatch at {}: {} vs {}",
                i,
                inplace,
                regular
            );
        }

        // Test pre-emphasis inplace
        let mut samples_pe = original.clone();
        let mut filter_pe = PreEmphasisFilter::new();
        filter_pe.process_inplace(&mut samples_pe);
        let expected_pe = PreEmphasisFilter::new().process(&original);

        for (i, (inplace, regular)) in samples_pe.iter().zip(expected_pe.iter()).enumerate() {
            assert!(
                (inplace - regular).abs() < 0.0001,
                "Pre-emphasis inplace mismatch at {}: {} vs {}",
                i,
                inplace,
                regular
            );
        }
    }
}

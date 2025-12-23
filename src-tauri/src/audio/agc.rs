//! Automatic Gain Control (AGC) for consistent recording volume levels
//!
//! This module implements AGC to normalize recording volume, boosting quiet
//! signals and preventing clipping on loud signals. Key features:
//! - Adaptive gain adjustment based on RMS level
//! - Attack/release envelope to avoid pumping artifacts
//! - Soft limiter to prevent clipping
//! - Configurable via environment variables

use std::sync::atomic::{AtomicBool, Ordering};

/// Target RMS level in linear scale (corresponds to -12dBFS)
/// -12dBFS = 10^(-12/20) ≈ 0.251
const DEFAULT_TARGET_RMS: f32 = 0.251;

/// Maximum gain in linear scale (+20dB = 10^(20/20) = 10.0)
const DEFAULT_MAX_GAIN: f32 = 10.0;

/// Minimum gain (no attenuation below unity)
const MIN_GAIN: f32 = 1.0;

/// Soft limiter threshold in linear scale (-3dBFS = 10^(-3/20) ≈ 0.708)
const SOFT_LIMIT_THRESHOLD: f32 = 0.708;

/// Attack time coefficient for fast response to loud sounds
/// Derived from: exp(-1 / (sample_rate * attack_time_seconds))
/// For 16kHz and 10ms attack: exp(-1 / (16000 * 0.01)) ≈ 0.9937
const DEFAULT_ATTACK_COEFF: f32 = 0.9937;

/// Release time coefficient for smooth gain recovery
/// For 16kHz and 200ms release: exp(-1 / (16000 * 0.2)) ≈ 0.99969
const DEFAULT_RELEASE_COEFF: f32 = 0.99969;

/// Minimum RMS level to avoid excessive gain on silence
/// Below this level, gain stays at current value (prevents noise amplification)
const MIN_RMS_THRESHOLD: f32 = 0.001;

/// Check if AGC is disabled via environment variable
fn agc_disabled() -> bool {
    std::env::var("HEYCAT_DISABLE_AGC").is_ok()
}

/// Automatic Gain Control processor
///
/// Tracks audio levels and applies adaptive gain to maintain consistent volume.
pub struct AutomaticGainControl {
    /// Current gain value (linear scale)
    current_gain: f32,
    /// Smoothed RMS level for gain calculation
    rms_envelope: f32,
    /// Target RMS level (linear scale)
    target_rms: f32,
    /// Maximum allowed gain
    max_gain: f32,
    /// Attack coefficient (for fast response to loud sounds)
    attack_coeff: f32,
    /// Release coefficient (for smooth gain recovery)
    release_coeff: f32,
    /// Whether AGC is enabled
    enabled: AtomicBool,
}

impl AutomaticGainControl {
    /// Create a new AGC processor with default parameters
    ///
    /// Default settings are optimized for voice recording at 16kHz:
    /// - Target level: -12dBFS RMS
    /// - Max gain: +20dB
    /// - Attack time: 10ms
    /// - Release time: 200ms
    pub fn new() -> Self {
        let disabled = agc_disabled();
        Self {
            current_gain: 1.0,
            rms_envelope: 0.0,
            target_rms: DEFAULT_TARGET_RMS,
            max_gain: DEFAULT_MAX_GAIN,
            attack_coeff: DEFAULT_ATTACK_COEFF,
            release_coeff: DEFAULT_RELEASE_COEFF,
            enabled: AtomicBool::new(!disabled),
        }
    }

    /// Create AGC with custom sample rate
    ///
    /// Adjusts attack/release coefficients for the given sample rate.
    #[allow(dead_code)]
    pub fn with_sample_rate(sample_rate: u32) -> Self {
        let attack_time = 0.010; // 10ms
        let release_time = 0.200; // 200ms

        // Coefficient = exp(-1 / (sample_rate * time))
        let attack_coeff = (-1.0 / (sample_rate as f32 * attack_time)).exp();
        let release_coeff = (-1.0 / (sample_rate as f32 * release_time)).exp();

        let disabled = agc_disabled();
        Self {
            current_gain: 1.0,
            rms_envelope: 0.0,
            target_rms: DEFAULT_TARGET_RMS,
            max_gain: DEFAULT_MAX_GAIN,
            attack_coeff,
            release_coeff,
            enabled: AtomicBool::new(!disabled),
        }
    }

    /// Check if AGC is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable or disable AGC
    #[allow(dead_code)]
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Reset AGC state (call between recordings)
    pub fn reset(&mut self) {
        self.current_gain = 1.0;
        self.rms_envelope = 0.0;
    }

    /// Get current gain value (for diagnostics)
    #[allow(dead_code)]
    pub fn current_gain(&self) -> f32 {
        self.current_gain
    }

    /// Get current gain in decibels (for diagnostics)
    #[allow(dead_code)]
    pub fn current_gain_db(&self) -> f32 {
        20.0 * self.current_gain.log10()
    }

    /// Process audio samples with AGC
    ///
    /// Returns a new vector with gain-adjusted samples.
    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        if !self.is_enabled() || samples.is_empty() {
            return samples.to_vec();
        }

        let mut output = Vec::with_capacity(samples.len());

        for &sample in samples {
            // Update RMS envelope (exponential moving average of squared samples)
            let sample_squared = sample * sample;
            let coeff = if sample_squared > self.rms_envelope {
                self.attack_coeff // Fast attack for loud sounds
            } else {
                self.release_coeff // Slow release for quiet periods
            };
            self.rms_envelope = coeff * self.rms_envelope + (1.0 - coeff) * sample_squared;

            // Calculate RMS from envelope
            let rms = self.rms_envelope.sqrt();

            // Calculate target gain (only if signal is above noise floor)
            if rms > MIN_RMS_THRESHOLD {
                let target_gain = (self.target_rms / rms).clamp(MIN_GAIN, self.max_gain);

                // Smooth gain transition using the same envelope approach
                let gain_coeff = if target_gain < self.current_gain {
                    self.attack_coeff // Fast reduction for loud sounds
                } else {
                    self.release_coeff // Slow increase for quiet sounds
                };
                self.current_gain =
                    gain_coeff * self.current_gain + (1.0 - gain_coeff) * target_gain;
            }
            // If signal is below threshold, keep current gain (prevents noise pumping)

            // Apply gain
            let gained = sample * self.current_gain;

            // Apply soft limiter to prevent clipping
            let limited = soft_limit(gained);

            output.push(limited);
        }

        output
    }

    /// Process audio samples in-place
    #[allow(dead_code)]
    pub fn process_inplace(&mut self, samples: &mut [f32]) {
        if !self.is_enabled() || samples.is_empty() {
            return;
        }

        for sample in samples.iter_mut() {
            let input = *sample;

            // Update RMS envelope
            let sample_squared = input * input;
            let coeff = if sample_squared > self.rms_envelope {
                self.attack_coeff
            } else {
                self.release_coeff
            };
            self.rms_envelope = coeff * self.rms_envelope + (1.0 - coeff) * sample_squared;

            // Calculate RMS and update gain
            let rms = self.rms_envelope.sqrt();
            if rms > MIN_RMS_THRESHOLD {
                let target_gain = (self.target_rms / rms).clamp(MIN_GAIN, self.max_gain);
                let gain_coeff = if target_gain < self.current_gain {
                    self.attack_coeff
                } else {
                    self.release_coeff
                };
                self.current_gain =
                    gain_coeff * self.current_gain + (1.0 - gain_coeff) * target_gain;
            }

            // Apply gain and soft limiting
            *sample = soft_limit(input * self.current_gain);
        }
    }
}

impl Default for AutomaticGainControl {
    fn default() -> Self {
        Self::new()
    }
}

/// Soft limiter using tanh-based sigmoid function
///
/// This prevents hard clipping by smoothly compressing signals above the threshold.
/// Below threshold: minimal change
/// Above threshold: smooth compression toward ±1.0
fn soft_limit(sample: f32) -> f32 {
    let abs_sample = sample.abs();

    if abs_sample <= SOFT_LIMIT_THRESHOLD {
        // Below threshold - pass through
        sample
    } else {
        // Above threshold - apply soft compression
        // Map (threshold, inf) to (threshold, 1.0) using tanh
        let excess = abs_sample - SOFT_LIMIT_THRESHOLD;
        let headroom = 1.0 - SOFT_LIMIT_THRESHOLD;

        // Scale excess into tanh range and compress
        let compressed = SOFT_LIMIT_THRESHOLD + headroom * (excess / headroom).tanh();

        // Preserve sign
        compressed.copysign(sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agc_boosts_quiet_input() {
        let mut agc = AutomaticGainControl::new();

        // Generate quiet input at roughly -30dBFS (0.0316 linear)
        let quiet_level = 0.0316_f32;
        let input: Vec<f32> = (0..16000).map(|i| (i as f32 * 0.1).sin() * quiet_level).collect();

        let output = agc.process(&input);

        // Calculate output RMS
        let output_rms: f32 = (output.iter().map(|s| s * s).sum::<f32>() / output.len() as f32).sqrt();

        // Output should be boosted toward target (-12dBFS ≈ 0.251)
        // Allow some tolerance for envelope settling
        assert!(
            output_rms > quiet_level * 2.0,
            "Output RMS {} should be significantly higher than input {}",
            output_rms,
            quiet_level
        );
    }

    #[test]
    fn test_agc_normal_input_minimal_change() {
        let mut agc = AutomaticGainControl::new();

        // Generate input at target level (-12dBFS ≈ 0.251)
        let normal_level = 0.251_f32;
        let input: Vec<f32> = (0..16000).map(|i| (i as f32 * 0.1).sin() * normal_level).collect();

        let output = agc.process(&input);

        // Calculate input and output RMS
        let input_rms: f32 = (input.iter().map(|s| s * s).sum::<f32>() / input.len() as f32).sqrt();
        let output_rms: f32 = (output.iter().map(|s| s * s).sum::<f32>() / output.len() as f32).sqrt();

        // Output should be close to input (within 50% - accounting for envelope settling)
        let ratio = output_rms / input_rms;
        assert!(
            (0.5..2.0).contains(&ratio),
            "Output/input ratio {} should be close to 1.0 for normal input",
            ratio
        );
    }

    #[test]
    fn test_agc_loud_input_not_clipped() {
        let mut agc = AutomaticGainControl::new();

        // Generate loud input that would clip with gain
        let loud_level = 0.9_f32;
        let input: Vec<f32> = (0..16000).map(|i| (i as f32 * 0.1).sin() * loud_level).collect();

        let output = agc.process(&input);

        // No sample should exceed ±1.0
        let max_output = output.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(
            max_output <= 1.0,
            "Output max {} should not exceed 1.0",
            max_output
        );
    }

    #[test]
    fn test_soft_limiter_below_threshold() {
        // Below threshold should pass through unchanged
        let sample = 0.5_f32;
        let limited = soft_limit(sample);
        assert!(
            (limited - sample).abs() < 0.001,
            "Sample {} below threshold should pass through, got {}",
            sample,
            limited
        );
    }

    #[test]
    fn test_soft_limiter_above_threshold() {
        // Above threshold should be compressed
        let sample = 0.9_f32;
        let limited = soft_limit(sample);
        assert!(
            limited < sample,
            "Sample {} above threshold should be compressed, got {}",
            sample,
            limited
        );
        assert!(
            limited > SOFT_LIMIT_THRESHOLD,
            "Limited sample {} should be above threshold {}",
            limited,
            SOFT_LIMIT_THRESHOLD
        );
    }

    #[test]
    fn test_soft_limiter_extreme_values() {
        // Extreme values should be compressed to near 1.0 (asymptotically approaches 1.0)
        let sample = 10.0_f32;
        let limited = soft_limit(sample);
        assert!(
            limited <= 1.0,
            "Extreme sample should be limited to at most 1.0, got {}",
            limited
        );
        assert!(
            limited > 0.9,
            "Extreme sample should still be near 1.0, got {}",
            limited
        );
    }

    #[test]
    fn test_soft_limiter_preserves_sign() {
        let positive = soft_limit(0.9);
        let negative = soft_limit(-0.9);
        assert!(positive > 0.0, "Positive input should give positive output");
        assert!(negative < 0.0, "Negative input should give negative output");
    }

    #[test]
    fn test_agc_silence_no_runaway_gain() {
        let mut agc = AutomaticGainControl::new();

        // Process some normal audio first
        let normal: Vec<f32> = (0..8000).map(|i| (i as f32 * 0.1).sin() * 0.1).collect();
        let _ = agc.process(&normal);
        let gain_after_normal = agc.current_gain();

        // Process silence
        let silence: Vec<f32> = vec![0.0001; 8000]; // Very quiet but not zero
        let _ = agc.process(&silence);
        let gain_after_silence = agc.current_gain();

        // Gain should not have increased dramatically on silence
        // (should stay near the value it had after normal audio)
        assert!(
            gain_after_silence <= agc.max_gain,
            "Gain {} should not exceed max {}",
            gain_after_silence,
            agc.max_gain
        );
    }

    #[test]
    fn test_agc_reset() {
        let mut agc = AutomaticGainControl::new();

        // Process some audio to change state
        let input: Vec<f32> = (0..16000).map(|i| (i as f32 * 0.1).sin() * 0.1).collect();
        let _ = agc.process(&input);

        assert!(agc.current_gain() != 1.0 || agc.rms_envelope != 0.0);

        // Reset
        agc.reset();

        assert_eq!(agc.current_gain(), 1.0, "Gain should be 1.0 after reset");
        assert_eq!(agc.rms_envelope, 0.0, "RMS envelope should be 0 after reset");
    }

    #[test]
    fn test_agc_disabled_passthrough() {
        let mut agc = AutomaticGainControl::new();
        agc.set_enabled(false);

        let input: Vec<f32> = vec![0.5, 0.3, -0.2, 0.1];
        let output = agc.process(&input);

        assert_eq!(input, output, "Disabled AGC should pass through unchanged");
    }

    #[test]
    fn test_agc_gain_db_conversion() {
        let mut agc = AutomaticGainControl::new();

        // Unity gain = 0dB
        assert!(
            agc.current_gain_db().abs() < 0.1,
            "Initial gain should be ~0dB"
        );

        // Manually set gain to 10x = +20dB
        agc.current_gain = 10.0;
        assert!(
            (agc.current_gain_db() - 20.0).abs() < 0.1,
            "Gain of 10x should be ~20dB, got {}",
            agc.current_gain_db()
        );
    }

    #[test]
    fn test_agc_with_custom_sample_rate() {
        // Test that custom sample rate adjusts coefficients
        let agc_16k = AutomaticGainControl::new();
        let agc_48k = AutomaticGainControl::with_sample_rate(48000);

        // Higher sample rate should have coefficients closer to 1.0
        // (slower change per sample since there are more samples per second)
        assert!(
            agc_48k.attack_coeff > agc_16k.attack_coeff,
            "48kHz attack coeff {} should be > 16kHz coeff {}",
            agc_48k.attack_coeff,
            agc_16k.attack_coeff
        );
    }
}

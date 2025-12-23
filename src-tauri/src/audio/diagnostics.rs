//! Audio pipeline diagnostics and quality metrics
//!
//! This module provides comprehensive quality metrics, warnings, and diagnostic
//! tooling for the audio processing pipeline. Features include:
//! - Real-time level tracking (peak/RMS)
//! - Clipping detection
//! - AGC gain monitoring
//! - Debug mode for raw audio capture
//! - Frontend warning events

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Threshold for "too quiet" warning (-30dBFS RMS ≈ 0.0316 linear)
const QUIET_THRESHOLD_RMS: f32 = 0.0316;

/// Threshold for clipping detection (samples at or near ±1.0)
const CLIPPING_THRESHOLD: f32 = 0.99;

/// Minimum sample count before issuing warnings (avoid false positives on short bursts)
const MIN_SAMPLES_FOR_WARNING: usize = 8000; // ~0.5 seconds at 16kHz

/// Check if diagnostics verbose mode is enabled via environment variable
fn diagnostics_verbose() -> bool {
    std::env::var("HEYCAT_DIAGNOSTICS_VERBOSE").is_ok()
}

/// Check if debug audio capture is enabled via environment variable
pub fn debug_audio_enabled() -> bool {
    std::env::var("HEYCAT_DEBUG_AUDIO").is_ok()
}

/// Quality warning types emitted to the frontend
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityWarningType {
    /// Input signal is too quiet for reliable transcription
    TooQuiet,
    /// Input signal is clipping (distortion)
    Clipping,
}

/// Severity level for quality warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    /// Informational - may affect quality but not critical
    Info,
    /// Warning - likely to affect transcription quality
    Warning,
}

/// Quality warning event payload for frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct QualityWarning {
    pub warning_type: QualityWarningType,
    pub severity: WarningSeverity,
    pub message: String,
}

/// Audio level metrics
#[derive(Debug, Clone, Default)]
pub struct LevelMetrics {
    /// Peak level (maximum absolute sample value)
    pub peak: f32,
    /// RMS level (root mean square)
    pub rms: f32,
    /// Number of samples analyzed
    pub sample_count: usize,
}

impl LevelMetrics {
    /// Calculate peak and RMS from samples
    pub fn from_samples(samples: &[f32]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut peak: f32 = 0.0;
        let mut sum_sq: f32 = 0.0;

        for &sample in samples {
            let abs_sample = sample.abs();
            if abs_sample > peak {
                peak = abs_sample;
            }
            sum_sq += sample * sample;
        }

        let rms = (sum_sq / samples.len() as f32).sqrt();

        Self {
            peak,
            rms,
            sample_count: samples.len(),
        }
    }

    /// Convert peak to dBFS
    pub fn peak_dbfs(&self) -> f32 {
        if self.peak <= 0.0 {
            f32::NEG_INFINITY
        } else {
            20.0 * self.peak.log10()
        }
    }

    /// Convert RMS to dBFS
    pub fn rms_dbfs(&self) -> f32 {
        if self.rms <= 0.0 {
            f32::NEG_INFINITY
        } else {
            20.0 * self.rms.log10()
        }
    }
}

/// Recording diagnostics collector
///
/// Collects metrics throughout a recording session and can emit warnings
/// or save debug files when appropriate.
pub struct RecordingDiagnostics {
    /// Total input samples received
    input_sample_count: AtomicUsize,
    /// Total output samples produced
    output_sample_count: AtomicUsize,
    /// Running peak level (input)
    input_peak: std::sync::Mutex<f32>,
    /// Running sum of squared samples for RMS (input)
    input_sum_sq: std::sync::Mutex<f64>,
    /// Running peak level (output)
    output_peak: std::sync::Mutex<f32>,
    /// Running sum of squared samples for RMS (output)
    output_sum_sq: std::sync::Mutex<f64>,
    /// Count of clipping samples detected
    clipping_count: AtomicUsize,
    /// Whether verbose diagnostics are enabled
    verbose: bool,
    /// Whether debug audio capture is enabled
    debug_enabled: bool,
    /// Buffer for raw (pre-processing) audio in debug mode
    raw_audio_buffer: std::sync::Mutex<Vec<f32>>,
    /// Whether warnings have been emitted (to avoid spam)
    quiet_warning_emitted: AtomicBool,
    clipping_warning_emitted: AtomicBool,
}

impl RecordingDiagnostics {
    /// Create a new diagnostics collector
    pub fn new() -> Self {
        Self {
            input_sample_count: AtomicUsize::new(0),
            output_sample_count: AtomicUsize::new(0),
            input_peak: std::sync::Mutex::new(0.0),
            input_sum_sq: std::sync::Mutex::new(0.0),
            output_peak: std::sync::Mutex::new(0.0),
            output_sum_sq: std::sync::Mutex::new(0.0),
            clipping_count: AtomicUsize::new(0),
            verbose: diagnostics_verbose(),
            debug_enabled: debug_audio_enabled(),
            raw_audio_buffer: std::sync::Mutex::new(Vec::new()),
            quiet_warning_emitted: AtomicBool::new(false),
            clipping_warning_emitted: AtomicBool::new(false),
        }
    }

    /// Record input samples (call before processing)
    pub fn record_input(&self, samples: &[f32]) {
        let count = samples.len();
        self.input_sample_count.fetch_add(count, Ordering::Relaxed);

        // Update peak and sum of squares
        if let (Ok(mut peak), Ok(mut sum_sq)) =
            (self.input_peak.lock(), self.input_sum_sq.lock())
        {
            for &sample in samples {
                let abs_sample = sample.abs();
                if abs_sample > *peak {
                    *peak = abs_sample;
                }
                *sum_sq += (sample * sample) as f64;

                // Detect clipping
                if abs_sample >= CLIPPING_THRESHOLD {
                    self.clipping_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Store raw audio in debug mode
        if self.debug_enabled {
            if let Ok(mut buffer) = self.raw_audio_buffer.lock() {
                buffer.extend_from_slice(samples);
            }
        }
    }

    /// Record output samples (call after processing)
    pub fn record_output(&self, samples: &[f32]) {
        let count = samples.len();
        self.output_sample_count.fetch_add(count, Ordering::Relaxed);

        // Update peak and sum of squares
        if let (Ok(mut peak), Ok(mut sum_sq)) =
            (self.output_peak.lock(), self.output_sum_sq.lock())
        {
            for &sample in samples {
                let abs_sample = sample.abs();
                if abs_sample > *peak {
                    *peak = abs_sample;
                }
                *sum_sq += (sample * sample) as f64;
            }
        }
    }

    /// Get input level metrics
    pub fn input_metrics(&self) -> LevelMetrics {
        let sample_count = self.input_sample_count.load(Ordering::Relaxed);
        let peak = self.input_peak.lock().map(|p| *p).unwrap_or(0.0);
        let sum_sq = self.input_sum_sq.lock().map(|s| *s).unwrap_or(0.0);

        let rms = if sample_count > 0 {
            ((sum_sq / sample_count as f64) as f32).sqrt()
        } else {
            0.0
        };

        LevelMetrics {
            peak,
            rms,
            sample_count,
        }
    }

    /// Get output level metrics
    pub fn output_metrics(&self) -> LevelMetrics {
        let sample_count = self.output_sample_count.load(Ordering::Relaxed);
        let peak = self.output_peak.lock().map(|p| *p).unwrap_or(0.0);
        let sum_sq = self.output_sum_sq.lock().map(|s| *s).unwrap_or(0.0);

        let rms = if sample_count > 0 {
            ((sum_sq / sample_count as f64) as f32).sqrt()
        } else {
            0.0
        };

        LevelMetrics {
            peak,
            rms,
            sample_count,
        }
    }

    /// Get clipping count
    pub fn clipping_count(&self) -> usize {
        self.clipping_count.load(Ordering::Relaxed)
    }

    /// Check for quality warnings and return them
    ///
    /// Call this periodically or at the end of recording.
    /// Each warning type is only returned once per recording session.
    pub fn check_warnings(&self) -> Vec<QualityWarning> {
        let mut warnings = Vec::new();

        let input = self.input_metrics();

        // Check for quiet input (only after enough samples)
        if input.sample_count >= MIN_SAMPLES_FOR_WARNING {
            if input.rms < QUIET_THRESHOLD_RMS
                && !self.quiet_warning_emitted.swap(true, Ordering::Relaxed)
            {
                warnings.push(QualityWarning {
                    warning_type: QualityWarningType::TooQuiet,
                    severity: WarningSeverity::Warning,
                    message: format!(
                        "Input signal is very quiet ({:.1}dBFS RMS). Move closer to microphone or speak louder.",
                        input.rms_dbfs()
                    ),
                });
            }
        }

        // Check for clipping
        let clip_count = self.clipping_count();
        if clip_count > 0
            && !self.clipping_warning_emitted.swap(true, Ordering::Relaxed)
        {
            let severity = if clip_count > 100 {
                WarningSeverity::Warning
            } else {
                WarningSeverity::Info
            };

            warnings.push(QualityWarning {
                warning_type: QualityWarningType::Clipping,
                severity,
                message: format!(
                    "Audio clipping detected ({} samples). Reduce microphone gain or move further away.",
                    clip_count
                ),
            });
        }

        warnings
    }

    /// Get raw audio buffer for debug mode
    ///
    /// Returns the raw audio if debug mode is enabled, otherwise None.
    pub fn raw_audio(&self) -> Option<Vec<f32>> {
        if self.debug_enabled {
            self.raw_audio_buffer.lock().ok().map(|b| b.clone())
        } else {
            None
        }
    }

    /// Log comprehensive diagnostics (call at end of recording)
    pub fn log_summary(&self, agc_gain_db: Option<f32>) {
        let input = self.input_metrics();
        let output = self.output_metrics();
        let clip_count = self.clipping_count();

        // Always log basic summary
        crate::info!(
            "[DIAGNOSTICS] Recording summary: input={} samples (peak={:.2}dBFS, rms={:.2}dBFS), output={} samples (peak={:.2}dBFS, rms={:.2}dBFS), clipping={}{}",
            input.sample_count,
            input.peak_dbfs(),
            input.rms_dbfs(),
            output.sample_count,
            output.peak_dbfs(),
            output.rms_dbfs(),
            clip_count,
            agc_gain_db.map(|g| format!(", agc_gain={:.1}dB", g)).unwrap_or_default()
        );

        // Verbose mode: additional details
        if self.verbose {
            let ratio = if input.sample_count > 0 {
                output.sample_count as f64 / input.sample_count as f64
            } else {
                0.0
            };

            crate::info!(
                "[DIAGNOSTICS] Verbose: sample_ratio={:.4}, debug_mode={}",
                ratio,
                self.debug_enabled
            );
        }
    }

    /// Check if verbose mode is enabled
    #[allow(dead_code)]
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Check if debug audio capture is enabled
    #[allow(dead_code)]
    pub fn is_debug_enabled(&self) -> bool {
        self.debug_enabled
    }
}

impl Default for RecordingDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

// Allow sharing across threads
unsafe impl Send for RecordingDiagnostics {}
unsafe impl Sync for RecordingDiagnostics {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_metrics_from_samples() {
        let samples = vec![0.5_f32, -0.3, 0.8, -0.2, 0.1];
        let metrics = LevelMetrics::from_samples(&samples);

        assert_eq!(metrics.sample_count, 5);
        assert!((metrics.peak - 0.8).abs() < 0.001);
        // RMS = sqrt((0.25 + 0.09 + 0.64 + 0.04 + 0.01) / 5) = sqrt(0.206) ≈ 0.454
        assert!((metrics.rms - 0.454).abs() < 0.01);
    }

    #[test]
    fn test_level_metrics_empty() {
        let metrics = LevelMetrics::from_samples(&[]);
        assert_eq!(metrics.sample_count, 0);
        assert_eq!(metrics.peak, 0.0);
        assert_eq!(metrics.rms, 0.0);
    }

    #[test]
    fn test_dbfs_conversion() {
        let metrics = LevelMetrics {
            peak: 1.0, // 0 dBFS
            rms: 0.5,  // ~-6 dBFS
            sample_count: 100,
        };

        assert!((metrics.peak_dbfs() - 0.0).abs() < 0.001);
        assert!((metrics.rms_dbfs() - (-6.02)).abs() < 0.1);
    }

    #[test]
    fn test_dbfs_zero() {
        let metrics = LevelMetrics {
            peak: 0.0,
            rms: 0.0,
            sample_count: 0,
        };

        assert!(metrics.peak_dbfs() == f32::NEG_INFINITY);
        assert!(metrics.rms_dbfs() == f32::NEG_INFINITY);
    }

    #[test]
    fn test_diagnostics_record_input() {
        let diag = RecordingDiagnostics::new();

        let samples = vec![0.5_f32; 1000];
        diag.record_input(&samples);

        let metrics = diag.input_metrics();
        assert_eq!(metrics.sample_count, 1000);
        assert!((metrics.peak - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_diagnostics_record_output() {
        let diag = RecordingDiagnostics::new();

        let samples = vec![0.3_f32; 500];
        diag.record_output(&samples);

        let metrics = diag.output_metrics();
        assert_eq!(metrics.sample_count, 500);
        assert!((metrics.peak - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_clipping_detection() {
        let diag = RecordingDiagnostics::new();

        let samples = vec![0.5, 0.99, 1.0, -1.0, 0.5]; // 3 clipping samples
        diag.record_input(&samples);

        assert_eq!(diag.clipping_count(), 3);
    }

    #[test]
    fn test_quiet_warning() {
        let diag = RecordingDiagnostics::new();

        // Generate enough quiet samples
        let quiet_samples: Vec<f32> = vec![0.01; MIN_SAMPLES_FOR_WARNING + 100];
        diag.record_input(&quiet_samples);

        let warnings = diag.check_warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].warning_type, QualityWarningType::TooQuiet);
        assert_eq!(warnings[0].severity, WarningSeverity::Warning);
    }

    #[test]
    fn test_clipping_warning() {
        let diag = RecordingDiagnostics::new();

        // Generate samples with clipping
        let clipping_samples: Vec<f32> = (0..1000).map(|_| 1.0).collect();
        diag.record_input(&clipping_samples);

        let warnings = diag.check_warnings();
        assert!(warnings.iter().any(|w| w.warning_type == QualityWarningType::Clipping));
    }

    #[test]
    fn test_warning_only_emitted_once() {
        let diag = RecordingDiagnostics::new();

        // Generate clipping samples
        let samples = vec![1.0_f32; 1000];
        diag.record_input(&samples);

        let warnings1 = diag.check_warnings();
        let warnings2 = diag.check_warnings();

        // First call should return warnings
        assert!(!warnings1.is_empty());
        // Second call should return empty (already emitted)
        assert!(warnings2.is_empty());
    }

    #[test]
    fn test_no_warning_for_short_audio() {
        let diag = RecordingDiagnostics::new();

        // Very quiet but short audio
        let samples: Vec<f32> = vec![0.001; 100];
        diag.record_input(&samples);

        let warnings = diag.check_warnings();
        // Should not emit warning for short audio
        assert!(
            !warnings.iter().any(|w| w.warning_type == QualityWarningType::TooQuiet),
            "Should not warn about quiet audio until minimum sample count reached"
        );
    }

    #[test]
    fn test_diagnostics_default() {
        let diag = RecordingDiagnostics::default();
        assert_eq!(diag.input_metrics().sample_count, 0);
        assert_eq!(diag.output_metrics().sample_count, 0);
        assert_eq!(diag.clipping_count(), 0);
    }
}

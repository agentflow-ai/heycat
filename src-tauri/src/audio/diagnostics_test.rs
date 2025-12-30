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

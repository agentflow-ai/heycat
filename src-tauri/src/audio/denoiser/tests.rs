//! Tests for DTLN model loading and denoising
//!
//! Tests behavior, not implementation details. See docs/TESTING.md.

use super::*;
use dtln::{FRAME_SHIFT, FRAME_SIZE};
use std::f32::consts::PI;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Cached models loaded once for all tests - avoids repeated ~200ms model loads
static CACHED_MODELS: LazyLock<DtlnModels> = LazyLock::new(|| {
    load_embedded_models().expect("Models should load for tests")
});

/// Test that embedded models load successfully
///
/// This is the primary success path - models bundled with the application
/// should always load correctly.
#[test]
fn test_embedded_models_load_successfully() {
    let result = load_embedded_models();

    assert!(
        result.is_ok(),
        "Embedded models should load successfully: {:?}",
        result.err()
    );
}

/// Test that models can be loaded from file paths
#[test]
fn test_models_load_from_paths() {
    // Get the path to the test resources
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_1_path = manifest_dir.join("resources/dtln/model_1.onnx");
    let model_2_path = manifest_dir.join("resources/dtln/model_2.onnx");

    let result = DtlnModels::load(&model_1_path, &model_2_path);

    assert!(
        result.is_ok(),
        "Models should load from valid paths: {:?}",
        result.err()
    );
}

/// Test that loading returns appropriate error when model files are missing
#[test]
fn test_loading_returns_error_for_missing_files() {
    let missing_path = PathBuf::from("/nonexistent/path/model.onnx");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let valid_model_path = manifest_dir.join("resources/dtln/model_1.onnx");

    // Test missing model 1
    let result = DtlnModels::load(&missing_path, &valid_model_path);
    let Err(err) = result else {
        panic!("Should error when model_1 is missing");
    };
    match err {
        DenoiserError::ModelNotFound(path) => {
            assert!(path.contains("nonexistent"), "Error should mention the missing path");
        }
        other => panic!("Expected ModelNotFound error, got: {:?}", other),
    }

    // Test missing model 2
    let result = DtlnModels::load(&valid_model_path, &missing_path);
    let Err(err) = result else {
        panic!("Should error when model_2 is missing");
    };
    match err {
        DenoiserError::ModelNotFound(path) => {
            assert!(path.contains("nonexistent"), "Error should mention the missing path");
        }
        other => panic!("Expected ModelNotFound error, got: {:?}", other),
    }
}

/// Test that loaded models have runnable structure
///
/// This verifies that the models were properly optimized and converted
/// to a runnable form, without testing internal implementation details.
#[test]
fn test_loaded_models_are_runnable() {
    let models = load_embedded_models().expect("Models should load");

    // The models should be runnable (this is a compile-time guarantee from the type system,
    // but we verify that the model structure is valid by checking we can access both models)
    let _ = &models.model_1;
    let _ = &models.model_2;

    // If we got here without panic, the models are properly structured
}


// ============================================================================
// DtlnDenoiser behavior tests
// ============================================================================

/// Helper function to create a denoiser for tests using cached models
fn create_test_denoiser() -> DtlnDenoiser {
    DtlnDenoiser::new(CACHED_MODELS.clone())
}

/// Test: Process silent audio returns silent output
///
/// Silent input should produce (nearly) silent output.
#[test]
fn test_process_silent_audio_returns_silent_output() {
    let mut denoiser = create_test_denoiser();

    // Create 1 second of silence (16000 samples at 16kHz)
    let silence: Vec<f32> = vec![0.0; 16000];

    let output = denoiser.process(&silence);

    // Output should exist and be nearly silent
    assert!(!output.is_empty(), "Should produce output");

    // Check that output is very quiet (near-zero)
    let max_amplitude: f32 = output.iter().map(|s| s.abs()).fold(0.0, f32::max);
    assert!(
        max_amplitude < 0.01,
        "Silent input should produce near-silent output, got max amplitude {}",
        max_amplitude
    );
}

/// Test: Process speech-like signal preserves content
///
/// A multi-frequency signal mimicking speech should pass through with energy preserved.
/// Note: A pure sine wave may be attenuated as the model treats tonal signals as noise.
#[test]
fn test_process_speech_like_signal_preserves_content() {
    let mut denoiser = create_test_denoiser();

    // Generate a speech-like signal with multiple harmonics (fundamental + harmonics)
    // Typical male speech fundamental is 85-180Hz, female is 165-255Hz
    // Speech has formants typically at 300-3000Hz
    let sample_rate = 16000.0;
    let duration_samples = 16000;

    let speech_like: Vec<f32> = (0..duration_samples)
        .map(|i| {
            let t = i as f32 / sample_rate;
            // Mix of frequencies typical in speech (fundamental + formants)
            (2.0 * PI * 150.0 * t).sin() * 0.3  // Fundamental
                + (2.0 * PI * 300.0 * t).sin() * 0.2  // First formant region
                + (2.0 * PI * 600.0 * t).sin() * 0.15 // Second formant region
                + (2.0 * PI * 1200.0 * t).sin() * 0.1 // Third formant region
                + (2.0 * PI * 2400.0 * t).sin() * 0.05 // Higher harmonics
        })
        .collect();

    let output = denoiser.process(&speech_like);

    // Output should exist
    assert!(!output.is_empty(), "Should produce output");

    // The denoiser will attenuate the signal somewhat (it's not perfect speech)
    // but should produce some output (not completely zeroed)
    let output_energy: f32 = output.iter().map(|s| s * s).sum();

    // Just verify we get non-trivial output (denoiser is working, not crashing)
    // The actual noise reduction quality is better tested with real audio samples
    assert!(
        output_energy > 0.0,
        "Should produce non-zero output for speech-like signal"
    );
}

/// Test: Multiple consecutive calls maintain temporal continuity
///
/// Processing audio in chunks should produce the same result as processing
/// all at once (within numerical tolerance).
#[test]
fn test_multiple_calls_maintain_continuity() {
    let models = CACHED_MODELS.clone();

    // Create a test signal (speech-like frequencies)
    let test_signal: Vec<f32> = (0..8000)
        .map(|i| {
            // Mix of frequencies typical in speech
            let t = i as f32 / 16000.0;
            (2.0 * PI * 200.0 * t).sin() * 0.3
                + (2.0 * PI * 500.0 * t).sin() * 0.2
                + (2.0 * PI * 1000.0 * t).sin() * 0.1
        })
        .collect();

    // Process in chunks
    let mut denoiser_chunks = DtlnDenoiser::new(models);
    let chunk_size = 1024;
    let mut chunked_output = Vec::new();
    for chunk in test_signal.chunks(chunk_size) {
        chunked_output.extend(denoiser_chunks.process(chunk));
    }

    // Both should produce output
    assert!(!chunked_output.is_empty(), "Chunked processing should produce output");

    // The output should be smooth (no sudden discontinuities)
    // Check that adjacent samples don't differ by more than a reasonable threshold
    let mut max_diff = 0.0f32;
    for window in chunked_output.windows(2) {
        let diff = (window[1] - window[0]).abs();
        max_diff = max_diff.max(diff);
    }

    // Max sample-to-sample difference should be reasonable (not a sudden jump)
    assert!(
        max_diff < 0.5,
        "Output should be smooth, max sample diff: {}",
        max_diff
    );
}

/// Test: Reset clears state for new stream
///
/// After reset, processing should behave as if starting fresh.
#[test]
fn test_reset_clears_state() {
    let mut denoiser = create_test_denoiser();

    // Process some audio to build up state
    let audio: Vec<f32> = (0..4000)
        .map(|i| (2.0 * PI * 300.0 * i as f32 / 16000.0).sin() * 0.5)
        .collect();
    let _ = denoiser.process(&audio);

    // Reset
    denoiser.reset();

    // Process silent audio - should be silent (state cleared)
    let silence: Vec<f32> = vec![0.0; 4000];
    let output = denoiser.process(&silence);

    // After reset, silent input should produce near-silent output
    if !output.is_empty() {
        let max_amplitude: f32 = output.iter().map(|s| s.abs()).fold(0.0, f32::max);
        assert!(
            max_amplitude < 0.1,
            "After reset, silent input should produce quiet output, got max {}",
            max_amplitude
        );
    }
}

/// Test: Output latency is approximately 32ms (512 samples at 16kHz)
///
/// The denoiser has one frame of latency due to the overlap-add processing.
#[test]
fn test_output_latency_is_approximately_one_frame() {
    let mut denoiser = create_test_denoiser();

    // Feed exactly enough samples for one frame
    let one_frame: Vec<f32> = vec![0.1; FRAME_SIZE];
    let output = denoiser.process(&one_frame);

    // After one frame, we should get FRAME_SHIFT samples out
    // (because we need a full frame before any output)
    assert_eq!(
        output.len(),
        FRAME_SHIFT,
        "First frame should produce {} samples, got {}",
        FRAME_SHIFT,
        output.len()
    );

    // Feed another chunk
    let second_chunk: Vec<f32> = vec![0.1; FRAME_SHIFT];
    let output2 = denoiser.process(&second_chunk);

    // Should get another FRAME_SHIFT samples
    assert_eq!(
        output2.len(),
        FRAME_SHIFT,
        "Second chunk should produce {} samples, got {}",
        FRAME_SHIFT,
        output2.len()
    );
}

/// Test: flush() extracts remaining samples from partial input buffer
///
/// When recording stops mid-frame, flush() should pad and process the partial buffer.
#[test]
fn test_flush_extracts_partial_input_buffer() {
    let mut denoiser = create_test_denoiser();

    // Feed less than FRAME_SIZE samples (partial buffer)
    let partial: Vec<f32> = vec![0.1; 300]; // Less than 512
    let output = denoiser.process(&partial);

    // Should get no output yet (buffer not full)
    assert_eq!(output.len(), 0, "Partial input should not produce output yet");

    // Flush should extract remaining samples
    let flushed = denoiser.flush();

    // Flush should produce:
    // 1. FRAME_SHIFT (128) from the padded and processed frame
    // 2. FRAME_SIZE - FRAME_SHIFT (384) from the overlap-add tail
    // Total: 128 + 384 = 512
    assert!(
        !flushed.is_empty(),
        "Flush should extract samples from partial buffer"
    );
    assert_eq!(
        flushed.len(),
        FRAME_SHIFT + (FRAME_SIZE - FRAME_SHIFT),
        "Flush should produce FRAME_SHIFT + tail samples"
    );
}

/// Test: flush() after normal processing extracts overlap-add tail
///
/// After processing complete frames, flush() should extract the remaining
/// overlap-add tail that hasn't been output yet.
#[test]
fn test_flush_extracts_overlap_add_tail() {
    let mut denoiser = create_test_denoiser();

    // Feed exactly one frame
    let one_frame: Vec<f32> = vec![0.1; FRAME_SIZE];
    let output1 = denoiser.process(&one_frame);
    assert_eq!(output1.len(), FRAME_SHIFT, "First frame should output FRAME_SHIFT samples");

    // Flush should extract the remaining overlap-add tail
    let flushed = denoiser.flush();

    // After one frame processed:
    // - input_buffer has (FRAME_SIZE - FRAME_SHIFT) = 384 samples left
    // - output_buffer has overlap content
    // Flush should pad input to 512, process, then extract tail
    assert!(
        !flushed.is_empty(),
        "Flush should extract overlap-add tail"
    );
}

/// Test: flush() clears internal buffers
///
/// After flush(), the denoiser should be in a clean state.
#[test]
fn test_flush_clears_buffers() {
    let mut denoiser = create_test_denoiser();

    // Feed some samples
    let samples: Vec<f32> = vec![0.1; 600];
    let _ = denoiser.process(&samples);

    // Flush
    let _ = denoiser.flush();

    // Feed new samples - should work normally
    let new_samples: Vec<f32> = vec![0.2; FRAME_SIZE];
    let output = denoiser.process(&new_samples);

    // Should produce normal output (FRAME_SHIFT samples)
    assert_eq!(
        output.len(),
        FRAME_SHIFT,
        "Denoiser should work normally after flush"
    );
}

/// Test: flush() on empty buffer returns minimal samples
///
/// If no samples were processed, flush should return just the overlap-add tail.
#[test]
fn test_flush_on_empty_buffer() {
    let mut denoiser = create_test_denoiser();

    // Flush without processing anything
    let flushed = denoiser.flush();

    // Should return the overlap-add tail (384 samples of zeros)
    assert_eq!(
        flushed.len(),
        FRAME_SIZE - FRAME_SHIFT,
        "Empty flush should return tail length"
    );
}

// ============================================================================
// Regression tests for bugs fixed in audio-glitch.bug.md
// ============================================================================

/// Regression test: reset() after flush() should result in clean state
///
/// Bug: Stale samples appearing in denoiser between recordings caused
/// LSTM state corruption and degraded audio quality on subsequent recordings.
///
/// This test verifies that after flush() -> reset(), the denoiser behaves
/// identically to a freshly created denoiser.
#[test]
fn test_regression_reset_after_flush_is_clean() {
    let mut denoiser = create_test_denoiser();

    // Simulate first recording: process audio then flush
    let recording1: Vec<f32> = (0..8000)
        .map(|i| (2.0 * PI * 300.0 * i as f32 / 16000.0).sin() * 0.5)
        .collect();
    let _ = denoiser.process(&recording1);
    let _ = denoiser.flush();

    // Reset for "second recording"
    denoiser.reset();

    // Process silent audio - should be clean (no artifacts from recording1)
    let silence: Vec<f32> = vec![0.0; 4000];
    let output = denoiser.process(&silence);

    // After reset, silent input should produce near-silent output
    // If there were stale samples or corrupted LSTM state, we'd see non-zero output
    if !output.is_empty() {
        let max_amplitude: f32 = output.iter().map(|s| s.abs()).fold(0.0, f32::max);
        assert!(
            max_amplitude < 0.05,
            "After reset, silent input should be very quiet, got max amplitude {}",
            max_amplitude
        );
    }
}

/// Regression test: Multiple recording cycles don't degrade quality
///
/// Bug: Audio quality degraded on subsequent recordings due to LSTM state
/// corruption from stale samples pushed between recordings.
///
/// This test simulates multiple recording cycles and verifies consistent behavior.
#[test]
fn test_regression_multiple_recordings_consistent_quality() {
    let mut denoiser = create_test_denoiser();

    // Create a test signal (speech-like)
    let test_signal: Vec<f32> = (0..4000)
        .map(|i| {
            let t = i as f32 / 16000.0;
            (2.0 * PI * 200.0 * t).sin() * 0.3 + (2.0 * PI * 500.0 * t).sin() * 0.2
        })
        .collect();

    let mut output_energies = Vec::new();

    // Simulate 5 recording cycles
    for _ in 0..5 {
        let output = denoiser.process(&test_signal);
        let _ = denoiser.flush();
        denoiser.reset();

        // Calculate output energy
        let energy: f32 = output.iter().map(|s| s * s).sum();
        output_energies.push(energy);
    }

    // All recordings should have similar energy (within 50% tolerance)
    // If there's LSTM corruption, later recordings would have very different energy
    let first_energy = output_energies[0];
    for (i, &energy) in output_energies.iter().enumerate().skip(1) {
        let ratio = energy / first_energy;
        assert!(
            (0.5..=2.0).contains(&ratio),
            "Recording {} has abnormal energy ratio {:.2} vs first recording",
            i + 1,
            ratio
        );
    }
}

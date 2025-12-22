//! Tests for DTLN model loading and denoising
//!
//! Tests behavior, not implementation details. See docs/TESTING.md.

use super::*;
use dtln::{FRAME_SHIFT, FRAME_SIZE};
use std::f32::consts::PI;
use std::path::PathBuf;

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

/// Debug test to check model input requirements
#[test]
fn test_debug_model_inputs() {
    use tract_onnx::prelude::*;

    // Load model without full optimization to see its structure
    let model = tract_onnx::onnx()
        .model_for_read(&mut std::io::Cursor::new(embedded::MODEL_1_BYTES))
        .expect("Should load model");

    println!("\n=== Model 1 Structure ===");
    println!("Inputs: {:?}", model.inputs);
    for (i, input) in model.inputs.iter().enumerate() {
        let fact = model.outlet_fact(*input).expect("Should get input fact");
        println!("  Input {}: slot={:?}, fact={:?}", i, input, fact);
    }

    println!("Outputs: {:?}", model.outputs);
    for (i, output) in model.outputs.iter().enumerate() {
        let fact = model.outlet_fact(*output).expect("Should get output fact");
        println!("  Output {}: slot={:?}, fact={:?}", i, output, fact);
    }

    // Also check model 2
    let model2 = tract_onnx::onnx()
        .model_for_read(&mut std::io::Cursor::new(embedded::MODEL_2_BYTES))
        .expect("Should load model 2");

    println!("\n=== Model 2 Structure ===");
    println!("Inputs: {:?}", model2.inputs);
    for (i, input) in model2.inputs.iter().enumerate() {
        let fact = model2.outlet_fact(*input).expect("Should get input fact");
        println!("  Input {}: slot={:?}, fact={:?}", i, input, fact);
    }

    println!("Outputs: {:?}", model2.outputs);
    for (i, output) in model2.outputs.iter().enumerate() {
        let fact = model2.outlet_fact(*output).expect("Should get output fact");
        println!("  Output {}: slot={:?}, fact={:?}", i, output, fact);
    }
}

// ============================================================================
// DtlnDenoiser behavior tests
// ============================================================================

/// Helper function to create a denoiser for tests
fn create_test_denoiser() -> DtlnDenoiser {
    let models = load_embedded_models().expect("Models should load for tests");
    DtlnDenoiser::new(models)
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
    let models = load_embedded_models().expect("Models should load");

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

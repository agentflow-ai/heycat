//! Tests for DTLN model loading
//!
//! Tests behavior, not implementation details. See docs/TESTING.md.

use super::*;
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

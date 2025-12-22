//! DTLN (Dual-signal Transformation LSTM Network) noise suppression module
//!
//! This module provides real-time audio noise suppression using the DTLN model
//! via ONNX inference. The model operates at 16kHz native sample rate and uses
//! two-stage processing: magnitude masking followed by time-domain refinement.

use std::path::Path;
use thiserror::Error;
use tract_onnx::prelude::*;

mod dtln;
pub use dtln::{DtlnDenoiser, FRAME_SHIFT, FRAME_SIZE, FFT_BINS};

#[cfg(test)]
mod tests;

/// Errors that can occur during denoiser operations
#[derive(Debug, Error)]
pub enum DenoiserError {
    /// Failed to load ONNX model file
    #[error("Failed to load model from {path}: {source}")]
    ModelLoadError {
        path: String,
        #[source]
        source: TractError,
    },

    /// Model file not found
    #[error("Model file not found: {0}")]
    ModelNotFound(String),

    /// Failed to optimize model
    #[error("Failed to optimize model: {0}")]
    ModelOptimizationError(String),
}

/// Type alias for tract's typed model
pub type TypedModel = tract_onnx::prelude::TypedModel;

/// Type alias for a runnable model (optimized for inference)
pub type RunnableModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

/// Loaded DTLN models ready for inference
///
/// DTLN uses two models:
/// - Model 1: Magnitude masking in frequency domain (LSTM-based)
/// - Model 2: Time-domain refinement (LSTM-based)
///
/// Both models have LSTM states that must be tracked between frames.
pub struct DtlnModels {
    /// Stage 1 model: Magnitude masking
    pub model_1: RunnableModel,
    /// Stage 2 model: Time-domain refinement
    pub model_2: RunnableModel,
}

impl DtlnModels {
    /// Load DTLN models from ONNX files
    ///
    /// # Arguments
    /// * `model_1_path` - Path to model_1.onnx (magnitude masking)
    /// * `model_2_path` - Path to model_2.onnx (time-domain refinement)
    ///
    /// # Returns
    /// * `Ok(DtlnModels)` - Successfully loaded and optimized models
    /// * `Err(DenoiserError)` - If loading or optimization fails
    pub fn load<P: AsRef<Path>>(model_1_path: P, model_2_path: P) -> Result<Self, DenoiserError> {
        let model_1_path = model_1_path.as_ref();
        let model_2_path = model_2_path.as_ref();

        // Check if model files exist
        if !model_1_path.exists() {
            return Err(DenoiserError::ModelNotFound(
                model_1_path.to_string_lossy().to_string(),
            ));
        }
        if !model_2_path.exists() {
            return Err(DenoiserError::ModelNotFound(
                model_2_path.to_string_lossy().to_string(),
            ));
        }

        // Load model 1
        let model_1 = Self::load_and_optimize_model(model_1_path)?;

        // Load model 2
        let model_2 = Self::load_and_optimize_model(model_2_path)?;

        Ok(Self { model_1, model_2 })
    }

    /// Load DTLN models from embedded bytes
    ///
    /// This is useful when models are bundled into the binary via `include_bytes!`
    ///
    /// # Arguments
    /// * `model_1_bytes` - Bytes of model_1.onnx
    /// * `model_2_bytes` - Bytes of model_2.onnx
    ///
    /// # Returns
    /// * `Ok(DtlnModels)` - Successfully loaded and optimized models
    /// * `Err(DenoiserError)` - If loading or optimization fails
    pub fn load_from_bytes(model_1_bytes: &[u8], model_2_bytes: &[u8]) -> Result<Self, DenoiserError> {
        let model_1 = Self::load_and_optimize_model_from_bytes(model_1_bytes, "model_1")?;
        let model_2 = Self::load_and_optimize_model_from_bytes(model_2_bytes, "model_2")?;

        Ok(Self { model_1, model_2 })
    }

    /// Load and optimize a single ONNX model from file
    fn load_and_optimize_model(path: &Path) -> Result<RunnableModel, DenoiserError> {
        let model = tract_onnx::onnx()
            .model_for_path(path)
            .map_err(|e| DenoiserError::ModelLoadError {
                path: path.to_string_lossy().to_string(),
                source: e,
            })?
            .into_optimized()
            .map_err(|e| DenoiserError::ModelOptimizationError(e.to_string()))?
            .into_runnable()
            .map_err(|e| DenoiserError::ModelOptimizationError(e.to_string()))?;

        Ok(model)
    }

    /// Load and optimize a single ONNX model from bytes
    fn load_and_optimize_model_from_bytes(bytes: &[u8], name: &str) -> Result<RunnableModel, DenoiserError> {
        let model = tract_onnx::onnx()
            .model_for_read(&mut std::io::Cursor::new(bytes))
            .map_err(|e| DenoiserError::ModelLoadError {
                path: name.to_string(),
                source: e,
            })?
            .into_optimized()
            .map_err(|e| DenoiserError::ModelOptimizationError(e.to_string()))?
            .into_runnable()
            .map_err(|e| DenoiserError::ModelOptimizationError(e.to_string()))?;

        Ok(model)
    }
}

/// Embedded DTLN model bytes (bundled at compile time)
pub mod embedded {
    /// Model 1 ONNX bytes (magnitude masking)
    pub static MODEL_1_BYTES: &[u8] = include_bytes!("../../../resources/dtln/model_1.onnx");

    /// Model 2 ONNX bytes (time-domain refinement)
    pub static MODEL_2_BYTES: &[u8] = include_bytes!("../../../resources/dtln/model_2.onnx");
}

/// Load embedded DTLN models
///
/// Convenience function that loads models from compile-time embedded bytes.
/// This is the recommended way to load models in production.
///
/// # Returns
/// * `Ok(DtlnModels)` - Successfully loaded models
/// * `Err(DenoiserError)` - If model loading fails
pub fn load_embedded_models() -> Result<DtlnModels, DenoiserError> {
    DtlnModels::load_from_bytes(embedded::MODEL_1_BYTES, embedded::MODEL_2_BYTES)
}

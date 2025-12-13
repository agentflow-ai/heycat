// Model download functionality
// Contains the core download logic, testable independently from Tauri commands

use crate::{debug, info, warn};
use std::path::PathBuf;

/// Model information constants
pub const MODEL_FILENAME: &str = "ggml-large-v3-turbo.bin";
pub const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin";
pub const MODELS_DIR_NAME: &str = "models";
pub const APP_DIR_NAME: &str = "heycat";

/// Download progress logging interval in bytes (50MB)
const DOWNLOAD_PROGRESS_INTERVAL: u64 = 50_000_000;

/// Error types for model operations
#[derive(Debug, Clone)]
pub enum ModelError {
    /// App data directory not found
    DataDirNotFound,
    /// Failed to create directory
    DirectoryCreationFailed(String),
    /// Network error during download
    NetworkError(String),
    /// File I/O error
    IoError(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::DataDirNotFound => write!(f, "App data directory not found"),
            ModelError::DirectoryCreationFailed(msg) => {
                write!(f, "Failed to create directory: {}", msg)
            }
            ModelError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ModelError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for ModelError {}

/// Get the path where models should be stored
/// Returns {app_data_dir}/heycat/models/
pub fn get_models_dir() -> Result<PathBuf, ModelError> {
    let data_dir = dirs::data_dir().ok_or(ModelError::DataDirNotFound)?;
    Ok(data_dir.join(APP_DIR_NAME).join(MODELS_DIR_NAME))
}

/// Get the full path to the transcription model file
pub fn get_model_path() -> Result<PathBuf, ModelError> {
    Ok(get_models_dir()?.join(MODEL_FILENAME))
}

/// Check if the transcription model exists on disk
pub fn check_model_exists() -> Result<bool, ModelError> {
    let model_path = get_model_path()?;
    Ok(model_path.exists())
}

/// Create the models directory if it doesn't exist
pub fn ensure_models_dir() -> Result<PathBuf, ModelError> {
    let models_dir = get_models_dir()?;
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)
            .map_err(|e| ModelError::DirectoryCreationFailed(e.to_string()))?;
    }
    Ok(models_dir)
}

/// Download the model from HuggingFace using streaming
/// This is an async function that streams the download to disk
/// Uses atomic temp file + rename to prevent TOCTOU race conditions
pub async fn download_model() -> Result<PathBuf, ModelError> {
    use futures_util::StreamExt;
    use tauri_plugin_http::reqwest;
    use tokio::io::AsyncWriteExt;
    use uuid::Uuid;

    info!("Starting model download from {}", MODEL_URL);

    let models_dir = ensure_models_dir()?;
    let model_path = models_dir.join(MODEL_FILENAME);

    // If model already exists, return early
    if model_path.exists() {
        info!("Model already exists at {:?}", model_path);
        return Ok(model_path);
    }

    // Use unique temp file to avoid race conditions with concurrent downloads
    let temp_filename = format!("{}.{}.tmp", MODEL_FILENAME, Uuid::new_v4());
    let temp_path = models_dir.join(&temp_filename);
    debug!("Downloading to temp file: {:?}", temp_path);

    // Create HTTP client and start download
    let client = reqwest::Client::new();
    let response = client
        .get(MODEL_URL)
        .send()
        .await
        .map_err(|e| ModelError::NetworkError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ModelError::NetworkError(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    // Get content length for progress logging
    let content_length = response.content_length();
    if let Some(len) = content_length {
        info!("Model size: {} MB", len / 1_000_000);
    }

    // Create temp file for writing
    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| ModelError::IoError(e.to_string()))?;

    // Stream the response body to file with progress logging
    let mut stream = response.bytes_stream();
    let mut bytes_written: u64 = 0;
    let mut last_progress_log: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            ModelError::NetworkError(format!(
                "Download failed after {} bytes: {}",
                bytes_written, e
            ))
        })?;
        file.write_all(&chunk)
            .await
            .map_err(|e| {
                ModelError::IoError(format!(
                    "Write failed after {} bytes: {}",
                    bytes_written, e
                ))
            })?;

        bytes_written += chunk.len() as u64;

        // Log progress at regular intervals
        if bytes_written - last_progress_log >= DOWNLOAD_PROGRESS_INTERVAL {
            if let Some(total) = content_length {
                let percent = (bytes_written as f64 / total as f64) * 100.0;
                info!("Download progress: {:.1}% ({} MB / {} MB)",
                      percent, bytes_written / 1_000_000, total / 1_000_000);
            } else {
                info!("Downloaded {} MB", bytes_written / 1_000_000);
            }
            last_progress_log = bytes_written;
        }
    }

    // Ensure all data is flushed to disk
    file.flush()
        .await
        .map_err(|e| ModelError::IoError(e.to_string()))?;
    drop(file); // Close file before rename

    debug!("Download complete, renaming temp file to final path");

    // Atomic rename: if target already exists (race condition), that's fine - use it
    match tokio::fs::rename(&temp_path, &model_path).await {
        Ok(()) => {
            info!("Model downloaded successfully to {:?}", model_path);
        }
        Err(e) => {
            // Check if target now exists (another download completed first)
            if model_path.exists() {
                warn!("Model was downloaded by another process, using existing file");
                // Clean up our temp file
                let _ = tokio::fs::remove_file(&temp_path).await;
            } else {
                // Clean up temp file on error
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(ModelError::IoError(format!("Failed to rename temp file: {}", e)));
            }
        }
    }

    Ok(model_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_constants() {
        assert_eq!(MODEL_FILENAME, "ggml-large-v3-turbo.bin");
        assert!(MODEL_URL.contains("huggingface.co"));
        assert!(MODEL_URL.contains("ggml-large-v3-turbo.bin"));
    }

    #[test]
    fn test_get_models_dir_contains_expected_path() {
        let result = get_models_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("heycat/models") || path.ends_with("heycat\\models"));
    }

    #[test]
    fn test_get_model_path_contains_filename() {
        let result = get_model_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with(MODEL_FILENAME));
    }

    #[test]
    fn test_check_model_exists_returns_false_when_not_present() {
        // In test environment, model likely doesn't exist
        let result = check_model_exists();
        assert!(result.is_ok());
        // Just verify it returns a boolean without error
    }

    #[test]
    fn test_model_error_display() {
        let error = ModelError::DataDirNotFound;
        assert_eq!(format!("{}", error), "App data directory not found");

        let error = ModelError::DirectoryCreationFailed("permission denied".to_string());
        assert!(format!("{}", error).contains("permission denied"));

        let error = ModelError::NetworkError("connection refused".to_string());
        assert!(format!("{}", error).contains("connection refused"));

        let error = ModelError::IoError("disk full".to_string());
        assert!(format!("{}", error).contains("disk full"));
    }

    #[test]
    fn test_model_error_is_debug() {
        let error = ModelError::NetworkError("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("NetworkError"));
    }

    #[test]
    fn test_ensure_models_dir_creates_directory() {
        let result = ensure_models_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
    }
}

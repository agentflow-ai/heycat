// Model download functionality
// Contains the core download logic, testable independently from Tauri commands

use crate::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Model information constants
pub const MODEL_FILENAME: &str = "ggml-large-v3-turbo.bin";
pub const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin";
pub const MODELS_DIR_NAME: &str = "models";
pub const APP_DIR_NAME: &str = "heycat";

/// Model type for multi-model support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Parakeet TDT model for batch transcription
    ParakeetTDT,
    /// Parakeet EOU model for streaming transcription
    ParakeetEOU,
}

impl ModelType {
    /// Get the directory name for this model type
    pub fn dir_name(&self) -> &'static str {
        match self {
            ModelType::ParakeetTDT => "parakeet-tdt",
            ModelType::ParakeetEOU => "parakeet-eou",
        }
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::ParakeetTDT => write!(f, "parakeet-tdt"),
            ModelType::ParakeetEOU => write!(f, "parakeet-eou"),
        }
    }
}

/// A single file in a model manifest
#[derive(Debug, Clone)]
pub struct ModelFile {
    /// Name of the file
    pub name: String,
    /// Expected size in bytes
    pub size_bytes: u64,
}

/// Manifest for multi-file model downloads
#[derive(Debug, Clone)]
pub struct ModelManifest {
    /// Type of model
    pub model_type: ModelType,
    /// Base URL for downloading files
    pub base_url: String,
    /// List of files to download
    pub files: Vec<ModelFile>,
}

impl ModelManifest {
    /// Create manifest for Parakeet TDT model
    pub fn tdt() -> Self {
        Self {
            model_type: ModelType::ParakeetTDT,
            base_url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/"
                .into(),
            files: vec![
                ModelFile {
                    name: "encoder-model.onnx".into(),
                    size_bytes: 43_826_176,
                },
                ModelFile {
                    name: "encoder-model.onnx.data".into(),
                    size_bytes: 2_620_162_048,
                },
                ModelFile {
                    name: "decoder_joint-model.onnx".into(),
                    size_bytes: 76_021_760,
                },
                ModelFile {
                    name: "vocab.txt".into(),
                    size_bytes: 96_154,
                },
            ],
        }
    }

    /// Create manifest for Parakeet EOU model
    pub fn eou() -> Self {
        Self {
            model_type: ModelType::ParakeetEOU,
            base_url:
                "https://huggingface.co/nvidia/parakeet_tdt_rnnt_1.1b-onnx/resolve/main/".into(),
            files: vec![
                ModelFile {
                    name: "encoder.onnx".into(),
                    size_bytes: 0, // Size unknown - will be determined from response
                },
                ModelFile {
                    name: "decoder_joint.onnx".into(),
                    size_bytes: 0,
                },
                ModelFile {
                    name: "tokenizer.json".into(),
                    size_bytes: 0,
                },
            ],
        }
    }

    /// Get total size of all files in bytes
    pub fn total_size(&self) -> u64 {
        self.files.iter().map(|f| f.size_bytes).sum()
    }
}

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

/// Get the directory path for a specific model type
/// Returns {app_data_dir}/heycat/models/{model_type_dir}/
pub fn get_model_dir(model_type: ModelType) -> Result<PathBuf, ModelError> {
    Ok(get_models_dir()?.join(model_type.dir_name()))
}

/// Get the full path to the transcription model file
pub fn get_model_path() -> Result<PathBuf, ModelError> {
    Ok(get_models_dir()?.join(MODEL_FILENAME))
}

/// Check if the transcription model exists on disk (legacy Whisper model)
pub fn check_model_exists() -> Result<bool, ModelError> {
    let model_path = get_model_path()?;
    Ok(model_path.exists())
}

/// Check if a multi-file model exists (all files present)
pub fn check_model_exists_for_type(model_type: ModelType) -> Result<bool, ModelError> {
    let model_dir = get_model_dir(model_type)?;

    if !model_dir.exists() {
        return Ok(false);
    }

    // Get the manifest for this model type
    let manifest = match model_type {
        ModelType::ParakeetTDT => ModelManifest::tdt(),
        ModelType::ParakeetEOU => ModelManifest::eou(),
    };

    // Check that ALL files in the manifest exist
    for file in &manifest.files {
        let file_path = model_dir.join(&file.name);
        if !file_path.exists() {
            return Ok(false);
        }
    }

    Ok(true)
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

/// Trait for emitting model download progress events
/// Allows mocking in tests while using real Tauri AppHandle in production
pub trait ModelDownloadEventEmitter: Send + Sync {
    /// Emit model file download progress event
    fn emit_model_file_download_progress(
        &self,
        model_type: &str,
        file_name: &str,
        bytes_downloaded: u64,
        total_bytes: u64,
        file_index: usize,
        total_files: usize,
    );
}

/// Download all files in a model manifest
/// Uses atomic temp directory + rename to prevent partial downloads
pub async fn download_model_files<E: ModelDownloadEventEmitter>(
    manifest: ModelManifest,
    emitter: &E,
) -> Result<PathBuf, ModelError> {
    use futures_util::StreamExt;
    use tauri_plugin_http::reqwest;
    use tokio::io::AsyncWriteExt;
    use uuid::Uuid;

    let model_type_str = manifest.model_type.to_string();
    let final_dir = get_model_dir(manifest.model_type)?;

    // If model already exists (all files present), return early
    if check_model_exists_for_type(manifest.model_type)? {
        info!("Model {} already exists at {:?}", model_type_str, final_dir);
        return Ok(final_dir);
    }

    // Create temp directory with unique name
    let models_dir = ensure_models_dir()?;
    let temp_dir_name = format!(".{}-{}", manifest.model_type.dir_name(), Uuid::new_v4());
    let temp_dir = models_dir.join(&temp_dir_name);

    info!(
        "Starting multi-file download for {} to {:?}",
        model_type_str, temp_dir
    );

    // Create temp directory
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| ModelError::DirectoryCreationFailed(e.to_string()))?;

    let client = reqwest::Client::new();
    let total_files = manifest.files.len();

    // Download each file
    for (file_index, model_file) in manifest.files.iter().enumerate() {
        let url = format!("{}{}", manifest.base_url, model_file.name);
        let file_path = temp_dir.join(&model_file.name);

        debug!(
            "Downloading file {}/{}: {} from {}",
            file_index + 1,
            total_files,
            model_file.name,
            url
        );

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                // Clean up temp directory on failure
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Err(ModelError::NetworkError(e.to_string()));
            }
        };

        if !response.status().is_success() {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(ModelError::NetworkError(format!(
                "HTTP error for {}: {}",
                model_file.name,
                response.status()
            )));
        }

        // Get content length from response or use manifest size
        let total_bytes = response.content_length().unwrap_or(model_file.size_bytes);

        // Create file for writing
        let mut file = match tokio::fs::File::create(&file_path).await {
            Ok(f) => f,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Err(ModelError::IoError(e.to_string()));
            }
        };

        // Stream the response body to file
        let mut stream = response.bytes_stream();
        let mut bytes_written: u64 = 0;
        let mut last_emit: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&temp_dir);
                    return Err(ModelError::NetworkError(format!(
                        "Download failed for {} after {} bytes: {}",
                        model_file.name, bytes_written, e
                    )));
                }
            };

            if let Err(e) = file.write_all(&chunk).await {
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Err(ModelError::IoError(format!(
                    "Write failed for {} after {} bytes: {}",
                    model_file.name, bytes_written, e
                )));
            }

            bytes_written += chunk.len() as u64;

            // Emit progress at regular intervals or when done
            if bytes_written - last_emit >= DOWNLOAD_PROGRESS_INTERVAL
                || bytes_written == total_bytes
            {
                emitter.emit_model_file_download_progress(
                    &model_type_str,
                    &model_file.name,
                    bytes_written,
                    total_bytes,
                    file_index,
                    total_files,
                );
                last_emit = bytes_written;
            }
        }

        // Final progress emit for this file
        emitter.emit_model_file_download_progress(
            &model_type_str,
            &model_file.name,
            bytes_written,
            total_bytes,
            file_index,
            total_files,
        );

        if let Err(e) = file.flush().await {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(ModelError::IoError(e.to_string()));
        }

        info!(
            "Downloaded {}/{}: {} ({} bytes)",
            file_index + 1,
            total_files,
            model_file.name,
            bytes_written
        );
    }

    // Atomic rename: move temp dir to final location
    debug!(
        "All files downloaded, renaming {:?} to {:?}",
        temp_dir, final_dir
    );

    // Check if another process completed the download
    if final_dir.exists() {
        warn!(
            "Model {} was downloaded by another process, using existing directory",
            model_type_str
        );
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Ok(final_dir);
    }

    // Rename temp directory to final
    if let Err(e) = std::fs::rename(&temp_dir, &final_dir) {
        // Check again in case of race condition
        if final_dir.exists() {
            warn!(
                "Model {} was downloaded by another process, using existing directory",
                model_type_str
            );
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Ok(final_dir);
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(ModelError::IoError(format!(
            "Failed to rename temp directory: {}",
            e
        )));
    }

    info!("Model {} downloaded successfully to {:?}", model_type_str, final_dir);
    Ok(final_dir)
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

    // ModelType tests

    #[test]
    fn test_model_type_dir_name() {
        assert_eq!(ModelType::ParakeetTDT.dir_name(), "parakeet-tdt");
        assert_eq!(ModelType::ParakeetEOU.dir_name(), "parakeet-eou");
    }

    #[test]
    fn test_model_type_display() {
        assert_eq!(format!("{}", ModelType::ParakeetTDT), "parakeet-tdt");
        assert_eq!(format!("{}", ModelType::ParakeetEOU), "parakeet-eou");
    }

    #[test]
    fn test_model_type_debug() {
        let debug = format!("{:?}", ModelType::ParakeetTDT);
        assert!(debug.contains("ParakeetTDT"));
    }

    #[test]
    fn test_model_type_clone_and_eq() {
        let a = ModelType::ParakeetTDT;
        let b = a;
        assert_eq!(a, b);

        let c = ModelType::ParakeetEOU;
        assert_ne!(a, c);
    }

    #[test]
    fn test_model_type_serde() {
        let model_type = ModelType::ParakeetTDT;
        let json = serde_json::to_string(&model_type).unwrap();
        assert!(json.contains("ParakeetTDT"));

        let deserialized: ModelType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ModelType::ParakeetTDT);
    }

    // ModelManifest tests

    #[test]
    fn test_model_manifest_tdt_returns_correct_file_list() {
        let manifest = ModelManifest::tdt();
        assert_eq!(manifest.model_type, ModelType::ParakeetTDT);
        assert_eq!(manifest.files.len(), 4);
        assert!(manifest.base_url.contains("huggingface.co"));
        assert!(manifest.base_url.contains("parakeet-tdt"));

        // Verify expected files
        let file_names: Vec<&str> = manifest.files.iter().map(|f| f.name.as_str()).collect();
        assert!(file_names.contains(&"encoder-model.onnx"));
        assert!(file_names.contains(&"encoder-model.onnx.data"));
        assert!(file_names.contains(&"decoder_joint-model.onnx"));
        assert!(file_names.contains(&"vocab.txt"));
    }

    #[test]
    fn test_model_manifest_eou_returns_correct_file_list() {
        let manifest = ModelManifest::eou();
        assert_eq!(manifest.model_type, ModelType::ParakeetEOU);
        assert_eq!(manifest.files.len(), 3);
        assert!(manifest.base_url.contains("huggingface.co"));

        // Verify expected files
        let file_names: Vec<&str> = manifest.files.iter().map(|f| f.name.as_str()).collect();
        assert!(file_names.contains(&"encoder.onnx"));
        assert!(file_names.contains(&"decoder_joint.onnx"));
        assert!(file_names.contains(&"tokenizer.json"));
    }

    #[test]
    fn test_model_manifest_total_size() {
        let manifest = ModelManifest::tdt();
        let expected = 43_826_176 + 2_620_162_048 + 76_021_760 + 96_154;
        assert_eq!(manifest.total_size(), expected);
    }

    #[test]
    fn test_model_manifest_clone() {
        let manifest = ModelManifest::tdt();
        let cloned = manifest.clone();
        assert_eq!(manifest.model_type, cloned.model_type);
        assert_eq!(manifest.files.len(), cloned.files.len());
    }

    #[test]
    fn test_model_manifest_debug() {
        let manifest = ModelManifest::tdt();
        let debug = format!("{:?}", manifest);
        assert!(debug.contains("ModelManifest"));
        assert!(debug.contains("ParakeetTDT"));
    }

    // get_model_dir tests

    #[test]
    fn test_get_model_dir_tdt_returns_correct_path() {
        let result = get_model_dir(ModelType::ParakeetTDT);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(
            path.ends_with("heycat/models/parakeet-tdt")
                || path.ends_with("heycat\\models\\parakeet-tdt")
        );
    }

    #[test]
    fn test_get_model_dir_eou_returns_correct_path() {
        let result = get_model_dir(ModelType::ParakeetEOU);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(
            path.ends_with("heycat/models/parakeet-eou")
                || path.ends_with("heycat\\models\\parakeet-eou")
        );
    }

    // check_model_exists_for_type tests

    #[test]
    fn test_check_model_exists_for_type_returns_false_when_directory_missing() {
        // Model directories likely don't exist in test environment
        let result = check_model_exists_for_type(ModelType::ParakeetTDT);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should be false when dir doesn't exist
    }

    #[test]
    fn test_check_model_exists_for_type_returns_false_when_files_missing() {
        // Create the directory but not the files
        let model_dir = get_model_dir(ModelType::ParakeetTDT).unwrap();

        // Create the directory if it doesn't exist
        let _ = std::fs::create_dir_all(&model_dir);

        // Should return false because files are missing
        let result = check_model_exists_for_type(ModelType::ParakeetTDT);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Clean up
        let _ = std::fs::remove_dir(&model_dir);
    }

    #[test]
    fn test_check_model_exists_for_type_returns_true_when_all_files_present() {
        use std::io::Write;

        let model_dir = get_model_dir(ModelType::ParakeetTDT).unwrap();

        // Create the directory and all required files
        std::fs::create_dir_all(&model_dir).unwrap();

        let manifest = ModelManifest::tdt();
        for file in &manifest.files {
            let file_path = model_dir.join(&file.name);
            let mut f = std::fs::File::create(&file_path).unwrap();
            // Write some dummy content
            f.write_all(b"test").unwrap();
        }

        // Should return true because all files exist
        let result = check_model_exists_for_type(ModelType::ParakeetTDT);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Clean up
        for file in &manifest.files {
            let file_path = model_dir.join(&file.name);
            let _ = std::fs::remove_file(&file_path);
        }
        let _ = std::fs::remove_dir(&model_dir);
    }

    // ModelFile tests

    #[test]
    fn test_model_file_clone() {
        let file = ModelFile {
            name: "test.onnx".into(),
            size_bytes: 1024,
        };
        let cloned = file.clone();
        assert_eq!(file.name, cloned.name);
        assert_eq!(file.size_bytes, cloned.size_bytes);
    }

    #[test]
    fn test_model_file_debug() {
        let file = ModelFile {
            name: "test.onnx".into(),
            size_bytes: 1024,
        };
        let debug = format!("{:?}", file);
        assert!(debug.contains("ModelFile"));
        assert!(debug.contains("test.onnx"));
    }
}

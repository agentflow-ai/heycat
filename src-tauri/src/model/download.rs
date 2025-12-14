// Model download functionality
// Contains the core download logic, testable independently from Tauri commands

use crate::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const MODELS_DIR_NAME: &str = "models";
pub const APP_DIR_NAME: &str = "heycat";

/// Model type for multi-model support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Parakeet TDT model for batch transcription
    #[serde(rename = "tdt")]
    ParakeetTDT,
    /// Parakeet EOU model for streaming transcription
    #[serde(rename = "eou")]
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
            ModelType::ParakeetTDT => write!(f, "tdt"),
            ModelType::ParakeetEOU => write!(f, "eou"),
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
    /// Uses community ONNX conversion from altunenes/parakeet-rs
    pub fn eou() -> Self {
        Self {
            model_type: ModelType::ParakeetEOU,
            base_url:
                "https://huggingface.co/altunenes/parakeet-rs/resolve/main/realtime_eou_120m-v1-onnx/".into(),
            files: vec![
                ModelFile {
                    name: "encoder.onnx".into(),
                    size_bytes: 481_296_384, // ~459 MB
                },
                ModelFile {
                    name: "decoder_joint.onnx".into(),
                    size_bytes: 22_334_054, // ~21.3 MB
                },
                ModelFile {
                    name: "tokenizer.json".into(),
                    size_bytes: 20_582, // ~20.1 KB
                },
            ],
        }
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

/// Check if all model files exist in a given directory
fn check_model_files_exist_in_dir(dir: &std::path::Path, manifest: &ModelManifest) -> bool {
    if !dir.exists() {
        return false;
    }
    manifest.files.iter().all(|f| dir.join(&f.name).exists())
}

/// Check if a multi-file model exists (all files present)
pub fn check_model_exists_for_type(model_type: ModelType) -> Result<bool, ModelError> {
    let model_dir = get_model_dir(model_type)?;
    let manifest = match model_type {
        ModelType::ParakeetTDT => ModelManifest::tdt(),
        ModelType::ParakeetEOU => ModelManifest::eou(),
    };
    Ok(check_model_files_exist_in_dir(&model_dir, &manifest))
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

    /// Get the path to models directory in the git repo (for tests)
    /// Returns {CARGO_MANIFEST_DIR}/../models/{model_type}/
    fn get_test_models_dir(model_type: ModelType) -> PathBuf {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        PathBuf::from(manifest_dir)
            .parent()
            .expect("Failed to get parent of manifest dir")
            .join("models")
            .join(model_type.dir_name())
    }

    #[test]
    fn test_get_models_dir_contains_expected_path() {
        let result = get_models_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("heycat/models") || path.ends_with("heycat\\models"));
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
        assert_eq!(format!("{}", ModelType::ParakeetTDT), "tdt");
        assert_eq!(format!("{}", ModelType::ParakeetEOU), "eou");
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
        assert_eq!(json, "\"tdt\"");

        let deserialized: ModelType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ModelType::ParakeetTDT);

        // Test EOU variant
        let eou_type = ModelType::ParakeetEOU;
        let eou_json = serde_json::to_string(&eou_type).unwrap();
        assert_eq!(eou_json, "\"eou\"");

        let eou_deserialized: ModelType = serde_json::from_str(&eou_json).unwrap();
        assert_eq!(eou_deserialized, ModelType::ParakeetEOU);
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

    // check_model_files_exist_in_dir tests (using temp directories, not real model dirs)

    #[test]
    fn test_check_model_files_exist_in_dir_returns_false_when_directory_missing() {
        let temp_dir =
            std::env::temp_dir().join(format!("heycat-test-{}", uuid::Uuid::new_v4()));
        // Don't create it - test missing dir case
        let manifest = ModelManifest::tdt();
        assert!(!check_model_files_exist_in_dir(&temp_dir, &manifest));
    }

    #[test]
    fn test_check_model_files_exist_in_dir_returns_false_when_files_missing() {
        let temp_dir =
            std::env::temp_dir().join(format!("heycat-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let manifest = ModelManifest::tdt();

        let result = check_model_files_exist_in_dir(&temp_dir, &manifest);

        // Cleanup temp dir
        let _ = std::fs::remove_dir_all(&temp_dir);
        assert!(!result);
    }

    #[test]
    fn test_check_model_files_exist_in_dir_returns_true_with_repo_models() {
        // Use models from the git repo (tracked by Git LFS)
        let repo_model_dir = get_test_models_dir(ModelType::ParakeetTDT);
        let manifest = ModelManifest::tdt();

        assert!(
            check_model_files_exist_in_dir(&repo_model_dir, &manifest),
            "TDT model not found in repo. Run 'git lfs pull' to fetch models. Dir: {:?}",
            repo_model_dir
        );
    }

    #[test]
    fn test_check_model_files_exist_in_dir_returns_true_with_stub_files() {
        use std::io::Write;

        // Create temp dir with stub files for the test
        let temp_dir =
            std::env::temp_dir().join(format!("heycat-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let manifest = ModelManifest::tdt();
        for file in &manifest.files {
            let file_path = temp_dir.join(&file.name);
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(b"stub").unwrap();
        }

        let result = check_model_files_exist_in_dir(&temp_dir, &manifest);

        // Cleanup temp dir (safe - it's a temp dir we created)
        let _ = std::fs::remove_dir_all(&temp_dir);
        assert!(result);
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

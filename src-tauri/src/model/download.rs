// Model download functionality
// Contains the core download logic, testable independently from Tauri commands

use crate::paths;
use crate::worktree::WorktreeContext;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Model type for multi-model support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Parakeet TDT model for batch transcription
    #[serde(rename = "tdt")]
    ParakeetTDT,
}

impl ModelType {
    /// Get the directory name for this model type
    pub fn dir_name(&self) -> &'static str {
        match self {
            ModelType::ParakeetTDT => "parakeet-tdt",
        }
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::ParakeetTDT => write!(f, "tdt"),
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
///
/// Returns:
/// - Main repo: `~/.local/share/heycat/models/`
/// - Worktree: `~/.local/share/heycat-{identifier}/models/`
///
/// For API-compatibility, passing `None` returns the main repo path.
pub fn get_models_dir_with_context(
    worktree_context: Option<&WorktreeContext>,
) -> Result<PathBuf, ModelError> {
    paths::get_models_dir(worktree_context).map_err(|e| match e {
        paths::PathError::DataDirNotFound => ModelError::DataDirNotFound,
        paths::PathError::DirectoryCreationFailed(msg) => ModelError::DirectoryCreationFailed(msg),
    })
}

/// Get the path where models should be stored (API-compatible, uses main repo path)
/// Returns {app_data_dir}/heycat/models/
///
/// Note: Currently only used in tests. Kept for API compatibility with existing code
/// that doesn't have worktree context available.
#[allow(dead_code)]
pub fn get_models_dir() -> Result<PathBuf, ModelError> {
    get_models_dir_with_context(None)
}

/// Get the directory path for a specific model type
///
/// Returns:
/// - Main repo: `~/.local/share/heycat/models/{model_type_dir}/`
/// - Worktree: `~/.local/share/heycat-{identifier}/models/{model_type_dir}/`
pub fn get_model_dir_with_context(
    model_type: ModelType,
    worktree_context: Option<&WorktreeContext>,
) -> Result<PathBuf, ModelError> {
    Ok(get_models_dir_with_context(worktree_context)?.join(model_type.dir_name()))
}

/// Get the directory path for a specific model type (API-compatible, uses main repo path)
/// Returns {app_data_dir}/heycat/models/{model_type_dir}/
pub fn get_model_dir(model_type: ModelType) -> Result<PathBuf, ModelError> {
    get_model_dir_with_context(model_type, None)
}

/// Check if all model files exist in a given directory
fn check_model_files_exist_in_dir(dir: &std::path::Path, manifest: &ModelManifest) -> bool {
    if !dir.exists() {
        return false;
    }
    manifest.files.iter().all(|f| dir.join(&f.name).exists())
}

/// Check if a multi-file model exists (all files present) with worktree context
pub fn check_model_exists_for_type_with_context(
    model_type: ModelType,
    worktree_context: Option<&WorktreeContext>,
) -> Result<bool, ModelError> {
    let model_dir = get_model_dir_with_context(model_type, worktree_context)?;
    let manifest = match model_type {
        ModelType::ParakeetTDT => ModelManifest::tdt(),
    };
    Ok(check_model_files_exist_in_dir(&model_dir, &manifest))
}

/// Check if a multi-file model exists (all files present) (API-compatible, uses main repo path)
pub fn check_model_exists_for_type(model_type: ModelType) -> Result<bool, ModelError> {
    check_model_exists_for_type_with_context(model_type, None)
}

/// Create the models directory if it doesn't exist with worktree context
pub fn ensure_models_dir_with_context(
    worktree_context: Option<&WorktreeContext>,
) -> Result<PathBuf, ModelError> {
    let models_dir = get_models_dir_with_context(worktree_context)?;
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)
            .map_err(|e| ModelError::DirectoryCreationFailed(e.to_string()))?;
    }
    Ok(models_dir)
}

/// Create the models directory if it doesn't exist (API-compatible, uses main repo path)
pub fn ensure_models_dir() -> Result<PathBuf, ModelError> {
    ensure_models_dir_with_context(None)
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
        crate::info!("Model {} already exists at {:?}", model_type_str, final_dir);
        return Ok(final_dir);
    }

    // Create temp directory with unique name
    let models_dir = ensure_models_dir()?;
    let temp_dir_name = format!(".{}-{}", manifest.model_type.dir_name(), Uuid::new_v4());
    let temp_dir = models_dir.join(&temp_dir_name);

    crate::info!(
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

        crate::debug!(
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

        crate::info!(
            "Downloaded {}/{}: {} ({} bytes)",
            file_index + 1,
            total_files,
            model_file.name,
            bytes_written
        );
    }

    // Atomic rename: move temp dir to final location
    crate::debug!(
        "All files downloaded, renaming {:?} to {:?}",
        temp_dir, final_dir
    );

    // Check if another process completed the download
    if final_dir.exists() {
        crate::warn!(
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
            crate::warn!(
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

    crate::info!("Model {} downloaded successfully to {:?}", model_type_str, final_dir);
    Ok(final_dir)
}

#[cfg(test)]
#[path = "download_test.rs"]
mod tests;

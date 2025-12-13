// Model management module
// Handles transcription model download and status checking
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod download;

pub use download::{
    check_model_exists_for_type, download_model_files, get_model_dir, ModelDownloadEventEmitter,
    ModelFile, ModelManifest, ModelType,
};

use tauri::{AppHandle, Emitter};

use crate::events::model_events;

/// Check if a Parakeet model (TDT or EOU) is available
/// model_type: "ParakeetTDT" or "ParakeetEOU"
#[tauri::command]
pub async fn check_parakeet_model_status(model_type: ModelType) -> Result<bool, String> {
    check_model_exists_for_type(model_type).map_err(|e| e.to_string())
}

/// Emitter implementation for Tauri AppHandle
struct TauriEmitter(AppHandle);

impl ModelDownloadEventEmitter for TauriEmitter {
    fn emit_model_file_download_progress(
        &self,
        model_type: &str,
        file_name: &str,
        bytes_downloaded: u64,
        total_bytes: u64,
        file_index: usize,
        total_files: usize,
    ) {
        let percent = if total_bytes > 0 {
            (bytes_downloaded as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };
        let _ = self.0.emit(
            model_events::MODEL_FILE_DOWNLOAD_PROGRESS,
            model_events::ModelFileDownloadProgressPayload {
                model_type: model_type.to_string(),
                file_name: file_name.to_string(),
                bytes_downloaded,
                total_bytes,
                file_index,
                total_files,
                percent,
            },
        );
    }
}

/// Download a Parakeet model (TDT or EOU) from HuggingFace
/// Emits progress events during download and completion event when done
#[tauri::command]
pub async fn download_model(
    app_handle: AppHandle,
    model_type: ModelType,
) -> Result<String, String> {
    let manifest = match model_type {
        ModelType::ParakeetTDT => ModelManifest::tdt(),
        ModelType::ParakeetEOU => ModelManifest::eou(),
    };

    let model_type_str = model_type.to_string();
    let emitter = TauriEmitter(app_handle.clone());

    let path = download_model_files(manifest, &emitter)
        .await
        .map_err(|e| e.to_string())?;

    // Emit completion event
    let _ = app_handle.emit(
        model_events::MODEL_DOWNLOAD_COMPLETED,
        model_events::ModelDownloadCompletedPayload {
            model_type: model_type_str,
            model_path: path.to_string_lossy().to_string(),
        },
    );

    Ok(path.to_string_lossy().to_string())
}

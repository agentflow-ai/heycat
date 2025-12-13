// Model management module
// Handles transcription model download and status checking
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod download;

pub use download::{
    check_model_exists, check_model_exists_for_type, download_model_files, get_model_dir,
    ModelDownloadEventEmitter, ModelFile, ModelManifest, ModelType,
};

use tauri::{AppHandle, Emitter};

use crate::events::model_events;

/// Check if the transcription model is available (legacy Whisper model)
#[tauri::command]
pub async fn check_model_status() -> Result<bool, String> {
    check_model_exists().map_err(|e| e.to_string())
}

/// Check if a Parakeet model (TDT or EOU) is available
/// model_type: "ParakeetTDT" or "ParakeetEOU"
#[tauri::command]
pub async fn check_parakeet_model_status(model_type: ModelType) -> Result<bool, String> {
    check_model_exists_for_type(model_type).map_err(|e| e.to_string())
}

/// Download the transcription model from HuggingFace
/// Emits model_download_completed event when done
#[tauri::command]
pub async fn download_model(app_handle: AppHandle) -> Result<String, String> {
    let path = download::download_model()
        .await
        .map_err(|e| e.to_string())?;

    // Emit completion event
    let _ = app_handle.emit(
        model_events::MODEL_DOWNLOAD_COMPLETED,
        model_events::ModelDownloadCompletedPayload {
            model_path: path.to_string_lossy().to_string(),
        },
    );

    Ok(path.to_string_lossy().to_string())
}

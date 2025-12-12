// Model management module
// Handles whisper model download and status checking
#![cfg_attr(coverage_nightly, coverage(off))]

pub mod download;

pub use download::check_model_exists;

use tauri::{AppHandle, Emitter};

use crate::events::model_events;

/// Check if the whisper model is available
#[tauri::command]
pub async fn check_model_status() -> Result<bool, String> {
    check_model_exists().map_err(|e| e.to_string())
}

/// Download the whisper model from HuggingFace
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

// Parakeet transcription module
// Provides TDT (batch) and EOU (streaming) transcription using NVIDIA Parakeet models

mod manager;
mod streaming;
mod types;

use crate::settings;
use std::sync::Arc;
use tauri::State;

pub use manager::TranscriptionManager;
pub use streaming::StreamingTranscriber;
pub use types::{TranscriptionMode, TranscriptionService};

/// Get the current transcription mode (batch or streaming)
#[tauri::command]
pub fn get_transcription_mode(
    manager: State<'_, Arc<TranscriptionManager>>,
) -> String {
    match manager.current_mode() {
        TranscriptionMode::Batch => "batch".to_string(),
        TranscriptionMode::Streaming => "streaming".to_string(),
    }
}

/// Set the transcription mode (batch or streaming)
/// Also persists the setting to disk for next startup
#[tauri::command]
pub fn set_transcription_mode(
    manager: State<'_, Arc<TranscriptionManager>>,
    mode: TranscriptionMode,
) -> Result<(), String> {
    // Apply the mode to the manager
    manager.set_mode(mode)
        .map_err(|e| e.to_string())?;

    // Persist to settings file
    let app_settings = settings::AppSettings {
        transcription_mode: mode,
    };
    settings::save_settings(&app_settings)?;

    Ok(())
}

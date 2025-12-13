// Parakeet transcription module
// Provides TDT (batch) and EOU (streaming) transcription using NVIDIA Parakeet models

mod manager;
mod streaming;
mod types;

use std::sync::Arc;
use tauri::State;

pub use manager::TranscriptionManager;
pub use streaming::{StreamingState, StreamingTranscriber, CHUNK_SIZE};
pub use types::{TranscriptionError, TranscriptionMode, TranscriptionResult, TranscriptionService, TranscriptionState};

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
#[tauri::command]
pub fn set_transcription_mode(
    manager: State<'_, Arc<TranscriptionManager>>,
    mode: String,
) -> Result<(), String> {
    let transcription_mode = match mode.as_str() {
        "batch" => TranscriptionMode::Batch,
        "streaming" => TranscriptionMode::Streaming,
        _ => return Err(format!("Invalid mode: {}. Expected 'batch' or 'streaming'", mode)),
    };
    manager.set_mode(transcription_mode)
        .map_err(|e| e.to_string())
}

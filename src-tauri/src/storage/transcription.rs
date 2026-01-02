//! Transcription storage operations.
//!
//! Provides a unified interface for storing transcriptions, eliminating
//! duplicated code from transcription/service.rs.

use crate::turso::{events as turso_events, TursoClient};
use crate::util::run_async;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// High-level transcription storage interface.
///
/// Provides methods for storing transcriptions with automatic recording
/// lookup and event emission.
pub struct TranscriptionStorage;

impl TranscriptionStorage {
    /// Store a transcription for a recording.
    ///
    /// This method:
    /// 1. Looks up the recording by file path
    /// 2. Generates a unique transcription ID
    /// 3. Stores the transcription linked to the recording
    /// 4. Emits a transcriptions_updated event on success
    ///
    /// Returns the transcription ID on success.
    pub async fn store(
        client: &TursoClient,
        file_path: &str,
        text: &str,
        duration_ms: u64,
        app_handle: &AppHandle,
    ) -> Result<String, String> {
        // Look up recording by file_path to get recording_id
        let recording_id = match client.get_recording_by_path(file_path).await {
            Ok(Some(recording)) => {
                crate::debug!("Found existing recording in Turso: {}", recording.id);
                recording.id
            }
            Ok(None) => {
                // Recording should exist - both normal and hotkey flows store recordings now
                crate::warn!(
                    "Recording not found in Turso for transcription: {}",
                    file_path
                );
                return Err("Recording not found".to_string());
            }
            Err(e) => {
                crate::debug!("Failed to look up recording in Turso: {}", e);
                return Err(format!("Failed to look up recording: {}", e));
            }
        };

        // Store the transcription
        let transcription_id = uuid::Uuid::new_v4().to_string();
        let model_version = "parakeet-tdt".to_string();

        client
            .add_transcription(
                transcription_id.clone(),
                recording_id.clone(),
                text.to_string(),
                None, // language - could be detected in future
                model_version,
                duration_ms,
            )
            .await
            .map_err(|e| format!("Failed to store transcription: {}", e))?;

        crate::debug!(
            "Transcription stored in Turso for recording {}",
            recording_id
        );

        // Emit transcriptions_updated event
        turso_events::emit_transcriptions_updated(
            app_handle,
            "add",
            Some(&transcription_id),
            Some(&recording_id),
        );

        Ok(transcription_id)
    }
}

/// Store transcription result in Turso (synchronous wrapper).
///
/// This function is called after successful transcription to persist the result.
/// It looks up the recording by file_path and stores the transcription linked to it.
///
/// This is a synchronous wrapper that handles the async-to-sync bridge,
/// suitable for calling from non-async contexts.
///
/// Note: Currently unused since TranscriptionService uses the async version directly,
/// but kept for potential future use in non-async contexts.
#[allow(dead_code)]
pub fn store_transcription(app_handle: &AppHandle, file_path: &str, text: &str, duration_ms: u64) {
    // Get Turso client from managed state
    let turso_client: Option<tauri::State<'_, Arc<TursoClient>>> = app_handle.try_state();

    if let Some(client) = turso_client {
        let client = client.inner().clone();
        let app_handle = app_handle.clone();
        let file_path = file_path.to_string();
        let text = text.to_string();

        // Run the async storage operation synchronously
        run_async(async move {
            if let Err(e) =
                TranscriptionStorage::store(&client, &file_path, &text, duration_ms, &app_handle)
                    .await
            {
                crate::warn!("Failed to store transcription: {}", e);
            }
        });
    }
}

#[cfg(test)]
#[path = "transcription_test.rs"]
mod tests;

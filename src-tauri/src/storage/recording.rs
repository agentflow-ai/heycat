//! Recording storage operations.
//!
//! Provides a unified interface for storing recordings, eliminating
//! duplicated code from hotkey/integration.rs and commands/mod.rs.

use crate::recording::RecordingMetadata;
use crate::turso::{events as turso_events, TursoClient};
use crate::window_context::get_active_window;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Window context information for a recording.
pub struct WindowContext {
    pub app_name: Option<String>,
    pub bundle_id: Option<String>,
    pub title: Option<String>,
}

impl WindowContext {
    /// Capture the current active window context.
    ///
    /// Returns a WindowContext with the app name, bundle ID, and window title
    /// of the currently active window. Returns empty values if the window
    /// information cannot be obtained.
    pub fn capture() -> Self {
        match get_active_window() {
            Ok(info) => Self {
                app_name: Some(info.app_name),
                bundle_id: info.bundle_id,
                title: info.window_title,
            },
            Err(e) => {
                crate::debug!("Could not get active window info: {}", e);
                Self {
                    app_name: None,
                    bundle_id: None,
                    title: None,
                }
            }
        }
    }
}

/// High-level recording storage interface.
///
/// Provides methods for storing recordings with automatic window context
/// capture and event emission.
pub struct RecordingStorage;

impl RecordingStorage {
    /// Store a recording with the given metadata and window context.
    ///
    /// This method:
    /// 1. Generates a unique recording ID
    /// 2. Stores the recording in Turso
    /// 3. Emits a recordings_updated event on success
    ///
    /// Returns the recording ID on success.
    pub async fn store(
        client: &TursoClient,
        metadata: &RecordingMetadata,
        window_context: WindowContext,
        app_handle: &AppHandle,
    ) -> Result<String, String> {
        let recording_id = uuid::Uuid::new_v4().to_string();

        client
            .add_recording(
                recording_id.clone(),
                metadata.file_path.clone(),
                metadata.duration_secs,
                metadata.sample_count as u64,
                metadata.stop_reason.clone(),
                window_context.app_name,
                window_context.bundle_id,
                window_context.title,
            )
            .await
            .map_err(|e| format!("Failed to store recording: {}", e))?;

        crate::debug!("Recording metadata stored in Turso: {}", recording_id);
        turso_events::emit_recordings_updated(app_handle, "add", Some(&recording_id));

        Ok(recording_id)
    }
}

/// Convenience function to store a recording asynchronously.
///
/// This function spawns an async task to store the recording, which is
/// useful for fire-and-forget storage from hotkey handlers.
///
/// # Arguments
/// * `app_handle` - The Tauri app handle
/// * `metadata` - Recording metadata to store
/// * `flow_name` - Name of the flow for logging (e.g., "hotkey", "PTT", "button")
pub fn store_recording(app_handle: &AppHandle, metadata: &RecordingMetadata, flow_name: &str) {
    // Get TursoClient from managed state
    let turso_client: Option<tauri::State<'_, Arc<TursoClient>>> = app_handle.try_state();

    if let Some(client) = turso_client {
        // Capture window context synchronously before spawning
        let window_context = WindowContext::capture();

        let recording_id = uuid::Uuid::new_v4().to_string();
        let file_path = metadata.file_path.clone();
        let duration_secs = metadata.duration_secs;
        let sample_count = metadata.sample_count as u64;
        let stop_reason = metadata.stop_reason.clone();
        let client = client.inner().clone();
        let app_handle_clone = app_handle.clone();
        let flow_name = flow_name.to_string();

        // Spawn async task to store recording
        tauri::async_runtime::spawn(async move {
            if let Err(e) = client
                .add_recording(
                    recording_id.clone(),
                    file_path,
                    duration_secs,
                    sample_count,
                    stop_reason,
                    window_context.app_name,
                    window_context.bundle_id,
                    window_context.title,
                )
                .await
            {
                crate::warn!("Failed to store recording in Turso: {}", e);
            } else {
                crate::debug!("Recording metadata stored in Turso ({} flow)", flow_name);
                turso_events::emit_recordings_updated(&app_handle_clone, "add", Some(&recording_id));
            }
        });
    } else {
        crate::debug!("TursoClient not available in app state");
    }
}

#[cfg(test)]
#[path = "recording_test.rs"]
mod tests;

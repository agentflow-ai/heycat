//! Storage backend traits for recordings and transcriptions.
//!
//! These traits define the interface for storage backends, allowing
//! the storage layer to be decoupled from the specific storage implementation.

use crate::audio::StopReason;
use crate::turso::{RecordingRecord, RecordingStoreError, TranscriptionRecord, TranscriptionStoreError};
use async_trait::async_trait;

/// Backend trait for recording storage operations.
///
/// Implementations of this trait provide the actual storage operations
/// for recordings. The primary implementation is TursoClient.
#[async_trait]
pub trait RecordingStoreBackend: Send + Sync {
    /// Add a new recording to storage.
    async fn add_recording(
        &self,
        id: String,
        file_path: String,
        duration_secs: f64,
        sample_count: u64,
        stop_reason: Option<StopReason>,
        active_window_app_name: Option<String>,
        active_window_bundle_id: Option<String>,
        active_window_title: Option<String>,
    ) -> Result<RecordingRecord, RecordingStoreError>;

    /// Get a recording by its file path.
    async fn get_recording_by_path(
        &self,
        file_path: &str,
    ) -> Result<Option<RecordingRecord>, RecordingStoreError>;
}

/// Backend trait for transcription storage operations.
///
/// Implementations of this trait provide the actual storage operations
/// for transcriptions. The primary implementation is TursoClient.
#[async_trait]
pub trait TranscriptionStoreBackend: Send + Sync {
    /// Add a new transcription to storage.
    async fn add_transcription(
        &self,
        id: String,
        recording_id: String,
        text: String,
        language: Option<String>,
        model_version: String,
        duration_ms: u64,
    ) -> Result<TranscriptionRecord, TranscriptionStoreError>;
}

#[cfg(test)]
#[path = "traits_test.rs"]
mod tests;

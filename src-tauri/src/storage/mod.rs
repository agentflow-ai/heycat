//! Storage abstraction layer for recordings and transcriptions.
//!
//! This module provides a unified interface for storing and retrieving
//! recordings and transcriptions, eliminating duplicated storage code
//! across the codebase.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::storage::{RecordingStorage, TranscriptionStorage};
//!
//! // Store a recording
//! let recording_id = storage::store_recording(
//!     &app_handle,
//!     &metadata,
//!     window_context,
//! ).await?;
//!
//! // Store a transcription
//! storage::store_transcription(
//!     &app_handle,
//!     &file_path,
//!     &text,
//!     duration_ms,
//! ).await?;
//! ```

mod recording;
mod traits;
mod transcription;

pub use recording::{store_recording, RecordingStorage, WindowContext};
pub use transcription::store_transcription;

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;

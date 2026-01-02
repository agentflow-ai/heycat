//! Storage abstraction layer for recordings and transcriptions.
//!
//! This module provides a unified interface for storing and retrieving
//! recordings and transcriptions, eliminating duplicated storage code
//! across the codebase.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::storage::{store_recording, store_transcription};
//!
//! // Store a recording (synchronous - handles async internally)
//! store_recording(&app_handle, &metadata, "hotkey");
//!
//! // Store a transcription (synchronous - handles async internally)
//! store_transcription(&app_handle, &file_path, &text, duration_ms);
//! ```

mod recording;
mod transcription;

pub use recording::{store_recording, RecordingStorage, WindowContext};
pub use transcription::{store_transcription, TranscriptionStorage};

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;

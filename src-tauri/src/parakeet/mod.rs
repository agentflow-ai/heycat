// Parakeet transcription module
// Provides TDT (batch) and EOU (streaming) transcription using NVIDIA Parakeet models

mod manager;
mod streaming;

pub use manager::TranscriptionManager;
pub use streaming::StreamingTranscriber;

// Re-export shared types from whisper module (will be moved in cleanup spec)
pub use crate::whisper::{TranscriptionError, TranscriptionResult, TranscriptionService, TranscriptionState};

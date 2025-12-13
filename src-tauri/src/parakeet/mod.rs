// Parakeet transcription module
// Provides TDT (batch) and EOU (streaming) transcription using NVIDIA Parakeet models

mod manager;
mod streaming;
mod types;

pub use manager::TranscriptionManager;
pub use streaming::StreamingTranscriber;
pub use types::{TranscriptionError, TranscriptionResult, TranscriptionService, TranscriptionState};

// Whisper transcription module
// Provides WhisperManager for model loading and audio transcription

mod context;

pub use context::{
    TranscriptionError, TranscriptionResult, TranscriptionService, TranscriptionState,
    WhisperManager,
};

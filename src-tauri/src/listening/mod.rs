// Listening module for always-on wake word detection
// Provides WakeWordDetector using Parakeet for on-device wake phrase recognition
// and ListeningManager for coordinating listening state with recording

mod buffer;
mod cancel;
mod detector;
mod manager;
mod pipeline;
mod silence;

pub use buffer::CircularBuffer;
pub use cancel::{
    CancelPhraseDetector, CancelPhraseDetectorConfig, CancelPhraseError, CancelPhraseResult,
};
pub use detector::{WakeWordDetector, WakeWordDetectorConfig, WakeWordError, WakeWordResult};
pub use manager::{ListeningError, ListeningManager, ListeningStatus};
pub use pipeline::{ListeningPipeline, PipelineConfig, PipelineError};
pub use silence::{SilenceConfig, SilenceDetectionResult, SilenceDetector, SilenceStopReason};

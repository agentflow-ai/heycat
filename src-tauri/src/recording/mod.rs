// Recording module for managing recording state

mod coordinator;
mod silence;
mod state;
mod vad;

pub use coordinator::RecordingDetectors;
pub use silence::{SilenceConfig, SilenceDetectionResult, SilenceDetector, SilenceStopReason};
pub use state::{AudioData, RecordingManager, RecordingMetadata, RecordingState};
pub use vad::{create_vad, VadConfig, VadError};

#[cfg(test)]
pub use state::RecordingStateError;

#[cfg(test)]
mod state_test;

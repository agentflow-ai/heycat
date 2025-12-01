// Recording module for managing recording state

mod state;
pub use state::{AudioData, RecordingManager, RecordingMetadata, RecordingState, RecordingStateError};

#[cfg(test)]
mod state_test;

// Recording module for managing recording state

mod state;
pub use state::{AudioData, RecordingManager, RecordingMetadata, RecordingState};

#[cfg(test)]
pub use state::RecordingStateError;

#[cfg(test)]
mod state_test;

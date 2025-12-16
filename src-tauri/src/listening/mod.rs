// Listening module for always-on wake word detection
// Provides WakeWordDetector using Parakeet for on-device wake phrase recognition
// and ListeningManager for coordinating listening state with recording

mod buffer;
mod coordinator;
mod detector;
mod events;
mod manager;
mod pipeline;
mod silence;
mod vad;

// Core components used by the listening pipeline
pub use buffer::CircularBuffer;
pub use detector::{WakeWordDetector, WakeWordDetectorConfig, WakeWordError};
pub use manager::{ListeningError, ListeningManager, ListeningStatus};
pub use pipeline::ListeningPipeline;

// Recording phase detectors - used by both wake word and hotkey recording flows
pub use coordinator::RecordingDetectors;
#[allow(unused_imports)]
pub use detector::WakeWordResult;
#[allow(unused_imports)]
pub use pipeline::{PipelineConfig, PipelineError};
#[allow(unused_imports, deprecated)]
pub use pipeline::WakeWordCallback;
#[allow(unused_imports)]
pub use silence::{SilenceConfig, SilenceDetectionResult, SilenceDetector, SilenceStopReason};

// Event channel types for safe async communication
pub use events::WakeWordEvent;

// Unified VAD configuration - exported for future external consumers
#[allow(unused_imports)]
pub use vad::{create_vad, VadConfig, VadError};

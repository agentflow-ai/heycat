// Listening module for always-on wake word detection
// Provides WakeWordDetector using Parakeet for on-device wake phrase recognition
// and ListeningManager for coordinating listening state with recording

mod buffer;
mod coordinator;
mod detector;
mod manager;
mod pipeline;
mod silence;

// Core components used by the listening pipeline
pub use buffer::CircularBuffer;
pub use detector::{WakeWordDetector, WakeWordDetectorConfig, WakeWordError};
pub use manager::{ListeningError, ListeningManager, ListeningStatus};
pub use pipeline::ListeningPipeline;

// Recording phase detectors - implemented but integration pending
// These will be wired when RecordingDetectors is integrated into HotkeyIntegration
#[allow(unused_imports)]
pub use coordinator::RecordingDetectors;
#[allow(unused_imports)]
pub use detector::WakeWordResult;
#[allow(unused_imports)]
pub use pipeline::{PipelineConfig, PipelineError, WakeWordCallback};
#[allow(unused_imports)]
pub use silence::{SilenceConfig, SilenceDetectionResult, SilenceDetector, SilenceStopReason};

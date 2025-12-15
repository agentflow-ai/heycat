// Parakeet transcription module
// Provides TDT (batch) transcription using NVIDIA Parakeet models

mod manager;
mod shared;
mod types;
mod utils;

pub use manager::TranscriptionManager;
pub use shared::SharedTranscriptionModel;
// TranscribingGuard exported for public API (RAII state management)
#[allow(unused_imports)]
pub use shared::TranscribingGuard;
pub use types::TranscriptionService;

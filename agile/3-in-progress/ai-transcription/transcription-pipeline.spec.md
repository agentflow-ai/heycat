---
status: pending
created: 2025-12-12
completed: null
dependencies:
  - model-download
---

# Spec: Core Transcription Pipeline

## Description

Implement the core Whisper transcription pipeline using whisper-rs. This includes the WhisperContext wrapper with thread-safety, TranscriptionManager state machine, and the transcribe function that converts audio samples to text.

## Acceptance Criteria

- [ ] WhisperContext wrapper loads model and provides transcription interface
- [ ] Mutex wrapping ensures thread-safe access to whisper.cpp (not thread-safe by default)
- [ ] TranscriptionManager with state machine: Idle -> Transcribing -> Completed/Error
- [ ] `TranscriptionService` trait defined for mockability in tests
- [ ] `transcribe()` function accepts &[f32] audio samples (16kHz mono) and returns Result<String>
- [ ] Proper error handling: ModelNotLoaded, TranscriptionFailed, InvalidAudio
- [ ] Model loaded eagerly at app startup (per technical guidance decision)

## Test Cases

- [ ] transcribe() returns error when model not loaded
- [ ] transcribe() succeeds with valid audio samples
- [ ] transcribe() handles empty audio gracefully
- [ ] State transitions correctly: Idle -> Transcribing -> Completed
- [ ] State transitions on error: Idle -> Transcribing -> Error -> Idle
- [ ] Concurrent transcription requests are serialized (Mutex)

## Dependencies

- model-download (model must be available to load)

## Preconditions

- whisper-rs crate added to Cargo.toml
- Model file exists at expected path

## Implementation Notes

- Add to Cargo.toml: `whisper-rs = "0.13"`
- Follow VoiceInk pattern: load model once, reuse context
- Use `WhisperContext::new_with_params()` for model loading
- Use `ctx.full()` for transcription with default parameters
- Consider language auto-detection (per feature.md out-of-scope, use default)

```rust
pub struct WhisperManager {
    context: Arc<Mutex<Option<WhisperContext>>>,
}

impl WhisperManager {
    pub fn load_model(&self, path: &Path) -> Result<(), Error>;
    pub fn transcribe(&self, samples: &[f32]) -> Result<String, Error>;
    pub fn is_loaded(&self) -> bool;
}
```

## Related Specs

- model-download.spec.md (provides the model file)
- auto-transcribe-on-stop.spec.md (calls transcribe())
- sample-rate-modification.spec.md (provides correct sample format)

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (state initialization)
- Connects to: RecordingManager, TranscriptionCommands

## Integration Test

- Test location: `src-tauri/src/whisper/context_test.rs`
- Verification: [ ] Integration test passes

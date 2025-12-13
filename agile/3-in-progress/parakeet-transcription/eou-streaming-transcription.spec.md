---
status: pending
created: 2025-12-13
completed: null
dependencies: ["parakeet-module-skeleton.spec.md", "streaming-audio-integration.spec.md"]
---

# Spec: Implement EOU streaming transcription

## Description

Implement a `StreamingTranscriber` struct that wraps `parakeet_rs::ParakeetEOU` for real-time streaming transcription during recording. The transcriber receives audio chunks via an MPSC channel from the audio callback, processes them through EOU in 160ms chunks (2560 samples at 16kHz), and emits `transcription_partial` events to the frontend. When recording stops, it processes the final chunk with `is_final=true` and emits a `transcription_completed` event.

This enables users to see their speech transcribed in real-time while they are still recording, rather than waiting until after recording stops.

## Acceptance Criteria

- [ ] `StreamingTranscriber` struct created in `src-tauri/src/parakeet/streaming.rs`
- [ ] Receives audio chunks via `tokio::sync::mpsc::Receiver<Vec<f32>>`
- [ ] Buffers incoming samples until 2560 samples (160ms at 16kHz) are accumulated
- [ ] Calls `parakeet.transcribe(chunk, false)` for intermediate chunks
- [ ] Calls `parakeet.transcribe(final_chunk, true)` when signaled to stop
- [ ] Emits `transcription_partial` events with accumulated partial text during recording
- [ ] Emits `transcription_completed` event with full text when finalized
- [ ] Handles empty/silent audio gracefully (no crash, empty partial events OK)
- [ ] Thread-safe: transcriber runs in dedicated async task

## Test Cases

- [ ] `test_streaming_transcriber_new_unloaded` - StreamingTranscriber starts in unloaded state when no model path provided
- [ ] `test_streaming_transcriber_load_model` - Load EOU model from valid directory path succeeds
- [ ] `test_streaming_transcriber_load_model_invalid_path` - Load from nonexistent path returns error
- [ ] `test_streaming_transcriber_process_chunk_emits_partial` - Processing a 160ms chunk emits partial event (mock)
- [ ] `test_streaming_transcriber_finalize_emits_completed` - Finalizing with is_final=true emits completed event (mock)
- [ ] `test_streaming_transcriber_buffers_small_chunks` - Chunks smaller than 2560 samples are buffered until complete

## Dependencies

- `parakeet-module-skeleton.spec.md` - Module structure and shared types
- `streaming-audio-integration.spec.md` - Audio callback sending chunks via channel

## Preconditions

- `parakeet-rs` crate added to `Cargo.toml`
- EOU model files downloaded to `{app_data_dir}/heycat/models/parakeet-eou/`
- `transcription_partial` event added to `events.rs`

## Implementation Notes

### Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/parakeet/streaming.rs` | Create | StreamingTranscriber implementation |
| `src-tauri/src/parakeet/mod.rs` | Modify | Export streaming module |
| `src-tauri/src/events.rs` | Modify | Add `transcription_partial` event and payload |

### Struct Design

```rust
use parakeet_rs::ParakeetEOU;
use tokio::sync::mpsc;
use std::sync::Arc;

/// Streaming transcription state
pub enum StreamingState {
    Unloaded,
    Idle,
    Streaming,
    Finalizing,
}

/// Receives audio chunks and emits partial transcription events
pub struct StreamingTranscriber<E: TranscriptionEventEmitter> {
    /// EOU model instance (None if not loaded)
    eou: Option<ParakeetEOU>,
    /// Current state
    state: StreamingState,
    /// Buffer for accumulating samples before processing
    sample_buffer: Vec<f32>,
    /// Accumulated partial text from all chunks
    partial_text: String,
    /// Event emitter for partial/completed events
    emitter: Arc<E>,
}

impl<E: TranscriptionEventEmitter> StreamingTranscriber<E> {
    const CHUNK_SIZE: usize = 2560; // 160ms at 16kHz

    pub fn new(emitter: Arc<E>) -> Self;
    pub fn load_model(&mut self, model_dir: &Path) -> Result<(), TranscriptionError>;
    pub fn is_loaded(&self) -> bool;

    /// Process incoming samples - buffers until CHUNK_SIZE reached
    pub fn process_samples(&mut self, samples: &[f32]) -> Result<(), TranscriptionError>;

    /// Finalize transcription with is_final=true
    pub fn finalize(&mut self) -> Result<String, TranscriptionError>;

    /// Reset for next recording
    pub fn reset(&mut self);
}
```

### Event Flow

```
Recording starts
    |
    v
Audio callback sends chunks via channel
    |
    v
StreamingTranscriber.process_samples()
    |
    +---> Buffer until 2560 samples
    |
    v
parakeet.transcribe(chunk, false)
    |
    v
Emit transcription_partial { text: accumulated_text }
    |
    v
(repeat for each 160ms of audio)
    |
    v
Recording stops
    |
    v
StreamingTranscriber.finalize()
    |
    v
parakeet.transcribe(remaining, true)
    |
    v
Emit transcription_completed { text, duration_ms }
```

### New Event Payload

```rust
// In events.rs
pub const TRANSCRIPTION_PARTIAL: &str = "transcription_partial";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TranscriptionPartialPayload {
    /// Accumulated partial transcription text so far
    pub text: String,
    /// Whether this is the final update before completed event
    pub is_final: bool,
}
```

## Related Specs

- `parakeet-module-skeleton.spec.md` - Module setup
- `streaming-audio-integration.spec.md` - Audio channel integration
- `tdt-batch-transcription.spec.md` - Batch mode alternative
- `wire-up-transcription.spec.md` - Integration with HotkeyIntegration

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` - spawn streaming transcription task
- Connects to: `audio/cpal_backend.rs` (receives chunks), `events.rs` (emits events)

## Integration Test

- Test location: `src-tauri/src/parakeet/streaming_test.rs`
- Verification: [ ] Integration test passes
- Test approach: Mock event emitter, feed sample data through StreamingTranscriber, verify partial and completed events are emitted with expected text

---
status: pending
created: 2025-12-13
completed: null
dependencies: ["parakeet-module-skeleton.spec.md"]
---

# Spec: Streaming audio pipeline integration

## Description

Add a channel-based audio streaming mechanism from the cpal audio callback to the `StreamingTranscriber`. The existing cpal backend accumulates samples into a buffer; this spec adds an optional mpsc channel that sends 160ms audio chunks (2560 samples at 16kHz) in real-time for streaming transcription. The channel is only active when streaming mode is enabled.

## Acceptance Criteria

- [ ] `StreamingAudioSender` type alias created: `mpsc::SyncSender<Vec<f32>>`
- [ ] `CallbackState` extended with optional `streaming_sender: Option<StreamingAudioSender>`
- [ ] `AudioCaptureBackend::start()` signature updated to accept optional streaming sender
- [ ] Callback accumulates samples and sends 2560-sample chunks when sender is present
- [ ] Chunks sent without blocking (use `try_send` with bounded channel)
- [ ] `STREAMING_CHUNK_SIZE` constant defined: `2560` (160ms at 16kHz)
- [ ] Channel overflow logs warning but does not stop recording

## Test Cases

- [ ] Unit test: `STREAMING_CHUNK_SIZE` equals 2560
- [ ] Unit test: `160ms * 16000Hz = 2560` (verify calculation)
- [ ] Unit test: `CallbackState::new()` with `streaming_sender: None` works (backward compatible)
- [ ] Unit test: Samples accumulate until chunk size reached before sending
- [ ] Unit test: Partial chunks (< 2560 samples) are not sent prematurely

## Dependencies

- `parakeet-module-skeleton.spec.md` - `StreamingTranscriber` struct must exist

## Preconditions

- cpal backend functional (`src-tauri/src/audio/cpal_backend.rs`)
- Audio capture working at 16kHz

## Implementation Notes

### Constants

Add to `audio/mod.rs`:
```rust
/// Chunk size for streaming transcription (160ms at 16kHz)
/// EOU model processes audio in 160ms chunks for real-time transcription
pub const STREAMING_CHUNK_SIZE: usize = 2560; // 160ms * 16kHz
```

### Type Aliases

Add to `audio/mod.rs`:
```rust
/// Sender for streaming audio chunks to transcriber
pub type StreamingAudioSender = std::sync::mpsc::SyncSender<Vec<f32>>;
/// Receiver for streaming audio chunks
pub type StreamingAudioReceiver = std::sync::mpsc::Receiver<Vec<f32>>;
```

### CallbackState Modifications

Update `cpal_backend.rs`:
```rust
struct CallbackState {
    buffer: AudioBuffer,
    stop_signal: Option<Sender<StopReason>>,
    signaled: Arc<AtomicBool>,
    resampler: Option<Arc<Mutex<FftFixedIn<f32>>>>,
    resample_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_buffer: Arc<Mutex<Vec<f32>>>,
    chunk_size: usize,
    // New: Streaming support
    streaming_sender: Option<StreamingAudioSender>,
    streaming_accumulator: Arc<Mutex<Vec<f32>>>,
}
```

### Streaming Logic in process_samples

After adding samples to buffer, if streaming is enabled:
```rust
fn process_samples(&self, f32_samples: &[f32]) {
    // ... existing buffer logic ...

    // Streaming: accumulate and send 160ms chunks
    if let Some(ref sender) = self.streaming_sender {
        if let Ok(mut acc) = self.streaming_accumulator.lock() {
            acc.extend_from_slice(&samples_to_add);

            while acc.len() >= STREAMING_CHUNK_SIZE {
                let chunk: Vec<f32> = acc.drain(..STREAMING_CHUNK_SIZE).collect();
                // Non-blocking send - log warning on overflow, don't stop recording
                if let Err(e) = sender.try_send(chunk) {
                    warn!("Streaming channel full, dropping chunk: {}", e);
                }
            }
        }
    }
}
```

### AudioCaptureBackend Signature Update

```rust
pub trait AudioCaptureBackend {
    fn start(
        &mut self,
        buffer: AudioBuffer,
        stop_signal: Option<std::sync::mpsc::Sender<StopReason>>,
        streaming_sender: Option<StreamingAudioSender>, // NEW
    ) -> Result<u32, AudioCaptureError>;

    fn stop(&mut self) -> Result<(), AudioCaptureError>;
}
```

### Channel Configuration

Use bounded sync channel to prevent memory growth:
```rust
// In calling code (hotkey integration or recording manager)
let (sender, receiver) = std::sync::mpsc::sync_channel::<Vec<f32>>(10); // ~1.6 seconds buffer
```

### Files to Modify

- `src-tauri/src/audio/mod.rs` - Add constants and type aliases
- `src-tauri/src/audio/cpal_backend.rs` - Add streaming sender to CallbackState
- `src-tauri/src/audio/thread.rs` - Update thread handle to pass streaming sender

### Pattern Reference

Follow existing `stop_signal` pattern in `CallbackState` - optional sender that doesn't block the callback.

## Related Specs

- `parakeet-module-skeleton.spec.md` - Prerequisite
- `eou-streaming-transcription.spec.md` - Will consume chunks from this channel
- `wire-up-transcription.spec.md` - Will connect channel to StreamingTranscriber

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:168` (start method)
- Connects to: `audio/thread.rs`, `parakeet/streaming.rs`

## Integration Test

- Test location: `src-tauri/src/audio/mod_test.rs` (add streaming tests)
- Verification: [ ] Integration test passes

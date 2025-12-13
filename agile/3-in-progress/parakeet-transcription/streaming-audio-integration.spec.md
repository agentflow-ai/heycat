---
status: completed
created: 2025-12-13
completed: 2025-12-13
dependencies: ["parakeet-module-skeleton.spec.md"]
review_round: 1
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

## Review

**Reviewed:** 2025-12-13
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `StreamingAudioSender` type alias created: `mpsc::SyncSender<Vec<f32>>` | PASS | src-tauri/src/audio/mod.rs:57 |
| `CallbackState` extended with optional `streaming_sender: Option<StreamingAudioSender>` | PASS | src-tauri/src/audio/cpal_backend.rs:84 |
| `AudioCaptureBackend::start()` signature updated to accept optional streaming sender | PASS | src-tauri/src/audio/mod.rs:134-139 (trait) and cpal_backend.rs:186-191 (impl) |
| Callback accumulates samples and sends 2560-sample chunks when sender is present | PASS | src-tauri/src/audio/cpal_backend.rs:168-181 - accumulator logic with `while acc.len() >= STREAMING_CHUNK_SIZE` |
| Chunks sent without blocking (use `try_send` with bounded channel) | PASS | src-tauri/src/audio/cpal_backend.rs:176 - `sender.try_send(chunk)` |
| `STREAMING_CHUNK_SIZE` constant defined: `2560` (160ms at 16kHz) | PASS | src-tauri/src/audio/mod.rs:54 |
| Channel overflow logs warning but does not stop recording | PASS | src-tauri/src/audio/cpal_backend.rs:177 - uses `warn!` macro and continues execution |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Unit test: `STREAMING_CHUNK_SIZE` equals 2560 | PASS | src-tauri/src/audio/mod_test.rs:101-104 (`test_streaming_chunk_size_value`) |
| Unit test: `160ms * 16000Hz = 2560` (verify calculation) | PASS | src-tauri/src/audio/mod_test.rs:106-113 (`test_streaming_chunk_size_calculation`) |
| Unit test: `CallbackState::new()` with `streaming_sender: None` works (backward compatible) | PASS | src-tauri/src/audio/thread.rs:276 - `test_start_stop_commands` calls `handle.start(buffer, None)` |
| Unit test: Samples accumulate until chunk size reached before sending | PASS | src-tauri/src/audio/mod_test.rs:141-166 (`test_streaming_accumulator_sends_when_chunk_size_reached`) |
| Unit test: Partial chunks (< 2560 samples) are not sent prematurely | PASS | src-tauri/src/audio/mod_test.rs:117-138 (`test_streaming_accumulator_partial_chunk_not_sent`) |

### Code Quality

**Strengths:**
- Clean separation of concerns: streaming logic is isolated in `process_samples()` method
- Follows existing patterns: streaming sender pattern mirrors the established `stop_signal` pattern
- Non-blocking design: `try_send` prevents audio callback from blocking on slow consumers
- Thread safety: uses `Arc<Mutex<Vec<f32>>>` for streaming accumulator, consistent with other shared state
- Comprehensive test coverage: includes edge cases like multiple chunks and partial chunks
- Good documentation: type aliases and constants include doc comments explaining purpose

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria are met with proper implementation. The streaming audio integration follows established patterns in the codebase, uses non-blocking channel operations to prevent audio dropouts, and has comprehensive unit tests covering the key behaviors including edge cases.

---
status: pending
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Extract Partial VAD Chunk Constant

## Description

The wake word detector VAD processing has a magic number `256` representing the minimum samples to process for a partial chunk. Extract this to a named constant in `audio_constants.rs` for clarity and consistency with other audio constants.

## Acceptance Criteria

- [ ] New constant `MIN_PARTIAL_VAD_CHUNK` defined in `audio_constants.rs`
- [ ] Constant has documentation explaining its purpose
- [ ] Magic number `256` replaced with constant in `detector.rs`
- [ ] Value equals `VAD_CHUNK_SIZE_16KHZ / 2` (512 / 2 = 256)
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Test Cases

- [ ] Constant value test in `audio_constants.rs` tests
- [ ] Wake word detector tests continue to pass

## Dependencies

None

## Preconditions

None

## Implementation Notes

**File 1: `src-tauri/src/audio_constants.rs`**

Add after `VAD_CHUNK_SIZE_8KHZ`:
```rust
/// Minimum samples to process for a partial VAD chunk.
///
/// When the remaining audio buffer doesn't fill a complete VAD chunk,
/// we still process it if it contains at least this many samples.
/// Set to half a chunk (256 samples at 16kHz = 16ms) to avoid
/// missing speech at buffer boundaries while filtering noise.
pub const MIN_PARTIAL_VAD_CHUNK: usize = VAD_CHUNK_SIZE_16KHZ / 2;
```

**File 2: `src-tauri/src/listening/detector.rs:552-553`**

Change from:
```rust
let remaining = samples.len() % CHUNK_SIZE;
if remaining >= 256 {  // Magic number
```

To:
```rust
use crate::audio_constants::MIN_PARTIAL_VAD_CHUNK;

let remaining = samples.len() % CHUNK_SIZE;
if remaining >= MIN_PARTIAL_VAD_CHUNK {
```

## Related Specs

None

## Integration Points

- Production call site: `WakeWordDetector::check_vad_internal()` at `detector.rs:509-578`
- Connects to: audio_constants module

## Integration Test

- Test location: N/A (constant extraction, no functional change)
- Verification: [x] N/A

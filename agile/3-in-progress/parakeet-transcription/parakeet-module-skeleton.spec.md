---
status: in-progress
created: 2025-12-13
completed: null
dependencies: []
---

# Spec: Create Parakeet module skeleton

## Description

Create the foundational Parakeet module structure in the Rust backend without actual transcription implementation. This establishes the module hierarchy, public exports, and type definitions that subsequent specs will build upon. The skeleton follows the same organizational pattern as the existing `whisper/` module while introducing the new types needed for multi-model support (TDT and EOU).

## Acceptance Criteria

- [ ] New `parakeet/` directory created under `src-tauri/src/`
- [ ] `parakeet/mod.rs` created with submodule declarations and public re-exports
- [ ] `parakeet/manager.rs` created with `TranscriptionManager` struct skeleton (empty impl blocks)
- [ ] `parakeet/streaming.rs` created with `StreamingTranscriber` struct skeleton (empty impl blocks)
- [ ] `parakeet-rs = "0.2"` dependency added to `Cargo.toml`
- [ ] `TranscriptionService` trait re-exported (existing trait from `whisper/context.rs`)
- [ ] Module compiles without errors (`cargo check` passes)

## Test Cases

- [ ] Unit test: `TranscriptionManager::new()` returns instance with `Unloaded` state
- [ ] Unit test: `TranscriptionManager::state()` returns current `TranscriptionState`
- [ ] Unit test: `StreamingTranscriber::new()` returns instance (no model loaded)
- [ ] Integration test: Module re-exports are accessible from `lib.rs` scope

## Dependencies

None - this is a foundational spec with no dependencies.

## Preconditions

- Rust toolchain and Cargo available
- Project compiles successfully before starting

## Implementation Notes

### Files to Create

1. **`src-tauri/src/parakeet/mod.rs`**
   ```rust
   // Parakeet transcription module
   // Provides TDT (batch) and EOU (streaming) transcription

   mod manager;
   mod streaming;

   pub use manager::TranscriptionManager;
   pub use streaming::StreamingTranscriber;

   // Re-export shared types from whisper module (will be moved in cleanup spec)
   pub use crate::whisper::{TranscriptionError, TranscriptionResult, TranscriptionService, TranscriptionState};
   ```

2. **`src-tauri/src/parakeet/manager.rs`**
   - `TranscriptionManager` struct with:
     - `tdt_context: Arc<Mutex<Option<ParakeetTDT>>>`
     - `eou_context: Arc<Mutex<Option<ParakeetEOU>>>`
     - `state: Arc<Mutex<TranscriptionState>>`
   - Stub implementations for `TranscriptionService` trait methods (return `unimplemented!()` or placeholder errors)
   - `new()` constructor returning `Unloaded` state

3. **`src-tauri/src/parakeet/streaming.rs`**
   - `StreamingTranscriber` struct with:
     - `audio_receiver: Option<Receiver<Vec<f32>>>`
     - `chunk_buffer: Vec<f32>`
   - `new()` constructor
   - `process_chunk()` stub method

### Cargo.toml Changes

Add to `[dependencies]`:
```toml
parakeet-rs = "0.2"
```

### Pattern Reference

Follow the structure of `src-tauri/src/whisper/mod.rs` and `src-tauri/src/whisper/context.rs` for module organization and trait patterns.

## Related Specs

- `multi-file-model-download.spec.md` - Depends on this skeleton existing
- `streaming-audio-integration.spec.md` - Depends on this skeleton existing
- `tdt-batch-transcription.spec.md` - Will implement `TranscriptionService` trait
- `eou-streaming-transcription.spec.md` - Will implement streaming logic

## Integration Points

- Production call site: N/A (standalone module - wired up in later specs)
- Connects to: `whisper/context.rs` (re-exports types that will be moved in cleanup)

## Integration Test

- Test location: N/A (unit-only spec)
- Verification: [x] N/A

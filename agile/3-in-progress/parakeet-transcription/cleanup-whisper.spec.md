---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - parakeet-module-skeleton.spec.md
  - multi-file-model-download.spec.md
  - tdt-batch-transcription.spec.md
  - eou-streaming-transcription.spec.md
  - streaming-audio-integration.spec.md
  - frontend-model-settings.spec.md
  - wire-up-transcription.spec.md
---

# Spec: Remove Whisper dependency

## Description

Remove the whisper-rs dependency and delete the entire whisper module after Parakeet transcription is fully functional. This is a cleanup spec that must be executed LAST after all other Parakeet specs are complete and verified working. The goal is to eliminate dead code and reduce binary size by removing the unused Whisper dependency.

## Acceptance Criteria

- [ ] `whisper-rs = "0.13"` removed from `src-tauri/Cargo.toml`
- [ ] `src-tauri/src/whisper/` directory deleted entirely (mod.rs and context.rs)
- [ ] `mod whisper;` declaration removed from `src-tauri/src/lib.rs`
- [ ] All references to `whisper::` module in `lib.rs` removed
- [ ] All references to `WhisperManager` in `hotkey/integration.rs` removed
- [ ] Application compiles without errors after removal
- [ ] Application starts and Parakeet transcription works correctly
- [ ] No references to "whisper" remain in Rust source code (except comments/docs)

## Test Cases

- [ ] `cargo build` succeeds after whisper removal
- [ ] `cargo test` passes (no tests reference whisper directly)
- [ ] Application launches without errors
- [ ] Recording and transcription workflow completes successfully with Parakeet
- [ ] No "whisper" strings appear in `cargo tree` output
- [ ] Binary size is reduced (whisper-rs brings in whisper.cpp C++ dependencies)

## Dependencies

ALL other Parakeet specs must be completed first:
1. `parakeet-module-skeleton.spec.md` - TranscriptionManager must exist
2. `multi-file-model-download.spec.md` - Model download must work
3. `tdt-batch-transcription.spec.md` - Batch transcription must work
4. `eou-streaming-transcription.spec.md` - Streaming transcription must work
5. `streaming-audio-integration.spec.md` - Audio integration must work
6. `frontend-model-settings.spec.md` - UI must work
7. `wire-up-transcription.spec.md` - Integration must be complete

## Preconditions

- Parakeet transcription is fully functional for both batch and streaming modes
- All Parakeet-related tests pass
- Manual verification that transcription works end-to-end

## Implementation Notes

### Files to Delete

1. `src-tauri/src/whisper/mod.rs`
2. `src-tauri/src/whisper/context.rs`

### Files to Modify

#### src-tauri/Cargo.toml
Remove line:
```toml
whisper-rs = "0.13"
```

#### src-tauri/src/lib.rs
Remove:
```rust
mod whisper;
```

Remove all code referencing:
- `whisper::WhisperManager`
- `whisper::TranscriptionService`
- `whisper_manager` variable

The `with_whisper_manager()` builder call should already be replaced with `with_transcription_manager()` by the wire-up spec.

#### src-tauri/src/hotkey/integration.rs
By this point, all `WhisperManager` references should already be replaced with `TranscriptionManager` by the wire-up spec. Verify no whisper imports remain:
```rust
// REMOVE if still present:
use crate::whisper::{TranscriptionService, WhisperManager};
```

### Verification Commands

```bash
# Check no whisper references remain
grep -r "whisper" src-tauri/src/ --include="*.rs" | grep -v "// " | grep -v "parakeet"

# Verify build
cd src-tauri && cargo build

# Verify tests
cd src-tauri && cargo test

# Check dependency tree
cd src-tauri && cargo tree | grep -i whisper
```

## Related Specs

- All other specs in `agile/2-todo/parakeet-transcription/`

## Integration Points

- Production call site: N/A (removal spec)
- Connects to: N/A (this removes unused code)

## Integration Test

- Test location: Existing tests should pass without modification
- Verification: [ ] All cargo tests pass after removal

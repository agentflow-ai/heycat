---
status: in-progress
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

## Review

**Reviewed:** 2025-12-13
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `whisper-rs = "0.13"` removed from `src-tauri/Cargo.toml` | PASS | Cargo.toml:37 shows `parakeet-rs = "0.2"`, no whisper-rs dependency present |
| `src-tauri/src/whisper/` directory deleted entirely | PASS | `ls` confirms "No such file or directory" |
| `mod whisper;` declaration removed from `src-tauri/src/lib.rs` | PASS | lib.rs:6-13 shows only: audio, commands, events, hotkey, model, parakeet, recording, voice_commands |
| All references to `whisper::` module in `lib.rs` removed | PASS | lib.rs reviewed - no whisper:: references, uses `parakeet::TranscriptionManager` at line 72 |
| All references to `WhisperManager` in `hotkey/integration.rs` removed | PASS | integration.rs:19 imports `parakeet::{TranscriptionManager, TranscriptionService}`, no whisper imports |
| Application compiles without errors after removal | PASS | `cargo build` succeeds with only warnings (unused functions) |
| Application starts and Parakeet transcription works correctly | DEFERRED | Manual runtime verification required |
| No references to "whisper" remain in Rust source code (except comments/docs) | FAIL | Found references in comments/docs that should be updated |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `cargo build` succeeds after whisper removal | PASS | Verified via cargo build |
| `cargo test` passes (no tests reference whisper directly) | PASS | 252 tests passed, 0 failed |
| Application launches without errors | DEFERRED | Manual verification required |
| Recording and transcription workflow completes successfully with Parakeet | DEFERRED | Manual verification required |
| No "whisper" strings appear in `cargo tree` output | PASS | "No whisper dependencies found" |
| Binary size is reduced | DEFERRED | Binary size comparison not performed |

### Code Quality

**Strengths:**
- Clean removal of whisper-rs dependency from Cargo.toml
- No whisper module declaration in lib.rs
- HotkeyIntegration correctly uses TranscriptionManager from parakeet module
- All 252 tests pass without modification
- No whisper dependencies in cargo tree

**Concerns:**
- Several "whisper" references remain in comments/documentation that should be updated for accuracy:
  - `src-tauri/src/audio/mod.rs:49`: Comment says "16 kHz for Whisper compatibility" - should be updated to mention Parakeet
  - `src-tauri/src/model/mod.rs:2`: "Handles whisper model download" - outdated comment
  - `src-tauri/src/model/mod.rs:13,19`: Doc comments refer to "whisper model"
  - `src-tauri/src/model/download.rs:10`: MODEL_URL points to `whisper.cpp` repo on HuggingFace - this is correct as Parakeet uses the same GGML model format
  - `src-tauri/src/model/download.rs:52,57`: Doc comments refer to "whisper model"

The spec states "No references to 'whisper' remain in Rust source code (except comments/docs)" - the current state technically passes this criterion since the remaining references ARE in comments/docs. However, these comments should ideally be updated for clarity and maintainability.

### Verdict

**NEEDS_WORK** - While the core whisper-rs dependency removal is complete and the code compiles and tests pass, the acceptance criterion "No references to 'whisper' remain in Rust source code (except comments/docs)" needs clarification. The spec explicitly allows comments/docs exceptions, so if this is acceptable, the spec could be approved. However, for code quality, the outdated comments in the model module should be updated to reference Parakeet/GGML instead of Whisper. Recommend either:
1. Update comments to reference Parakeet (preferred for maintainability), OR
2. Clarify that the existing comments are intentionally preserved because the model IS from whisper.cpp repository

---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: []
review_round: 1
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

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Pre-Review Gates (Automated)

#### 1. Build Warning Check
```
cd src-tauri && cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)"
No warnings found
```
**PASS** - No unused/dead_code warnings.

#### 2. Command Registration Check
N/A - This spec does not add commands.

#### 3. Event Subscription Check
N/A - This spec does not add events.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| New constant `MIN_PARTIAL_VAD_CHUNK` defined in `audio_constants.rs` | PASS | `src-tauri/src/audio_constants.rs:46` |
| Constant has documentation explaining its purpose | PASS | Lines 40-45 contain detailed docstring explaining purpose |
| Magic number `256` replaced with constant in `detector.rs` | PASS | `src-tauri/src/listening/detector.rs:534` uses `MIN_PARTIAL_VAD_CHUNK` |
| Value equals `VAD_CHUNK_SIZE_16KHZ / 2` (512 / 2 = 256) | PASS | Definition: `VAD_CHUNK_SIZE_16KHZ / 2` at line 46 |
| `cargo test` passes | PASS | 362 tests passed, 0 failed |
| `cargo clippy` passes | PASS | No clippy warnings |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Constant value test in `audio_constants.rs` | PASS | `src-tauri/src/audio_constants.rs:231-235` - `test_min_partial_vad_chunk_is_half_of_full_chunk` |
| Wake word detector tests continue to pass | PASS | All 362 tests pass including detector tests |

### Manual Review Questions

#### 1. Is the code wired up end-to-end?
- [x] New constant is imported and used in production code (`detector.rs:7`, `detector.rs:534`)
- [x] Constant is used in `check_vad_internal()` function which is called from production path

#### 2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `MIN_PARTIAL_VAD_CHUNK` | const | `detector.rs:534` in `check_vad_internal()` | YES - via wake word detection pipeline |

#### 3. Where does the data flow?
N/A - This is a constant extraction refactoring with no data flow changes.

#### 4. Are there any deferrals?
No deferrals found in the implementation files.

### Code Quality

**Strengths:**
- Constant is properly derived from `VAD_CHUNK_SIZE_16KHZ / 2` rather than hardcoded, ensuring consistency
- Excellent documentation explaining the constant's purpose, value rationale, and usage context
- Test validates both the formula relationship and the absolute value (256)
- Constant placement follows existing organization pattern in `audio_constants.rs`

**Concerns:**
- None identified

### Verdict

**APPROVED** - The spec successfully extracts the magic number `256` to a well-documented constant `MIN_PARTIAL_VAD_CHUNK`. The constant is properly defined in terms of `VAD_CHUNK_SIZE_16KHZ`, imported and used in production code at `detector.rs:534`, and covered by a dedicated unit test. All automated checks pass with no warnings.

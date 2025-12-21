---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: []
review_round: 1
---

# Spec: Adopt Thiserror for Remaining Error Types

## Description

Migrate three error types that currently implement `Display` and `Error` traits manually to use the `thiserror` derive macro. This brings consistency with `ActionError` which already uses thiserror, and reduces boilerplate code.

## Acceptance Criteria

- [ ] `ListeningError` in `listening/manager.rs` uses `#[derive(thiserror::Error)]`
- [ ] `WakeWordError` in `listening/detector.rs` uses `#[derive(thiserror::Error)]`
- [ ] `TranscriptionError` in `parakeet/types.rs` uses `#[derive(thiserror::Error)]`
- [ ] All manual `impl Display` and `impl Error` blocks removed
- [ ] Error messages remain identical to current behavior
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Test Cases

- [ ] Existing tests continue to pass (error types are tested via behavior, not Display output)
- [ ] Error formatting produces same messages as before

## Dependencies

None

## Preconditions

`thiserror = "2"` is already in Cargo.toml

## Implementation Notes

**Files to modify:**

1. `src-tauri/src/listening/manager.rs:21-55`
   - Current: Manual `Display` implementation with match statement
   - Change to: `#[derive(thiserror::Error)]` with `#[error("...")]` attributes

2. `src-tauri/src/listening/detector.rs:119-172`
   - Current: Manual `Display` and `Error` implementations
   - Change to: `#[derive(thiserror::Error)]` with `#[error("...")]` attributes

3. `src-tauri/src/parakeet/types.rs:23-51`
   - Current: Manual `Display` and `Error` implementations
   - Change to: `#[derive(thiserror::Error)]` with `#[error("...")]` attributes

**Example transformation for ListeningError:**

```rust
// Before
#[derive(Debug, Clone, PartialEq)]
pub enum ListeningError {
    InvalidTransition { current_state: RecordingState },
    RecordingInProgress,
    LockError,
    AlreadyInState,
}

impl std::fmt::Display for ListeningError { ... }
impl std::error::Error for ListeningError {}

// After
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ListeningError {
    #[error("Cannot change listening state from {current_state:?}")]
    InvalidTransition { current_state: RecordingState },
    #[error("Cannot enable listening while recording")]
    RecordingInProgress,
    #[error("Failed to acquire state lock")]
    LockError,
    #[error("Already in the requested listening state")]
    AlreadyInState,
}
```

## Related Specs

None

## Integration Points

- Production call site: Error types used throughout recording/listening subsystem
- Connects to: commands module (converts errors to strings at Tauri boundary)

## Integration Test

- Test location: N/A (unit-only spec - error formatting is internal)
- Verification: [x] N/A

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `ListeningError` in `listening/manager.rs` uses `#[derive(thiserror::Error)]` | PASS | `src-tauri/src/listening/manager.rs:21` - `#[derive(Debug, Clone, PartialEq, thiserror::Error)]` |
| `WakeWordError` in `listening/detector.rs` uses `#[derive(thiserror::Error)]` | PASS | `src-tauri/src/listening/detector.rs:120` - `#[derive(Debug, Clone, PartialEq, thiserror::Error)]` |
| `TranscriptionError` in `parakeet/types.rs` uses `#[derive(thiserror::Error)]` | PASS | `src-tauri/src/parakeet/types.rs:23` - `#[derive(Debug, Clone, PartialEq, thiserror::Error)]` |
| All manual `impl Display` and `impl Error` blocks removed | PASS | Grep search for `impl.*Display.*for.*(ListeningError\|WakeWordError\|TranscriptionError)` returns no matches |
| Error messages remain identical to current behavior | PASS | Compared all `#[error("...")]` attributes against original `impl Display` match arms - all messages preserved identically |
| `cargo test` passes | PASS | 361 tests passed, 0 failed |
| `cargo clippy` passes | PASS | No warnings or errors |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Existing tests continue to pass | PASS | `cargo test` - 361 passed, 0 failed |
| Error formatting produces same messages as before | PASS | Verified by comparing git history (commit d39dac4^) against current implementation |

### Code Quality

**Strengths:**
- Clean adoption of thiserror derive macro on all three error types
- Error messages preserved exactly from original manual implementations
- Consistent with ActionError which already uses thiserror
- Significant boilerplate reduction (~50 lines removed across 3 files)
- Proper use of `#[allow(dead_code)]` on error variants reserved for future use (e.g., `AlreadyInState`, `ModelLoadFailed`)

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria met. The three error types (`ListeningError`, `WakeWordError`, `TranscriptionError`) have been successfully migrated to use `thiserror::Error` derive macro. All manual `impl Display` and `impl Error` blocks have been removed. Error messages are identical to the previous implementation. All tests pass and clippy reports no issues.

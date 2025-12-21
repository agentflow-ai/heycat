---
status: in-progress
created: 2025-12-21
completed: null
dependencies: []
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

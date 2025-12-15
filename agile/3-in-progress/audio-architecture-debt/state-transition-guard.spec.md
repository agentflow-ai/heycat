---
status: in-progress
created: 2025-12-15
completed: null
dependencies:
  - shared-transcription-model
---

# Spec: Fix state transition race condition with RAII guard

## Description

Fix the race condition where transcription state is set to "Transcribing" before the operation actually starts. Currently, there's a window where `state()` returns `Transcribing` but no operation is running, and if transcription fails, state can get stuck.

## Acceptance Criteria

- [ ] Create `TranscribingGuard` RAII struct
- [ ] State transitions to `Transcribing` only when guard is acquired
- [ ] State automatically resets to `Idle` when guard is dropped
- [ ] Handle panic/error cases with proper cleanup
- [ ] No window where state is inconsistent
- [ ] Concurrent state queries return accurate value

## Test Cases

- [ ] Unit test: Guard sets state to Transcribing on creation
- [ ] Unit test: Guard resets state to Idle on drop
- [ ] Unit test: Guard resets state to Idle on panic
- [ ] Unit test: Guard sets state to Error on explicit error
- [ ] Integration test: Concurrent state queries are consistent
- [ ] Stress test: Rapid start/stop doesn't leave stuck state

## Dependencies

- `shared-transcription-model` - Should work with shared model

## Preconditions

- SharedTranscriptionModel implemented
- Understanding of RAII patterns in Rust

## Implementation Notes

```rust
// src-tauri/src/parakeet/manager.rs

/// RAII guard that manages transcription state transitions.
///
/// - Sets state to `Transcribing` on creation
/// - Sets state to `Idle` on drop (success) or `Error` on explicit error
/// - Handles panics by resetting to `Idle`
pub struct TranscribingGuard<'a> {
    state: &'a Mutex<TranscriptionState>,
    completed: bool,
}

impl<'a> TranscribingGuard<'a> {
    pub fn new(state: &'a Mutex<TranscriptionState>) -> Result<Self, TranscriptionError> {
        let mut guard = state.lock().map_err(|_| TranscriptionError::LockPoisoned)?;
        if *guard == TranscriptionState::Unloaded {
            return Err(TranscriptionError::ModelNotLoaded);
        }
        *guard = TranscriptionState::Transcribing;
        Ok(Self { state, completed: false })
    }

    pub fn complete_with_error(&mut self, error: TranscriptionError) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = TranscriptionState::Error(error.to_string());
        }
        self.completed = true;
    }
}

impl Drop for TranscribingGuard<'_> {
    fn drop(&mut self) {
        if !self.completed {
            if let Ok(mut guard) = self.state.lock() {
                *guard = TranscriptionState::Idle;
            }
        }
    }
}

// Usage in transcribe():
pub fn transcribe(&self, path: &Path) -> Result<String, TranscriptionError> {
    let _guard = TranscribingGuard::new(&self.state)?;

    // Actual transcription work here
    // State automatically resets to Idle when _guard drops

    let result = self.do_transcription(path)?;
    Ok(result)
}
```

Key changes:
- `parakeet/manager.rs:112-122` - Replace manual state setting with guard
- Remove explicit state reset in error paths (guard handles it)

## Related Specs

- `shared-transcription-model.spec.md` - Prerequisite
- `transcription-timeout.spec.md` - Related (both improve robustness)

## Integration Points

- Production call site: `src-tauri/src/parakeet/manager.rs` or `shared.rs`
- Connects to: `TranscriptionManager`, state query callers

## Integration Test

- Test location: `src-tauri/src/parakeet/manager_test.rs`
- Verification: [ ] Integration test passes

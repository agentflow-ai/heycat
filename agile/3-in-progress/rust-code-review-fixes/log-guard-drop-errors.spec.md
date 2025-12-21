---
status: pending
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Log Warning When TranscribingGuard Drop Fails

## Description

The `TranscribingGuard::Drop` implementation silently ignores lock acquisition failures. Add a warning log when the state lock is poisoned during drop, improving observability for debugging transcription state issues.

## Acceptance Criteria

- [ ] Warning logged when `self.state.lock()` fails in `Drop::drop()`
- [ ] Log message includes context about what failed
- [ ] No panic or error propagation (drop must not panic)
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Test Cases

- [ ] Existing `test_guard_resets_state_to_idle_on_panic` continues to pass
- [ ] Manual verification: poisoned lock produces warning in logs

## Dependencies

None

## Preconditions

`crate::warn!` macro is available (via log imports)

## Implementation Notes

**File to modify:** `src-tauri/src/parakeet/shared.rs:81-96`

**Current code:**
```rust
impl Drop for TranscribingGuard {
    fn drop(&mut self) {
        if !self.completed {
            if let Ok(mut guard) = self.state.lock() {
                if *guard == TranscriptionState::Transcribing {
                    *guard = TranscriptionState::Idle;
                }
            }
            // Silently ignores Err case
        }
    }
}
```

**Suggested fix:**
```rust
impl Drop for TranscribingGuard {
    fn drop(&mut self) {
        if !self.completed {
            match self.state.lock() {
                Ok(mut guard) => {
                    if *guard == TranscriptionState::Transcribing {
                        *guard = TranscriptionState::Idle;
                    }
                }
                Err(_) => {
                    crate::warn!(
                        "Failed to reset transcription state in drop - lock poisoned"
                    );
                }
            }
        }
    }
}
```

## Related Specs

None

## Integration Points

- Production call site: `src-tauri/src/parakeet/shared.rs` - used in transcription flow
- Connects to: TranscribingGuard RAII pattern for transcription state management

## Integration Test

- Test location: N/A (observability improvement, tested via existing guard tests)
- Verification: [x] N/A

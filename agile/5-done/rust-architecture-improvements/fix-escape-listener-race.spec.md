---
status: completed
created: 2025-12-20
completed: 2025-12-20
dependencies: []
review_round: 1
---

# Spec: Fix Escape Listener Race Condition

## Description

Fix race condition in `register_escape_listener` where `escape_registered` is set to `true` optimistically BEFORE the spawned thread completes registration. If registration fails in the background thread, `escape_registered` remains true, causing `unregister_escape_listener` to attempt unregistering a never-registered shortcut.

## Acceptance Criteria

- [ ] `escape_registered` only set to `true` after successful registration
- [ ] Registration failure properly reflected in state
- [ ] No spurious warnings in logs from failed unregister attempts
- [ ] Thread-safe state update mechanism implemented

## Test Cases

- [ ] Test that escape_registered is false when registration fails
- [ ] Test that unregister doesn't warn when registration never succeeded
- [ ] Test normal successful registration path

## Dependencies

None

## Preconditions

None

## Implementation Notes

Location: `src-tauri/src/hotkey/integration.rs:1262-1293`

Current problematic code:
```rust
#[cfg(not(test))]
{
    self.escape_registered = true;  // Set optimistically BEFORE spawning
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        match backend.register(...) { ... }
    });
}
```

Options:
1. Use `Arc<AtomicBool>` for `escape_registered` and set it in the spawned thread after success
2. Use a channel to communicate registration result back to main thread
3. Make registration synchronous (simpler but blocks caller)

Recommend option 1 (AtomicBool) for minimal code change.

## Related Specs

None

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: ShortcutBackend (CGEventTap on macOS, Tauri on other platforms)

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [x] Integration test passes

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `escape_registered` only set to `true` after successful registration | PASS | Line 1287: `escape_registered.store(true, Ordering::SeqCst)` only called inside `Ok(())` branch after successful registration |
| Registration failure properly reflected in state | PASS | Line 1292: Comment confirms `escape_registered remains false` on error path, no store(true) called |
| No spurious warnings in logs from failed unregister attempts | PASS | Line 1320-1322: Early return when `!escape_registered.load()`, preventing unregister attempt when registration never succeeded |
| Thread-safe state update mechanism implemented | PASS | Line 119: `escape_registered: Arc<AtomicBool>` and all accesses use SeqCst ordering (lines 1210, 1254, 1287, 1320, 1330) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Test that escape_registered is false when registration fails | PASS | integration_test.rs:1197-1222 `test_escape_registered_false_when_registration_fails` |
| Test that unregister doesn't warn when registration never succeeded | PASS | integration_test.rs:1224-1256 `test_unregister_not_called_when_registration_failed` |
| Test normal successful registration path | PASS | integration_test.rs:639-665 `test_escape_listener_registered_when_recording_starts` |

### Code Quality

**Strengths:**
- Proper use of `Arc<AtomicBool>` for thread-safe state management
- Clean separation of test vs production code paths using cfg attributes
- Comprehensive test coverage including failure scenarios
- Clear comments explaining the race condition fix (lines 1286-1287, 1292)
- Early return guard prevents unregister attempts when registration failed (lines 1320-1322)

**Concerns:**
- None identified

### Verdict

**APPROVED** - Implementation correctly fixes the race condition by deferring `escape_registered` state update until after successful registration. Thread-safe state management using `Arc<AtomicBool>` is properly implemented. All acceptance criteria met with comprehensive test coverage including both success and failure paths.

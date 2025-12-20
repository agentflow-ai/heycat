---
status: pending
created: 2025-12-20
completed: null
dependencies: []
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
- Verification: [ ] Integration test passes

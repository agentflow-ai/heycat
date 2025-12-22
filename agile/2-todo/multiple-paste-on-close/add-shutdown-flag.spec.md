---
status: pending
created: 2025-12-22
completed: null
dependencies: []
---

# Spec: Add Shutdown Flag

## Description

Add a global shutdown flag (`AtomicBool`) that prevents `simulate_paste()` from firing during app shutdown. Signal the flag on `WindowEvent::Destroyed` before cleanup, and check it in all paste code paths.

## Acceptance Criteria

- [ ] Create `src-tauri/src/shutdown.rs` with `signal_shutdown()` and `is_shutting_down()` functions
- [ ] Call `signal_shutdown()` first thing in `WindowEvent::Destroyed` handler
- [ ] Guard `copy_and_paste()` in `integration.rs` with shutdown check
- [ ] Guard paste logic in `service.rs` with shutdown check
- [ ] Add safety check inside both `simulate_paste()` function definitions

## Test Cases

- [ ] Manual: Close app after transcription - no paste operations fire
- [ ] `is_shutting_down()` returns false before signal, true after

## Dependencies

None

## Preconditions

App compiles and runs normally

## Implementation Notes

**Files to modify:**
- `src-tauri/src/shutdown.rs` (NEW) - Global shutdown flag
- `src-tauri/src/lib.rs` - Add `mod shutdown;`, call `signal_shutdown()` on destroy
- `src-tauri/src/hotkey/integration.rs:271` - Guard `copy_and_paste()`
- `src-tauri/src/hotkey/integration.rs:45` - Guard `simulate_paste()`
- `src-tauri/src/transcription/service.rs:356` - Guard paste call
- `src-tauri/src/transcription/service.rs:33` - Guard `simulate_paste()`

**shutdown.rs content:**
```rust
use std::sync::atomic::{AtomicBool, Ordering};

static APP_SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

pub fn signal_shutdown() {
    APP_SHUTTING_DOWN.store(true, Ordering::SeqCst);
    crate::info!("App shutdown signaled");
}

pub fn is_shutting_down() -> bool {
    APP_SHUTTING_DOWN.load(Ordering::SeqCst)
}
```

## Related Specs

None (single spec fix)

## Integration Points

- Production call site: `lib.rs:337` (WindowEvent::Destroyed handler)
- Connects to: `hotkey/integration.rs`, `transcription/service.rs`

## Integration Test

- Test location: N/A (manual test)
- Verification: [x] N/A

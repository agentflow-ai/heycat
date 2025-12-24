---
status: completed
created: 2025-12-22
completed: 2025-12-23
dependencies: []
review_round: 1
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

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Create `src-tauri/src/shutdown.rs` with `signal_shutdown()` and `is_shutting_down()` functions | PASS | `src-tauri/src/shutdown.rs:11-19` - Both functions implemented with AtomicBool and SeqCst ordering |
| Call `signal_shutdown()` first thing in `WindowEvent::Destroyed` handler | PASS | `src-tauri/src/lib.rs:429` - Called immediately after matching Destroyed event, before any cleanup |
| Guard `copy_and_paste()` in `integration.rs` with shutdown check | PASS | `src-tauri/src/hotkey/integration.rs:279-282` - Early return if shutting down |
| Guard paste logic in `service.rs` with shutdown check | PASS | `src-tauri/src/transcription/service.rs:358` - Shutdown check in conditional before paste |
| Add safety check inside both `simulate_paste()` function definitions | PASS | `src-tauri/src/hotkey/integration.rs:47-50` and `src-tauri/src/transcription/service.rs:35-38` - Both implementations guarded |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Manual: Close app after transcription - no paste operations fire | MANUAL | N/A - requires manual verification |
| `is_shutting_down()` returns false before signal, true after | PASS | `src-tauri/src/shutdown.rs:27-40` - Unit test `test_shutdown_flag_transitions` verifies state transitions |

### Code Quality

**Strengths:**
- Clean, minimal implementation using `AtomicBool` with `SeqCst` ordering for maximum safety across threads
- Defense in depth: shutdown checks at multiple levels (caller function + simulate_paste itself)
- Good comments explaining the purpose of each guard
- Includes unit test for the state transition behavior

**Concerns:**
- None identified. Pre-existing warnings (unused functions in other modules) are unrelated to this spec.

### Verdict

**APPROVED** - All acceptance criteria met. The shutdown flag is properly wired end-to-end: created in shutdown.rs, signaled in WindowEvent::Destroyed handler before cleanup, and checked in all paste code paths (copy_and_paste, both simulate_paste definitions, and service.rs paste logic). Unit test passes verifying state transitions.

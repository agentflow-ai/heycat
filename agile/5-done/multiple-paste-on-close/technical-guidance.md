---
last-updated: 2025-12-24
status: finalized
---

# Technical Guidance: Multiple Paste On Close

## Root Cause Analysis

**Initial hypothesis (WRONG):** Async transcription tasks continuing after window destruction.

**Actual root cause:** Calling `std::process::exit(0)` from a signal handler (ctrlc callback) is NOT async-signal-safe and causes undefined behavior on macOS. When Ctrl+C is pressed during `tauri dev`, the signal handler runs but `exit()` doesn't terminate cleanly - macOS's CGEvent system and the terminal are left in a corrupted state, causing spurious keyboard events.

**Diagnostic proof:** Added `[PASTE-TRACE]` logging to all paste paths. During Ctrl+C shutdown:
- NO trace logs from Rust paste code appeared
- Paste events occurred AFTER `exit(0)` was called
- Terminal left in broken state (arrow keys show escape sequences)

**Problem Flow:**
1. User presses Ctrl+C in terminal
2. ctrlc handler fires, calls `signal_shutdown()` and `stop_cgeventtap()`
3. Handler calls `std::process::exit(0)` - NOT async-signal-safe
4. Process doesn't exit cleanly, undefined behavior occurs
5. macOS replays or generates spurious keyboard events
6. Terminal state is corrupted

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/shutdown.rs` | Shutdown coordination: `signal_shutdown()`, `is_shutting_down()`, `register_app_handle()`, `request_app_exit()` |
| `src-tauri/src/lib.rs` | ctrlc handler using `request_app_exit(0)` for graceful shutdown |
| `src-tauri/src/keyboard/synth.rs` | Centralized keyboard synthesis (Cmd+V paste) |
| `src-tauri/src/transcription/service.rs` | `simulate_paste()` with shutdown guard |
| `src-tauri/src/hotkey/integration.rs` | `simulate_paste()` and `copy_and_paste()` with shutdown guards |

## Fix Approach

Use Tauri's graceful exit mechanism instead of `std::process::exit()`:

1. **Store AppHandle globally**: `shutdown::register_app_handle(app.handle().clone())` in setup
2. **Use graceful exit**: `shutdown::request_app_exit(0)` calls `AppHandle::exit(0)` which handles cleanup properly
3. **Keep shutdown guards**: Defense-in-depth for `simulate_paste()` calls
4. **Centralized synthesis**: `keyboard/synth.rs` module for consistent Cmd+V behavior

## Regression Risk

- Low risk - graceful exit is the intended Tauri pattern
- All paste operations still guarded by shutdown flag
- Normal transcription unaffected

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-23 | Two `simulate_paste()` locations identified | Both need guards |
| 2025-12-23 | Semaphore allows 2 concurrent transcriptions | Multiple paste events possible |
| 2025-12-23 | No existing shutdown coordination | Initial fix: added shutdown flag |
| 2025-12-23 | Ctrl+C (SIGINT) bypasses WindowEvent::Destroyed | Added ctrlc handler |
| 2025-12-23 | "ipipipip" spurious events after SIGINT handler | CGEventTap not cleaned up before exit |
| 2025-12-23 | `std::process::exit(0)` prevents CGEventTap cleanup | Added CFRunLoopStop |
| 2025-12-24 | Diagnostic logging shows NO paste trace during shutdown | Paste NOT from Rust code |
| 2025-12-24 | Paste occurs AFTER exit(0), terminal corrupted | `exit()` in signal handler is unsafe |
| 2025-12-24 | **Root cause confirmed**: `std::process::exit(0)` is not async-signal-safe | Use `AppHandle::exit(0)` instead |

## Definition of Done

- [x] Shutdown flag module created with signal_shutdown() and is_shutting_down()
- [x] All simulate_paste() calls guarded with shutdown check
- [x] Unit test for shutdown flag state transitions
- [x] Graceful exit via AppHandle::exit() instead of std::process::exit()
- [x] Centralized keyboard synthesis in keyboard/synth.rs
- [x] Bug review passed
- [x] Manual testing confirms fix works

## Open Questions

- [x] Where are all `simulate_paste()` calls? → service.rs and integration.rs
- [x] Why do paste events occur after exit()? → `exit()` from signal handler is undefined behavior

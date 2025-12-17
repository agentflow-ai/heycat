---
status: in-review
created: 2025-12-17
completed: null
dependencies:
  - escape-key-listener
---

# Spec: Detect double-tap pattern for cancel

## Description

Implement double-tap detection with a configurable time window. Single Escape key taps are ignored; only double-taps within the time window trigger the cancellation flow.

## Acceptance Criteria

- [ ] Double-tap detected within configurable time window (default 300ms)
- [ ] Single Escape tap does not cancel recording
- [ ] Triple+ taps within window treated as double-tap (cancel once)
- [ ] Timestamp tracking for tap detection

## Test Cases

- [ ] Two taps within 300ms triggers cancel
- [ ] Two taps with 500ms gap does not trigger cancel
- [ ] Single tap followed by nothing does not trigger cancel
- [ ] Three rapid taps triggers cancel only once
- [ ] Time window is configurable

## Dependencies

- escape-key-listener (provides Escape key events)

## Preconditions

- Escape key listener registered and firing events

## Implementation Notes

- Create `DoubleTapDetector` struct with configurable threshold
- Store `last_tap_time: Option<Instant>`
- On tap: check if within window of last tap
- If yes: trigger cancel callback, reset state
- If no: update last_tap_time

## Related Specs

- escape-key-listener.spec.md (provides events)
- cancel-recording-flow.spec.md (triggered on double-tap)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: Escape key listener callback

## Integration Test

- Test location: `src-tauri/src/hotkey/double_tap_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Double-tap detected within configurable time window (default 300ms) | PASS | `src-tauri/src/hotkey/double_tap.rs:8-9` - DEFAULT_DOUBLE_TAP_WINDOW_MS = 300, `double_tap.rs:39-45` - `with_window()` constructor |
| Single Escape tap does not cancel recording | PASS | `src-tauri/src/hotkey/double_tap.rs:54-69` - `on_tap()` only triggers on second tap within window |
| Triple+ taps within window treated as double-tap (cancel once) | PASS | `src-tauri/src/hotkey/double_tap.rs:62` - State reset after trigger prevents re-triggering |
| Timestamp tracking for tap detection | PASS | `src-tauri/src/hotkey/double_tap.rs:21` - `last_tap_time: Option<Instant>` field |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Two taps within 300ms triggers cancel | PASS | `src-tauri/src/hotkey/double_tap.rs:94` - `test_two_taps_within_window_triggers_cancel` |
| Two taps with 500ms gap does not trigger cancel | PASS | `src-tauri/src/hotkey/double_tap.rs:114` - `test_two_taps_outside_window_does_not_trigger` (uses 50ms/60ms) |
| Single tap followed by nothing does not trigger cancel | PASS | `src-tauri/src/hotkey/double_tap.rs:139` - `test_single_tap_does_not_trigger` |
| Three rapid taps triggers cancel only once | PASS | `src-tauri/src/hotkey/double_tap.rs:154` - `test_three_rapid_taps_triggers_only_once` |
| Time window is configurable | PASS | `src-tauri/src/hotkey/double_tap.rs:171` - `test_time_window_is_configurable` |

### Code Quality

**Strengths:**
- Clean, well-documented implementation with clear ownership semantics
- Comprehensive test coverage with both unit tests (8 tests) and integration tests (1 test)
- Proper state management with explicit reset functionality
- Generic callback support with `Send + Sync` bounds for thread safety

**Concerns:**
- Build warning: `unused import: double_tap::DoubleTapDetector` at `mod.rs:7` - The re-export is not used externally
- Build warning: `associated function 'new' is never used` at `double_tap.rs:30` - Production code only uses `with_window()`, not `new()`

### Verdict

**NEEDS_WORK** - Build warnings indicate dead code that should be cleaned up. The `pub use double_tap::DoubleTapDetector` re-export is unused, and the `new()` constructor is never called (only `with_window()` is used in production). Either remove these unused items or add `#[allow(dead_code)]` annotations with justification if they're intended for future public API use.

---
status: completed
created: 2025-12-17
completed: 2025-12-17
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

### Pre-Review Gates

**1. Build Warning Check:**
```
cd src-tauri && cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)"
```
Result: No output (no warnings)

**2. Command Registration Check:** N/A (no new commands)

**3. Event Subscription Check:** N/A (no new events)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Double-tap detected within configurable time window (default 300ms) | PASS | `src-tauri/src/hotkey/double_tap.rs:8-9` - DEFAULT_DOUBLE_TAP_WINDOW_MS = 300, `double_tap.rs:30-36` - `with_window()` constructor |
| Single Escape tap does not cancel recording | PASS | `src-tauri/src/hotkey/double_tap.rs:45-61` - `on_tap()` only triggers on second tap within window |
| Triple+ taps within window treated as double-tap (cancel once) | PASS | `src-tauri/src/hotkey/double_tap.rs:53` - State reset after trigger prevents re-triggering |
| Timestamp tracking for tap detection | PASS | `src-tauri/src/hotkey/double_tap.rs:21` - `last_tap_time: Option<Instant>` field |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Two taps within 300ms triggers cancel | PASS | `src-tauri/src/hotkey/double_tap.rs:85` - `test_two_taps_within_window_triggers_cancel` |
| Two taps with 500ms gap does not trigger cancel | PASS | `src-tauri/src/hotkey/double_tap.rs:107` - `test_two_taps_outside_window_does_not_trigger` |
| Single tap followed by nothing does not trigger cancel | PASS | `src-tauri/src/hotkey/double_tap.rs:132` - `test_single_tap_does_not_trigger` |
| Three rapid taps triggers cancel only once | PASS | `src-tauri/src/hotkey/double_tap.rs:150` - `test_three_rapid_taps_triggers_only_once` |
| Time window is configurable | PASS | `src-tauri/src/hotkey/double_tap.rs:170` - `test_time_window_is_configurable` |

Additional tests found:
- `test_default_window_is_300ms` (line 197)
- `test_reset_clears_state` (line 204)
- `test_multiple_double_tap_cycles` (line 228)
- Integration test: `test_escape_double_tap_window_is_configurable` (integration_test.rs:847)

### Integration Verification

**1. Is the code wired up end-to-end?**

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `DoubleTapDetector` | struct | `integration.rs:1182` | YES |
| `DEFAULT_DOUBLE_TAP_WINDOW_MS` | const | `integration.rs:14` import, `integration.rs:150` | YES |
| `with_window()` | fn | `integration.rs:1183` | YES |
| `on_tap()` | fn | `integration.rs:1191` | YES |
| `reset()` | fn | `integration.rs:1219` | YES |

**2. Data Flow Trace:**

```
[Hotkey Toggle - Start Recording]
     |
     v
[HotkeyIntegration::handle_toggle] src-tauri/src/hotkey/integration.rs:345
     | calls register_escape_listener()
     v
[register_escape_listener] src-tauri/src/hotkey/integration.rs:1156
     | creates DoubleTapDetector::with_window()
     | registers Escape key via backend
     v
[ShortcutBackend::register] src-tauri/src/hotkey/tauri_backend.rs
     |
     v
[Escape Key Press]
     |
     v
[Callback in register_escape_listener] src-tauri/src/hotkey/integration.rs:1189-1196
     | calls detector.on_tap()
     v
[DoubleTapDetector::on_tap] src-tauri/src/hotkey/double_tap.rs:45
     | if double-tap detected: invoke callback
     v
[escape_callback] src-tauri/src/lib.rs:158-160
```

**3. Production Wiring (lib.rs:153-177):**
- `escape_backend` created at line 154-155
- `escape_callback` created at line 158-160
- Wired via `.with_shortcut_backend(escape_backend)` at line 176
- Wired via `.with_escape_callback(escape_callback)` at line 177

**4. Deferrals Check:**
No TODOs, FIXMEs, or deferrals found in double_tap.rs.

### Code Quality

**Strengths:**
- Clean, well-documented implementation with clear ownership semantics
- Comprehensive test coverage: 8 unit tests + 1 integration test (9 total, all pass)
- Proper state management with explicit reset functionality
- Generic callback support with `Send + Sync` bounds for thread safety
- No build warnings - previous concerns about unused code have been resolved
- Production wiring verified in lib.rs

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria verified with evidence. Test coverage is comprehensive (9 tests, all passing). Code is fully wired to production via HotkeyIntegration. No build warnings. Data flow is complete from Escape key press through DoubleTapDetector to callback invocation.

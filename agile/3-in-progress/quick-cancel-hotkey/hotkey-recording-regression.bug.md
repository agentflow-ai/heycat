---
status: in-review
severity: critical
origin: testing
created: 2025-12-17
completed: null
parent_feature: "quick-cancel-hotkey"
parent_spec: null
review_round: 1
---

# Bug: Hotkey Recording Regression

**Created:** 2025-12-17
**Severity:** Critical

## Problem Description

After starting a recording, the frontend app becomes unresponsive. The last log message seen is:
```
[2025-12-17][18:24:03][heycat_lib::hotkey::integration][INFO] Recording started, emitted recording_started event
```

**Expected:** Recording starts and user can interact with the app, use hotkeys, and cancel recording with ESC double-tap.

**Actual:**
1. Frontend app becomes unresponsive after recording starts
2. Hotkey no longer works (cannot stop recording)
3. ESC + Cancel + Recording feature does not work

This regression occurred while implementing the quick-cancel-hotkey feature.

## Steps to Reproduce

1. Start the application
2. Trigger the recording hotkey
3. Observe the log message "Recording started, emitted recording_started event"
4. Try to interact with the frontend - it's unresponsive
5. Try the recording hotkey again - it doesn't work
6. Try double-tap ESC to cancel - it doesn't work

## Root Cause

**Re-entrancy deadlock in global shortcut registration.**

When the recording hotkey (Cmd+Shift+R) is pressed:
1. Tauri's global shortcut manager holds a lock while executing the callback
2. Inside `handle_toggle()`, we call `register_escape_listener()`
3. `register_escape_listener()` calls `backend.register()` which calls `app.global_shortcut().on_shortcut()`
4. This tries to acquire the same lock that's already held by the executing callback
5. **Deadlock** - the app freezes

The same issue affects `unregister_escape_listener()` when called from:
- The recording hotkey callback (when stopping recording)
- The Escape key callback (when cancelling via double-tap)

## Fix Approach

Defer both register and unregister operations to a spawned thread:
1. Mark `escape_registered` state optimistically/immediately
2. Spawn a thread with a small delay (10ms) to perform the actual registration/unregistration
3. The delay ensures the calling shortcut callback has completed and released its lock
4. If registration fails, log a warning but continue (graceful degradation - cancel feature won't work)

## Acceptance Criteria

- [ ] Bug no longer reproducible
- [ ] Recording starts and frontend remains responsive
- [ ] Hotkey works to stop recording
- [ ] ESC double-tap cancels recording
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Start recording and verify frontend responsiveness | Frontend remains responsive | [ ] |
| Start recording and use hotkey to stop | Recording stops successfully | [ ] |
| Start recording and double-tap ESC to cancel | Recording cancels without transcription | [ ] |

## Integration Points

- HotkeyIntegration (integration.rs)
- Escape key listener
- Frontend event handling (useRecording.ts)

## Integration Test

Manual testing required to verify end-to-end flow from hotkey press through recording state and UI responsiveness.

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Bug no longer reproducible | DEFERRED | Requires manual testing |
| Recording starts and frontend remains responsive | DEFERRED | Requires manual testing |
| Hotkey works to stop recording | DEFERRED | Requires manual testing |
| ESC double-tap cancels recording | DEFERRED | Requires manual testing |
| Root cause addressed (not just symptoms) | PASS | integration.rs:1170-1258 and 1269-1324 - Both `register_escape_listener()` and `unregister_escape_listener()` now defer actual registration/unregistration to a spawned thread with 10ms delay using `std::thread::spawn()`. The `#[cfg(not(test))]` blocks spawn the thread while test code uses synchronous calls. This directly addresses the re-entrancy deadlock. |
| Tests added to prevent regression | PASS | integration_test.rs has 12+ tests covering cancel functionality: test_cancel_recording_during_recording_clears_buffer, test_cancel_recording_does_not_emit_stopped_event, test_cancel_recording_emits_correct_payload, test_cancel_recording_ignored_when_not_recording, test_cancel_recording_ignored_when_processing, test_cancel_recording_with_audio_thread, test_cancel_recording_unregisters_escape_listener, test_cancel_recording_stops_silence_detection, test_cancel_recording_can_restart_after_cancel |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Cancel recording during recording clears buffer | PASS | src-tauri/src/hotkey/integration_test.rs:952 |
| Cancel does not emit stopped event | PASS | src-tauri/src/hotkey/integration_test.rs:977 |
| Cancel emits correct payload | PASS | src-tauri/src/hotkey/integration_test.rs:1000 |
| Cancel ignored when not recording | PASS | src-tauri/src/hotkey/integration_test.rs:1021 |
| Cancel ignored when processing | PASS | src-tauri/src/hotkey/integration_test.rs:1036 |
| Cancel with audio thread | PASS | src-tauri/src/hotkey/integration_test.rs:1060 |
| Cancel unregisters escape listener | PASS | src-tauri/src/hotkey/integration_test.rs:1085 |
| Cancel stops silence detection | PASS | src-tauri/src/hotkey/integration_test.rs:1114 |
| Can restart after cancel | PASS | src-tauri/src/hotkey/integration_test.rs:1143 |
| All backend tests pass | PASS | 313 tests passed, 0 failed |
| All frontend tests pass | PASS | 292 tests passed, 0 failed |

### Code Quality

**Strengths:**
- Fix directly addresses the root cause (re-entrancy deadlock) rather than just the symptoms
- Clean separation of test vs production code using `#[cfg(test)]` / `#[cfg(not(test))]` attributes
- Comprehensive test coverage for cancel functionality
- Full data flow wired up: escape callback in lib.rs:189-196 calls `cancel_recording()`, which emits `recording_cancelled` event, which is listened to in useRecording.ts:133-142

**Concerns:**
- `with_escape_callback` method (integration.rs:288) has dead_code warning - only used in tests, production uses `set_escape_callback` instead. This is acceptable since it's a builder pattern method for test ergonomics.
- Manual testing still required to verify end-to-end behavior in production (unit tests use mock backends that don't have deadlock issues)

### Pre-Review Gate Results

```
Build Warning Check:
warning: method `with_escape_callback` is never used
    = note: `#[warn(dead_code)]` on by default
```

Note: This warning is acceptable - the method exists for test ergonomics and is used extensively in tests. Production code uses `set_escape_callback` instead because it needs to set the callback after the integration is wrapped in `Arc<Mutex<>>` (to allow the callback to capture a reference to the integration itself).

### Verdict

**NEEDS_WORK** - The code fix is correctly implemented and all automated tests pass. However, the bug's core acceptance criteria (frontend remains responsive, hotkey works, ESC double-tap works) require manual testing that was not performed. The fix approach is sound and addresses the root cause, but verification requires running the actual application since the deadlock only occurs in production (test mocks don't reproduce the issue).

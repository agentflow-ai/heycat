---
status: pending
severity: critical
origin: testing
created: 2025-12-17
completed: null
parent_feature: "quick-cancel-hotkey"
parent_spec: null
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

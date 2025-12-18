---
status: pending
severity: major
origin: manual
created: 2025-12-18
completed: null
parent_feature: "ui-redesign"
parent_spec: null
---

# Bug: Dashboard recording UI state issues

**Created:** 2025-12-18
**Severity:** Major

## Problem Description

Multiple UI state synchronization issues on the new Dashboard page:

1. **Start Recording button doesn't update**: When pressing "Start Recording" in the dashboard, the button should change to "Stop Recording" but it stays as "Start Recording"

2. **Double-escape doesn't work when started via button**: When recording is started using the dashboard button, the double-escape hotkey does not stop the recording

3. **Listening Mode toggle shows wrong state**: The Listening Mode toggle displays as "off" when it is actually "on" (inverted state)

## Steps to Reproduce

### Bug 1: Start Recording button
1. Open the Dashboard
2. Click "Start Recording"
3. **Expected:** Button changes to "Stop Recording"
4. **Actual:** Button remains as "Start Recording"

### Bug 2: Double-escape not working
1. Open the Dashboard
2. Click "Start Recording" to begin recording
3. Press Escape twice quickly (double-escape)
4. **Expected:** Recording should stop
5. **Actual:** Recording continues, double-escape has no effect

### Bug 3: Listening Mode toggle inverted
1. Open the Dashboard
2. Observe the Listening Mode toggle
3. **Expected:** Toggle should reflect actual state (on when enabled, off when disabled)
4. **Actual:** Toggle shows "off" when listening mode is actually "on"

## Root Cause

1. **Start Recording button not updating**: The Dashboard component was not using `isRecording` state from `useRecording` hook and was only calling `startRecording()` without conditionally rendering button text or handling stop.

2. **Double-escape not working via button**: The escape key listener is set up in `HotkeyIntegration::handle_toggle()` which only runs when recording is started via the global hotkey. When started via the button (which calls `invoke("start_recording")`), the escape listener was never registered.

3. **Listening Mode toggle inverted**: The `useListening` hook initialized `isListening` to `false` and only updated via events. It didn't fetch the initial state from the backend on mount using `get_listening_status`, so if listening was already enabled, the UI showed "off".

## Fix Approach

1. **Start Recording button**: Updated Dashboard to use `isRecording` and `stopRecording` from `useRecording`, making the button toggle between Start/Stop based on recording state.

2. **Double-escape**: Added frontend keyboard event listener in Dashboard that detects double-escape (two Escape presses within 300ms) and calls `stopRecording()`. This mirrors the backend behavior for hotkey-started recordings.

3. **Listening Mode toggle**: Added `useEffect` in `useListening` hook to fetch initial state from backend via `get_listening_status` command on mount. Also added same pattern to `useRecording` hook for consistency.

## Acceptance Criteria

- [ ] Start Recording button changes to Stop Recording when clicked
- [ ] Double-escape hotkey stops recording regardless of how recording was started
- [ ] Listening Mode toggle correctly reflects the actual listening state
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression
- [ ] Related specs/features not broken

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Click Start Recording button | Button text changes to "Stop Recording" | [ ] |
| Start recording via button, press double-escape | Recording stops | [ ] |
| Enable listening mode, check toggle | Toggle shows "on" state | [ ] |
| Disable listening mode, check toggle | Toggle shows "off" state | [ ] |

## Integration Points

- Dashboard component state management
- Tauri backend recording state events
- Global hotkey event handling
- Listening mode state synchronization

## Integration Test

Test the full flow: start recording via dashboard → verify button state → stop via double-escape → verify button returns to start state

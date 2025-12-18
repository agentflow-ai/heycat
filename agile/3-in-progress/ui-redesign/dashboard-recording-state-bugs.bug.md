---
status: completed
severity: major
origin: manual
created: 2025-12-18
completed: 2025-12-18
parent_feature: "ui-redesign"
parent_spec: null
review_round: 1
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

## Review

**Reviewed:** 2025-12-18
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Start Recording button changes to Stop Recording when clicked | PASS | Dashboard.tsx:198-199 uses `isRecording` state from `useRecording` hook and conditionally renders "Stop Recording" / "Start Recording" |
| Double-escape hotkey stops recording regardless of how recording was started | PASS | Dashboard.tsx:78-112 implements frontend double-escape detection with 300ms window, calls `stopRecording()` |
| Listening Mode toggle correctly reflects the actual listening state | PASS | useListening.ts:94-108 fetches initial state via `get_listening_status` on mount, Dashboard.tsx:138 binds toggle to `isListening` state |
| Root cause addressed (not just symptoms) | PASS | All three root causes identified in bug description are addressed with proper fixes |
| Tests added to prevent regression | PASS | useRecording.test.ts (double-tap-escape handling), useListening.test.ts (initial state fetch), Dashboard.test.tsx (button behavior) |
| Related specs/features not broken | PASS | All 381 tests pass |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Recording state from events updates UI | PASS | src/hooks/useRecording.test.ts:141-180 |
| Double-tap-escape cancellation handled | PASS | src/hooks/useRecording.test.ts:186-264 |
| Initial listening state fetched on mount | PASS | src/hooks/useListening.test.ts:177-195 |
| Start recording button triggers recording | PASS | src/pages/Dashboard.test.tsx:174-179 |
| Listening toggle triggers enable/disable | PASS | src/pages/Dashboard.test.tsx:161-172 |

### Code Quality

**Strengths:**
- Clean separation of concerns: hooks manage state, Dashboard handles UI and user interactions
- Double-escape detection matches backend behavior (300ms window)
- Initial state fetch prevents UI/backend state mismatch
- Proper cleanup of keyboard event listeners on unmount

**Concerns:**
- None identified

### Pre-Review Gate Results

1. **Build Warning Check:** No warnings found (PASS)
2. **Command Registration Check:** `get_listening_status` and `get_recording_state` registered in lib.rs:262,269 (PASS)
3. **Event Subscription Check:** All events (recording_started, recording_stopped, listening_started, listening_stopped) have corresponding listeners in hooks (PASS)

### Data Flow Verification

**Bug 1 - Recording Button State:**
```
[UI: Click Start Recording]
     |
     v
[Dashboard.tsx:63-69 handleRecordingToggle]
     | calls startRecording() or stopRecording()
     v
[useRecording.ts:92-104] invoke("start_recording")
     |
     v
[Backend] recording_started event
     |
     v
[useRecording.ts:123-131] listen("recording_started")
     | setIsRecording(true)
     v
[Dashboard.tsx:198-199] Button re-renders with "Stop Recording"
```

**Bug 2 - Double-Escape:**
```
[UI: Press Escape twice within 300ms]
     |
     v
[Dashboard.tsx:83-101 handleEscapeKeyDown]
     | detects double-tap, calls stopRecording()
     v
[useRecording.ts:106-116] invoke("stop_recording")
     |
     v
[Backend] recording_stopped event
     |
     v
[useRecording.ts:135-142] listen("recording_stopped")
     | setIsRecording(false)
     v
[UI: Button returns to "Start Recording"]
```

**Bug 3 - Listening Toggle Initial State:**
```
[App Mount]
     |
     v
[useListening.ts:95-108 fetchInitialState]
     | invoke("get_listening_status")
     v
[Backend: get_listening_status_impl]
     | returns { enabled, active, micAvailable }
     v
[useListening.ts:100-101] setIsListening(status.enabled)
     |
     v
[Dashboard.tsx:138] Toggle checked={isListening}
```

### Verdict

**APPROVED** - All three bugs have been properly fixed with root causes addressed. The implementation correctly uses React hooks for state management, adds proper initial state fetching from the backend, and implements frontend double-escape detection. All 381 tests pass, data flows are complete end-to-end, and no deferrals or hardcoded state values were found in production code.

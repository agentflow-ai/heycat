---
status: pending
severity: major
origin: manual
created: 2025-12-22
completed: null
parent_feature: null
parent_spec: null
---

# Bug: Escape Key Not Stopping Recording

**Created:** 2025-12-22
**Owner:** Claude
**Severity:** Major

## Problem Description

Double escape key is supposed to stop recording in progress. Backend logs show it's working correctly (audio capture stopped, recording cancelled successfully), but the frontend UI still shows the recording status and recording overlay.

## Steps to Reproduce

1. Start recording (via hotkey or UI)
2. Press Escape twice quickly (double-tap within 300ms)
3. **Expected:** Recording stops, overlay hides, UI shows idle state
4. **Actual:** Backend cancels correctly (see logs), but UI still shows recording status and overlay

## Root Cause

The frontend Event Bridge (`src/lib/eventBridge.ts`) is missing a listener for the `recording_cancelled` event.

- Backend emits `recording_cancelled` when double-Escape cancels recording
- Event Bridge has listeners for `recording_started`, `recording_stopped`, `recording_error`
- **Missing:** `RECORDING_CANCELLED` constant and listener
- Without query invalidation, the recording state query cache stays stale
- `useRecording` hook returns `isRecording: true` (stale)
- `useCatOverlay` derives overlay mode as "recording" (based on stale state)

## Fix Approach

Add `recording_cancelled` event handling to the Event Bridge:
1. Add `RECORDING_CANCELLED: "recording_cancelled"` to `eventNames` object
2. Add listener that invalidates `getRecordingState` query

## Acceptance Criteria

- [x] Bug no longer reproducible
- [x] Root cause addressed (not just symptoms)
- [x] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Double-Escape during recording | Overlay hides, recording status clears | [x] |
| Event bridge receives `recording_cancelled` | Query invalidation triggered | [x] |

## Definition of Done

- [x] Event bridge handles `recording_cancelled` event
- [x] Unit tests verify query invalidation
- [x] Manual testing confirms UI updates correctly

## Bug Review

**Verdict: APPROVED_FOR_DONE**

Manual review by user. Bug fix implemented correctly - added missing `recording_cancelled` event handler to Event Bridge.

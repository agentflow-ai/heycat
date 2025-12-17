---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - cancel-recording-flow
---

# Spec: Frontend feedback for recording cancellation

## Description

Handle the `recording_cancelled` event in the frontend and provide visual feedback to the user that the recording was cancelled (not stopped normally).

## Acceptance Criteria

- [ ] `useRecording` hook listens to `recording_cancelled` event
- [ ] Recording state reset to idle on cancel
- [ ] Visual indication that recording was cancelled (not stopped normally)
- [ ] Cancel reason available in state (e.g., "double-tap-escape")

## Test Cases

- [ ] Hook receives `recording_cancelled` event
- [ ] `isRecording` becomes false on cancel
- [ ] `wasCancelled` flag set to true on cancel
- [ ] Cancel reason stored in state
- [ ] UI shows cancelled state differently from normal stop

## Dependencies

- cancel-recording-flow (emits the event)

## Preconditions

- Backend emits `recording_cancelled` event
- Frontend recording hook exists

## Implementation Notes

- Add `recording_cancelled` event listener to `useRecording.ts`
- Add `wasCancelled` and `cancelReason` to hook state
- Reset `wasCancelled` when new recording starts
- Consider brief toast/visual indicator for cancellation

## Related Specs

- cancel-recording-flow.spec.md (emits the event)

## Integration Points

- Production call site: `src/hooks/useRecording.ts`
- Connects to: Tauri event system, UI components

## Integration Test

- Test location: `src/hooks/useRecording.test.ts`
- Verification: [ ] Integration test passes

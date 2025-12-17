---
status: completed
created: 2025-12-17
completed: 2025-12-17
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

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Pre-Review Gates

#### 1. Build Warning Check
```
warning: method `with_escape_callback` is never used
    = note: `#[warn(dead_code)]` on by default
warning: `heycat` (lib) generated 1 warning
```
**Status:** PASS - The warning is for `with_escape_callback` at src/hotkey/integration.rs:288, which is unrelated to this spec. No new warnings introduced by this spec's code.

#### 2. Command Registration Check
**Status:** N/A - This spec does not add new commands.

#### 3. Event Subscription Check
**Status:** PASS
- Backend event `recording_cancelled` defined at: src-tauri/src/events.rs:12
- Backend event emitted at: src-tauri/src/hotkey/integration.rs:1319
- Frontend listener registered at: src/hooks/useRecording.ts:133-142

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| useRecording hook listens to recording_cancelled event | PASS | src/hooks/useRecording.ts:133-142 |
| Recording state reset to idle on cancel | PASS | src/hooks/useRecording.ts:136 sets isRecording to false |
| Visual indication that recording was cancelled (not stopped normally) | PASS | src/components/RecordingIndicator.tsx:25-26,32-33 shows "Cancelled" status with recording-indicator--cancelled CSS class |
| Cancel reason available in state (e.g., "double-tap-escape") | PASS | src/hooks/useRecording.ts:138 stores reason in cancelReason state |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Hook receives recording_cancelled event | PASS | src/hooks/useRecording.test.ts:178-219 |
| isRecording becomes false on cancel | PASS | src/hooks/useRecording.test.ts:215 |
| wasCancelled flag set to true on cancel | PASS | src/hooks/useRecording.test.ts:216 |
| Cancel reason stored in state | PASS | src/hooks/useRecording.test.ts:217 |
| UI shows cancelled state differently from normal stop | PASS | src/components/RecordingIndicator.test.tsx:134-150 |
| Cancelled state resets when new recording starts | PASS | src/hooks/useRecording.test.ts:221-263 |

### Data Flow

```
[Backend: Double-tap Escape Detection]
     |
     v
[Hotkey Integration] src-tauri/src/hotkey/integration.rs:1319
     | emit_recording_cancelled(RecordingCancelledPayload)
     v
[Event: recording_cancelled]
     |
     v
[Hook Listener] src/hooks/useRecording.ts:133-142
     | setIsRecording(false)
     | setWasCancelled(true)
     | setCancelReason(reason)
     v
[State Update] wasCancelled=true, cancelReason="double-tap-escape"
     |
     v
[UI Component] src/components/RecordingIndicator.tsx:25-26
     | Renders "Cancelled" text
     | Applies "recording-indicator--cancelled" CSS class
     v
[UI Re-render]
```

**Status:** COMPLETE - All links verified and functional.

### Code Quality

**Strengths:**
- Clean event-driven architecture with proper separation of concerns
- Hook properly listens to recording_cancelled event and updates state
- Cancel reason is captured and exposed via hook state
- Cancelled state properly resets when new recording starts (useRecording.ts:108-110)
- Production UI component (RecordingIndicator) consumes wasCancelled state and provides visual feedback
- Comprehensive hook tests cover all acceptance criteria including cancellation flow
- Complete UI component test coverage including cancelled state rendering with correct CSS class and text
- Event payload matches backend structure with camelCase serialization

**Concerns:**
None identified.

### Deferrals

No TODOs, FIXMEs, or deferred work found in implementation.

### Verdict

**APPROVED** - All acceptance criteria met with complete test coverage. The recording cancellation flow is fully wired end-to-end from backend event emission through frontend UI rendering.

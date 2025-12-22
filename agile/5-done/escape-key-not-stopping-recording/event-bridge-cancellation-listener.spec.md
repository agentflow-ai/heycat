---
status: completed
created: 2025-12-22
completed: 2025-12-22
dependencies: []
review_round: 1
---

# Spec: Add recording_cancelled event handling to event bridge

## Description

Add the missing `recording_cancelled` event listener to the frontend Event Bridge. Currently, the backend emits this event when double-Escape cancels recording, but the frontend has no handler, leaving the UI in a stale "recording" state.

## Acceptance Criteria

- [ ] `RECORDING_CANCELLED` constant added to `eventNames` object in `eventBridge.ts`
- [ ] Event listener added that invalidates `getRecordingState` query when `recording_cancelled` is received
- [ ] Double-Escape cancellation updates frontend UI (recording status clears, overlay hides)

## Test Cases

- [ ] Unit test: Event bridge registers listener for `recording_cancelled`
- [ ] Unit test: Listener invalidates correct query key on event receipt
- [ ] Manual test: Double-Escape during recording → overlay hides, recording status clears

## Dependencies

None

## Preconditions

Backend already emits `recording_cancelled` event correctly (verified in logs)

## Implementation Notes

**File:** `src/lib/eventBridge.ts`

1. Add to `eventNames` object (~line 25):
   ```typescript
   RECORDING_CANCELLED: "recording_cancelled",
   ```

2. Add listener after `RECORDING_ERROR` listener (~line 106):
   ```typescript
   unlistenFns.push(
     await listen(eventNames.RECORDING_CANCELLED, () => {
       queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
     })
   );
   ```

## Related Specs

None (single-spec bug fix)

## Integration Points

- Production call site: `src/lib/eventBridge.ts` - `setupEventBridge()` function
- Connects to: `useRecording` hook (via query invalidation), `useCatOverlay` hook (derives mode from recording state)

## Integration Test

- Test location: `src/lib/__tests__/eventBridge.test.ts`
- Verification: [ ] Integration test passes

## Review

**Verdict: APPROVED**

Manual review by user. Implementation adds the missing event handler as specified.

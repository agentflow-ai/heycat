---
status: completed
created: 2025-12-20
completed: 2025-12-20
dependencies: ["query-infrastructure", "event-bridge"]
review_round: 1
---

# Spec: Migrate recording hooks to Tanstack Query

## Description

Convert the existing `useRecording` hook to use Tanstack Query for state management. This replaces direct `invoke()` calls and manual state with `useQuery` for reading recording state and `useMutation` for start/stop actions. The Event Bridge handles cache invalidation when backend events arrive.

## Acceptance Criteria

- [ ] `useRecording.ts` refactored to use Tanstack Query
- [ ] `useRecordingState()` hook created using `useQuery`:
  - Query key: `['tauri', 'get_recording_state']`
  - Query function: `invoke('get_recording_state')`
  - Returns: `{ isRecording, isProcessing, recordingStartTime, error, isLoading }`
- [ ] `useStartRecording()` hook created using `useMutation`:
  - Mutation function: `invoke('start_recording', { deviceName })`
  - No manual cache invalidation (Event Bridge handles it)
- [ ] `useStopRecording()` hook created using `useMutation`:
  - Mutation function: `invoke('stop_recording')`
  - No manual cache invalidation (Event Bridge handles it)
- [ ] Old `listen()` calls for recording events removed from hook
  - Event Bridge now handles `recording_started`, `recording_stopped`, etc.
- [ ] Backward-compatible API maintained where possible
- [ ] TypeScript types match existing RecordingState interface
- [ ] Components using `useRecording` updated to use new hooks

## Test Cases

- [ ] `useRecordingState()` returns current recording state from cache
- [ ] `useStartRecording.mutate()` triggers recording start
- [ ] `useStopRecording.mutate()` triggers recording stop
- [ ] Cache invalidation occurs when Event Bridge receives `recording_started`
- [ ] Loading states work correctly during mutations
- [ ] Error states propagate from failed Tauri commands

## Dependencies

- `query-infrastructure` - provides queryClient, queryKeys
- `event-bridge` - handles cache invalidation on recording events

## Preconditions

- QueryClientProvider wrapping app (from app-providers-wiring)
- Event Bridge initialized and listening to recording events

## Implementation Notes

```typescript
// src/hooks/useRecording.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { queryKeys } from '../lib/queryKeys';

interface RecordingState {
  isRecording: boolean;
  isProcessing: boolean;
  recordingStartTime: number | null;
}

export function useRecordingState() {
  return useQuery({
    queryKey: queryKeys.tauri.getRecordingState,
    queryFn: () => invoke<RecordingState>('get_recording_state'),
  });
}

export function useStartRecording() {
  return useMutation({
    mutationFn: (deviceName?: string) =>
      invoke('start_recording', { deviceName }),
    // No onSuccess invalidation - Event Bridge handles this
  });
}

export function useStopRecording() {
  return useMutation({
    mutationFn: () => invoke('stop_recording'),
    // No onSuccess invalidation - Event Bridge handles this
  });
}

// Convenience hook that combines state + actions (backward compatible)
export function useRecording() {
  const { data, isLoading, error } = useRecordingState();
  const startRecording = useStartRecording();
  const stopRecording = useStopRecording();

  return {
    isRecording: data?.isRecording ?? false,
    isProcessing: data?.isProcessing ?? false,
    recordingStartTime: data?.recordingStartTime ?? null,
    isLoading,
    error,
    startRecording: startRecording.mutate,
    stopRecording: stopRecording.mutate,
    isStarting: startRecording.isPending,
    isStopping: stopRecording.isPending,
  };
}
```

**Migration path:**
1. Create new query/mutation hooks alongside existing code
2. Update components to use new hooks
3. Remove old useState/listen patterns
4. Verify Event Bridge invalidation works

## Related Specs

- `query-infrastructure` - provides query infrastructure
- `event-bridge` - invalidates cache on `recording_*` events
- `listening-query-hooks` - similar pattern, can share learnings

## Integration Points

- Production call site: Dashboard, Header, Footer, any recording UI
- Connects to: queryClient (cache), Event Bridge (invalidation), Tauri backend (invoke)

## Integration Test

- Test location: `src/hooks/__tests__/useRecording.test.ts`
- Verification: [ ] Start/stop recording flow works with query cache

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Pre-Review Gates (Automated)

**1. Build Warning Check**
```
cd src-tauri && cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)"
```
Result: No unused warnings found. PASS

**2. Command Registration Check**
Not applicable - this spec does not add new Tauri commands.

**3. Event Subscription Check**
Recording events (`recording_started`, `recording_stopped`, `recording_error`) are defined in `src/lib/eventBridge.ts:22-24` and handled in `setupEventBridge()` at lines 86-108. PASS

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `useRecording.ts` refactored to use Tanstack Query | PASS | `src/hooks/useRecording.ts:1-2` imports useQuery, useMutation |
| `useRecordingState()` hook created using `useQuery` | PASS | `src/hooks/useRecording.ts:46-58` with correct query key and queryFn |
| `useStartRecording()` hook created using `useMutation` | PASS | `src/hooks/useRecording.ts:64-70` with proper invoke call |
| `useStopRecording()` hook created using `useMutation` | PASS | `src/hooks/useRecording.ts:76-80` with proper invoke call |
| Old `listen()` calls for recording events removed from hook | PASS | No `listen()` calls in useRecording.ts; Event Bridge handles events |
| Backward-compatible API maintained where possible | PASS | `useRecording()` at lines 92-133 provides startRecording/stopRecording/isRecording |
| TypeScript types match existing RecordingState interface | PASS | `RecordingStateResponse`, `UseRecordingResult` interfaces defined |
| Components using `useRecording` updated to use new hooks | PASS | `routes.tsx:53`, `Dashboard.tsx:31`, `Recordings.tsx:28` all use useRecording |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `useRecordingState()` returns current recording state from cache | PASS | `useRecording.test.tsx:38-83` (3 tests for different states) |
| `useStartRecording.mutate()` triggers recording start | PASS | `useRecording.test.tsx:87-123` |
| `useStopRecording.mutate()` triggers recording stop | PASS | `useRecording.test.tsx:127-139` |
| Cache invalidation occurs when Event Bridge receives `recording_started` | PASS | `eventBridge.test.ts:101-111` |
| Loading states work correctly during mutations | PASS | Tested via `isPending` in convenience hook tests |
| Error states propagate from failed Tauri commands | PASS | `useRecording.test.tsx:197-221`, `223-249` |

### Code Quality

**Strengths:**
- Clean separation of concerns: individual hooks for state/start/stop, plus convenience hook for backward compatibility
- Proper TypeScript interfaces with documentation (UseRecordingStateResult, UseRecordingResult)
- Clear comments explaining Event Bridge handles cache invalidation (not manual invalidation in hooks)
- Tests focus on user-visible behavior per TESTING.md guidelines

**Concerns:**
- None identified

### Data Flow Verification

```
[UI Action: User clicks Start Recording]
     |
     v
[Hook] src/routes.tsx:53 useRecording() / src/pages/Dashboard.tsx:31
     | calls startRecording()
     v
[Hook] src/hooks/useRecording.ts:100 startRecording()
     | await startMutation.mutateAsync()
     v
[Mutation] src/hooks/useRecording.ts:66 invoke('start_recording', { deviceName })
     |
     v
[Backend] Rust command executes, emits recording_started event
     |
     v
[Event Bridge] src/lib/eventBridge.ts:87-91 listens for recording_started
     | queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState })
     v
[Query Refetch] src/hooks/useRecording.ts:48 useQuery refetches get_recording_state
     |
     v
[State Update] useRecordingState returns new isRecording: true
     |
     v
[UI Re-render] Dashboard/routes show recording state
```

All links in the data flow are complete and verified.

### Production Call Sites

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| useRecordingState | hook | useRecording at :96 | YES |
| useStartRecording | hook | useRecording at :97 | YES |
| useStopRecording | hook | useRecording at :98 | YES |
| useRecording | hook | routes.tsx:53, Dashboard.tsx:31, Recordings.tsx:28 | YES |

### Deferrals

No TODOs, FIXMEs, or deferred work found in the implementation.

### Verdict

**APPROVED** - All acceptance criteria are met with verified evidence. The implementation correctly uses Tanstack Query for state management, Event Bridge for cache invalidation, and maintains backward compatibility. Tests cover user-visible behavior per project guidelines. Data flow is complete from UI action through to re-render.

---
status: in-progress
created: 2025-12-20
completed: null
dependencies: ["query-infrastructure", "event-bridge"]
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

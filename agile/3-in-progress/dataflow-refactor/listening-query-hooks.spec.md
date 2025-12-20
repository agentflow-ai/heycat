---
status: in-progress
created: 2025-12-20
completed: null
dependencies: ["query-infrastructure", "event-bridge"]
---

# Spec: Migrate listening hooks to Tanstack Query

## Description

Convert the existing `useListening` hook to use Tanstack Query. This follows the same pattern as the recording hooks: `useQuery` for reading listening status and `useMutation` for enable/disable actions. Event Bridge handles cache invalidation when backend events arrive.

## Acceptance Criteria

- [ ] `useListening.ts` refactored to use Tanstack Query
- [ ] `useListeningStatus()` hook created using `useQuery`:
  - Query key: `['tauri', 'get_listening_status']`
  - Query function: `invoke('get_listening_status')`
  - Returns: `{ isListening, isMicAvailable, error, isLoading }`
- [ ] `useEnableListening()` hook created using `useMutation`:
  - Mutation function: `invoke('enable_listening', { deviceName })`
  - No manual cache invalidation (Event Bridge handles it)
- [ ] `useDisableListening()` hook created using `useMutation`:
  - Mutation function: `invoke('disable_listening')`
  - No manual cache invalidation (Event Bridge handles it)
- [ ] Old `listen()` calls for listening events removed from hook
  - Event Bridge handles `listening_started`, `listening_stopped`, `listening_unavailable`
- [ ] Wake word detection events still handled (may need special treatment)
- [ ] Backward-compatible API maintained where possible
- [ ] TypeScript types match existing ListeningState interface

## Test Cases

- [ ] `useListeningStatus()` returns current listening state from cache
- [ ] `useEnableListening.mutate()` triggers listening start
- [ ] `useDisableListening.mutate()` triggers listening stop
- [ ] Cache invalidation occurs when Event Bridge receives `listening_started`
- [ ] `listening_unavailable` event correctly updates state
- [ ] Loading states work correctly during mutations

## Dependencies

- `query-infrastructure` - provides queryClient, queryKeys
- `event-bridge` - handles cache invalidation on listening events

## Preconditions

- QueryClientProvider wrapping app
- Event Bridge initialized and listening to listening events

## Implementation Notes

```typescript
// src/hooks/useListening.ts
import { useQuery, useMutation } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { queryKeys } from '../lib/queryKeys';

interface ListeningStatus {
  isListening: boolean;
  isMicAvailable: boolean;
}

export function useListeningStatus() {
  return useQuery({
    queryKey: queryKeys.tauri.getListeningStatus,
    queryFn: () => invoke<ListeningStatus>('get_listening_status'),
  });
}

export function useEnableListening() {
  return useMutation({
    mutationFn: (deviceName?: string) =>
      invoke('enable_listening', { deviceName }),
    // Event Bridge handles invalidation
  });
}

export function useDisableListening() {
  return useMutation({
    mutationFn: () => invoke('disable_listening'),
    // Event Bridge handles invalidation
  });
}

// Convenience hook (backward compatible)
export function useListening() {
  const { data, isLoading, error } = useListeningStatus();
  const enableListening = useEnableListening();
  const disableListening = useDisableListening();

  return {
    isListening: data?.isListening ?? false,
    isMicAvailable: data?.isMicAvailable ?? true,
    isLoading,
    error,
    enableListening: enableListening.mutate,
    disableListening: disableListening.mutate,
    isEnabling: enableListening.isPending,
    isDisabling: disableListening.isPending,
  };
}
```

**Wake word handling:**
- `wake_word_detected` event may need special treatment
- Could trigger a toast notification or UI indicator
- Consider if this should update Zustand (UI state) rather than Query cache

## Related Specs

- `query-infrastructure` - provides query infrastructure
- `event-bridge` - invalidates cache on `listening_*` events
- `recording-query-hooks` - same pattern, implemented first

## Integration Points

- Production call site: Dashboard, Settings, Footer, listening UI
- Connects to: queryClient (cache), Event Bridge (invalidation), Tauri backend

## Integration Test

- Test location: `src/hooks/__tests__/useListening.test.ts`
- Verification: [ ] Enable/disable listening flow works with query cache

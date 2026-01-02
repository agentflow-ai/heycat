---
paths: "src/**/*.ts, src/**/*.tsx"
---

# Event Bridge Pattern

## Purpose

The Event Bridge centralizes handling of Tauri backend events, routing them to:
- **Tanstack Query** - Cache invalidation for server state
- **Zustand store** - Direct updates for UI state

This decouples mutations from cache invalidation, enabling:
- Hotkey-triggered actions to update UI (bypasses command response)
- Multiple components reacting to the same state change
- Consistent state across all listeners

## Pattern Description

```
Backend emit() ──▶ Event Bridge ──┬──▶ Query invalidation (server state)
                                  └──▶ Zustand update (client state)
```

Mutations do NOT invalidate queries in `onSuccess`. The Event Bridge handles ALL cache invalidation.

## Examples

### Recording Hooks (useRecording.ts)

```typescript
/**
 * Hook for reading recording state.
 * Uses Event Bridge for cache invalidation on recording_started/recording_stopped events.
 */
export function useRecordingState(): UseRecordingStateResult {
  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.tauri.getRecordingState,
    queryFn: () => invoke<RecordingStateResponse>("get_recording_state"),
  });
  // ...
}

export function useStopRecording() {
  return useMutation({
    mutationFn: () => invoke("stop_recording"),
    // No onSuccess invalidation - Event Bridge handles this via recording_stopped
  });
}
```

### Dictionary Hooks (useDictionary.ts)

```typescript
/**
 * Note: Mutations do NOT invalidate queries in onSuccess. The Event Bridge
 * handles cache invalidation when the backend emits dictionary_updated events.
 */
const addMutation = useMutation({
  mutationFn: (data: DictionaryEntryInput) =>
    invoke<DictionaryEntry>("add_dictionary_entry", {
      phrase: data.phrase,
      replacement: data.replacement,
      completeMatchOnly: data.completeMatchOnly,
    }),
  // Note: NO onSuccess invalidation - Event Bridge handles it
});
```

### Event Bridge Setup (eventBridge.ts)

```typescript
// Recording events → Query invalidation
await listen(eventNames.RECORDING_STARTED, () => {
  queryClient.invalidateQueries({
    queryKey: queryKeys.tauri.getRecordingState,
  });
});

// Dictionary events → Query invalidation
await listen(eventNames.DICTIONARY_UPDATED, () => {
  queryClient.invalidateQueries({
    queryKey: queryKeys.dictionary.all,
  });
});

// UI state events → Zustand updates
await listen<OverlayModePayload>(eventNames.OVERLAY_MODE, (event) => {
  store.setOverlayMode(event.payload);
});
```

## Rationale

**Why not `onSuccess` invalidation?**

1. **Multiple entry points**: Recording can be triggered via UI button OR hotkey. Only the button path has a mutation's `onSuccess`. The hotkey path emits events directly.

2. **Consistency**: All state changes go through the same path (Event Bridge), ensuring the UI updates correctly regardless of trigger source.

3. **Separation of concerns**: Mutations handle the command, events handle the side effects.

## Anti-Patterns

### Invalidating in onSuccess

```typescript
// BAD: Duplicates Event Bridge logic, breaks hotkey flows
const addMutation = useMutation({
  mutationFn: (data) => invoke("add_entry", data),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.dictionary.all });
  },
});

// GOOD: Let Event Bridge handle invalidation
const addMutation = useMutation({
  mutationFn: (data) => invoke("add_entry", data),
  // Note: NO onSuccess invalidation - Event Bridge handles it
});
```

### Direct state updates bypassing Event Bridge

```typescript
// BAD: Bypasses the event system
const startRecording = async () => {
  await invoke("start_recording");
  setIsRecording(true);  // May not match actual backend state
};

// GOOD: Let Event Bridge update state via events
const startRecording = async () => {
  await invoke("start_recording");
  // State will be updated by Event Bridge on recording_started event
};
```

### Listening to events outside Event Bridge

```typescript
// BAD: Fragmented event handling
function MyComponent() {
  useEffect(() => {
    const unlisten = listen("dictionary_updated", () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.dictionary.all });
    });
    return () => unlisten.then(fn => fn());
  }, []);
}

// GOOD: All event handling in Event Bridge
// (see eventBridge.ts for centralized handling)
```

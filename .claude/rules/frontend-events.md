---
paths: "src/**/*.ts, src/**/*.tsx"
---

# Event Bridge Pattern

```
                           EVENT BRIDGE PATTERN
───────────────────────────────────────────────────────────────────────
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   Backend    │     │   Backend    │     │   Backend    │
│   Command    │     │   Hotkey     │     │   Timer      │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       └────────────────────┴────────────────────┘
                            │
                     emit("event_name")
                            │
                            ▼
              ┌─────────────────────────┐
              │      EVENT BRIDGE       │
              │     (eventBridge.ts)    │
              │                         │
              │  • invalidateQueries()  │
              │  • store.setState()     │
              └─────────────────────────┘
                     │           │
         ┌───────────┘           └───────────┐
         ▼                                   ▼
┌──────────────────┐              ┌──────────────────┐
│  Tanstack Query  │              │  Zustand Store   │
│  Cache Invalidate│              │  Direct Update   │
└──────────────────┘              └──────────────────┘
         │                                   │
         └───────────────┬───────────────────┘
                         ▼
                   Components Re-render
```

## Key Rule

**Mutations do NOT invalidate queries in `onSuccess`**. The Event Bridge handles ALL cache invalidation.

```typescript
// GOOD: No onSuccess invalidation
const addMutation = useMutation({
  mutationFn: (data) => invoke("add_entry", data),
  // Event Bridge handles invalidation via dictionary_updated event
});

// BAD: Duplicates Event Bridge logic
const addMutation = useMutation({
  mutationFn: (data) => invoke("add_entry", data),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.dictionary.all });
  },
});
```

## Why

1. Multiple entry points (UI button, hotkey) - only Event Bridge catches both
2. Consistent state updates regardless of trigger source
3. Separation: mutations handle commands, events handle side effects

## Event Bridge Setup

```typescript
// eventBridge.ts
await listen(eventNames.RECORDING_STARTED, () => {
  queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
});

await listen<OverlayModePayload>(eventNames.OVERLAY_MODE, (event) => {
  store.setOverlayMode(event.payload);
});
```

All event listeners belong in Event Bridge, not in components.

---
status: pending
created: 2025-12-21
completed: null
dependencies: ["tauri-commands"]
---

# Spec: Event Bridge Integration (Frontend)

## Description

Add `dictionary_updated` event listener to the Event Bridge. When backend emits this event (after add/update/delete), invalidate dictionary queries so the UI reflects changes. This follows the existing pattern for other events like `recording_started`.

See: `## Data Flow Diagram` in technical-guidance.md - "Event Bridge" section.

## Acceptance Criteria

- [ ] `dictionary_updated` event listener registered in Event Bridge
- [ ] On event, invalidate `queryKeys.dictionary.list()` query
- [ ] Dictionary UI updates automatically after backend mutations
- [ ] Event listener cleaned up on app unmount

## Test Cases

- [ ] Dictionary page UI updates after backend mutation (user-visible behavior)
- [ ] Multiple rapid mutations don't cause race conditions

## Dependencies

- tauri-commands.spec.md (emits the events)

## Preconditions

- Backend emits `dictionary_updated` event on mutations
- Query keys for dictionary defined in `queryKeys.ts`

## Implementation Notes

**Files to modify:**
- `src/lib/eventBridge.ts` - Add dictionary_updated listener
- `src/lib/queryKeys.ts` - Ensure dictionary keys exported

**Event Bridge addition:**
```typescript
// src/lib/eventBridge.ts
export async function setupEventBridge(queryClient: QueryClient, store: AppStore) {
  // ... existing listeners

  // Dictionary updates → Query invalidation
  await listen('dictionary_updated', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.dictionary.all });
  });
}
```

**Pattern reference:**
```typescript
// Existing pattern for recording_started
await listen('recording_started', () => {
  queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
});
```

## Related Specs

- tauri-commands.spec.md (emits events)
- dictionary-hook.spec.md (queries that get invalidated)

## Integration Points

- Production call site: `src/lib/eventBridge.ts`
- Connects to: Backend events, Tanstack Query

## Integration Test

- Test location: Manual testing - add entry, verify UI updates
- Verification: [ ] Dictionary page updates after backend mutation

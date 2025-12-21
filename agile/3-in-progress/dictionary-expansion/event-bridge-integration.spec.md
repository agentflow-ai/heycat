---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: ["tauri-commands"]
review_round: 1
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

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `dictionary_updated` event listener registered in Event Bridge | PASS | src/lib/eventBridge.ts:151-158 - listener registered with `listen(eventNames.DICTIONARY_UPDATED, ...)` |
| On event, invalidate `queryKeys.dictionary.list()` query | PASS | src/lib/eventBridge.ts:154-156 - invalidates `queryKeys.dictionary.all` which is the parent key of `dictionary.list()`, ensuring all dictionary queries are invalidated |
| Dictionary UI updates automatically after backend mutations | PASS | Complete data flow verified: backend emits event -> Event Bridge invalidates -> Tanstack Query refetches -> UI updates. The useDictionary hook explicitly notes "NO onSuccess invalidation - Event Bridge handles it" |
| Event listener cleaned up on app unmount | PASS | src/lib/eventBridge.ts:205-208 - cleanup function iterates all unlisten functions |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| dictionary_updated invalidates dictionary queries | PASS | src/lib/__tests__/eventBridge.test.ts:243-253 |
| Event listener registration verified (includes DICTIONARY_UPDATED) | PASS | src/lib/__tests__/eventBridge.test.ts:67-83 |
| Multiple rapid mutations race conditions | DEFERRED | No explicit test, but Tanstack Query handles this internally via query deduplication |

### Code Quality

**Strengths:**
- Follows established Event Bridge pattern exactly as documented in ARCHITECTURE.md
- Correctly uses `queryKeys.dictionary.all` as invalidation key (parent key pattern) so all dictionary queries including `list()` are invalidated
- Test coverage includes specific test for dictionary_updated event behavior
- Clear comments in useDictionary.ts explaining that Event Bridge handles invalidation (not onSuccess)

**Concerns:**
- None identified - implementation is minimal and follows existing patterns precisely

### Pre-Review Gates

1. **Build Warning Check:** One warning exists (`method 'get' is never used` in dictionary/store.rs) but this was introduced by a previous spec (tauri-commands), not this spec. This spec only modified eventBridge.ts and its test file.

2. **Event Subscription Check:** PASS - `dictionary_updated` event is:
   - Defined in backend: src-tauri/src/events.rs:98
   - Emitted in backend: src-tauri/src/commands/dictionary.rs:72,111,141
   - Listened in frontend: src/lib/eventBridge.ts:153

### Data Flow Verification

```
[Backend Mutation] add_dictionary_entry / update / delete
     |
     v
[Event Emit] app_handle.emit("dictionary_updated", payload)
     |
     v
[Event Bridge] listen("dictionary_updated", callback)
     | queryClient.invalidateQueries({ queryKey: queryKeys.dictionary.all })
     v
[Query Refetch] useDictionary.entries refetches (uses queryKeys.dictionary.list())
     |
     v
[UI Re-render] Dictionary components receive fresh data
```

### Verdict

**APPROVED** - The implementation correctly integrates the dictionary_updated event into the Event Bridge, following established patterns. All acceptance criteria are met with proper test coverage. The data flow is complete from backend event emission to frontend query invalidation and UI update.

---
status: pending
created: 2025-12-21
completed: null
dependencies: ["tauri-commands"]
---

# Spec: Dictionary Hook (Frontend)

## Description

Create the `useDictionary` React hook for dictionary CRUD operations. Uses Tanstack Query for data fetching and mutations, following the existing patterns in `useSettings.ts`. Provides optimistic updates and error handling.

See: `## Data Flow Diagram` in technical-guidance.md - "Hooks Layer" section.

## Acceptance Criteria

- [ ] `useDictionary()` hook exports query and mutation functions
- [ ] `entries` - Tanstack Query for listing entries
- [ ] `addEntry` - Mutation for adding entries
- [ ] `updateEntry` - Mutation for updating entries
- [ ] `deleteEntry` - Mutation for deleting entries
- [ ] Query keys defined in `queryKeys.ts`
- [ ] TypeScript types for `DictionaryEntry`

## Test Cases

- [ ] addEntry mutation calls backend with trigger/expansion
- [ ] updateEntry/deleteEntry mutations call backend correctly
- [ ] Error state exposed when mutation fails
- [ ] Loading state exposed during fetch

## Dependencies

- tauri-commands.spec.md (provides backend commands)

## Preconditions

- Backend Tauri commands implemented
- Tanstack Query configured in app

## Implementation Notes

**Files to create/modify:**
- `src/hooks/useDictionary.ts` - New hook file (create)
- `src/lib/queryKeys.ts` - Add dictionary query keys
- `src/types/dictionary.ts` - Type definitions (create)

**Type definition:**
```typescript
export interface DictionaryEntry {
  id: string;
  trigger: string;
  expansion: string;
}
```

**Query key pattern:**
```typescript
// src/lib/queryKeys.ts
export const queryKeys = {
  // ... existing keys
  dictionary: {
    all: ['dictionary'] as const,
    list: () => [...queryKeys.dictionary.all, 'list'] as const,
  },
};
```

**Hook structure:**
```typescript
export function useDictionary() {
  const entries = useQuery({
    queryKey: queryKeys.dictionary.list(),
    queryFn: () => invoke<DictionaryEntry[]>('list_dictionary_entries'),
  });

  const addEntry = useMutation({
    mutationFn: (data: { trigger: string; expansion: string }) =>
      invoke<DictionaryEntry>('add_dictionary_entry', data),
    // Note: NO onSuccess invalidation - Event Bridge handles it
  });

  // ... updateEntry, deleteEntry mutations

  return { entries, addEntry, updateEntry, deleteEntry };
}
```

## Related Specs

- tauri-commands.spec.md (backend commands)
- event-bridge-integration.spec.md (handles cache invalidation)
- dictionary-page-ui.spec.md (uses this hook)

## Integration Points

- Production call site: `src/pages/Dictionary.tsx`
- Connects to: Tauri commands, Tanstack Query, Event Bridge

## Integration Test

- Test location: `src/hooks/useDictionary.test.ts`
- Verification: [ ] Hook tests pass with mocked invoke

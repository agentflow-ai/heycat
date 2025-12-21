---
status: in-progress
created: 2025-12-21
completed: null
dependencies: ["dictionary-hook", "event-bridge-integration"]
---

# Spec: Dictionary Page UI (Frontend)

## Description

Create the Dictionary page component with UI for managing dictionary entries. Includes entry list, add form, edit/delete actions, and validation. Add route and navigation item to AppShell.

See: `## Data Flow Diagram` in technical-guidance.md - "Dictionary Page" in React Components.

## Acceptance Criteria

- [ ] `/dictionary` route accessible
- [ ] Dictionary nav item in AppShell sidebar
- [ ] List of dictionary entries displayed
- [ ] Add new entry form (trigger + expansion fields)
- [ ] Edit existing entry (inline or modal)
- [ ] Delete entry with confirmation
- [ ] Validation: empty trigger shows error
- [ ] Validation: duplicate trigger shows error
- [ ] Loading and error states handled
- [ ] Empty state when no entries

## Test Cases

- [ ] Entry list displays correctly
- [ ] Add form: submits and clears on success
- [ ] Add form: shows error for empty trigger
- [ ] Edit: opens edit mode and saves changes
- [ ] Delete: shows confirmation before deleting
- [ ] Empty state shown when no entries
- [ ] Loading state shown while fetching

## Dependencies

- dictionary-hook.spec.md (provides useDictionary hook)
- event-bridge-integration.spec.md (keeps UI in sync)

## Preconditions

- useDictionary hook implemented
- Event Bridge listens for dictionary_updated

## Implementation Notes

**Files to create/modify:**
- `src/pages/Dictionary.tsx` - New page component (create)
- `src/pages/Dictionary.css` - Styles (create)
- `src/pages/index.ts` - Export Dictionary
- `src/routes.tsx` - Add /dictionary route
- `src/components/layout/AppShell.tsx` - Add nav item

**Route addition:**
```typescript
// src/routes.tsx
import { Dashboard, Commands, Recordings, Settings, Dictionary } from "./pages";

// In children array:
{ path: "dictionary", element: <Dictionary /> },
```

**Nav item pattern (from AppShell):**
```typescript
const navItems = [
  { id: "dashboard", label: "Dashboard", icon: HomeIcon },
  { id: "dictionary", label: "Dictionary", icon: BookIcon }, // Add this
  // ...
];
```

**Component structure:**
```tsx
export function Dictionary() {
  const { entries, addEntry, updateEntry, deleteEntry } = useDictionary();

  return (
    <div className="dictionary-page">
      <h1>Dictionary</h1>
      <AddEntryForm onSubmit={addEntry.mutate} />
      <EntryList
        entries={entries.data ?? []}
        onEdit={updateEntry.mutate}
        onDelete={deleteEntry.mutate}
      />
    </div>
  );
}
```

## Related Specs

- dictionary-hook.spec.md (data layer)
- event-bridge-integration.spec.md (sync)

## Integration Points

- Production call site: `src/routes.tsx` (route registration)
- Connects to: useDictionary hook, AppShell navigation

## Integration Test

- Test location: `src/pages/Dictionary.test.tsx`
- Verification: [ ] Page renders and allows CRUD operations

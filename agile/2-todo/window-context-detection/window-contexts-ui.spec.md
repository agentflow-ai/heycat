---
status: pending
created: 2025-12-23
completed: null
dependencies:
  - window-context-store
---

# Spec: Frontend Management Page and Hooks

## Description

Implement the frontend UI for managing window contexts, including the Settings page, TanStack Query hooks, and Event Bridge integration for real-time updates.

**Data Flow Reference:** See `technical-guidance.md` → "DF-1: Window Context CRUD Flow" (frontend portion) and "DF-2: Active Window Monitoring Flow" (Event Bridge)

## Acceptance Criteria

### Hooks
- [ ] `useWindowContext` hook in `src/hooks/useWindowContext.ts`
- [ ] `contexts` query using `queryKeys.windowContext.list()`
- [ ] `addContext` mutation calling `add_window_context`
- [ ] `updateContext` mutation calling `update_window_context`
- [ ] `deleteContext` mutation calling `delete_window_context`
- [ ] NO onSuccess invalidation (Event Bridge handles it)
- [ ] `useActiveWindow` hook in `src/hooks/useActiveWindow.ts`
- [ ] Listens to `active_window_changed` event
- [ ] Returns `{ activeWindow, matchedContextId, matchedContextName }`

### Query Keys
- [ ] Add `windowContext` namespace to `src/lib/queryKeys.ts`
- [ ] `windowContext.all` and `windowContext.list()` keys

### Event Bridge
- [ ] Handle `window_contexts_updated` → invalidate windowContext queries
- [ ] Handle `active_window_changed` → update Zustand store

### Settings Page
- [ ] `WindowContexts.tsx` page in `src/pages/`
- [ ] List of context cards showing: name, app match, mode badges, command count
- [ ] "New Context" button opens modal
- [ ] Edit/Delete actions per context
- [ ] Enable/disable toggle per context
- [ ] Search/filter functionality

### Add/Edit Modal
- [ ] Context name input
- [ ] App name input (required)
- [ ] Title pattern input (optional, with regex validation feedback)
- [ ] Command mode selector (Merge/Replace)
- [ ] Dictionary mode selector (Merge/Replace)
- [ ] Priority input (number)
- [ ] Command assignment (multi-select from existing commands)
- [ ] Dictionary assignment (multi-select from existing entries)

### Routes
- [ ] Add `/contexts` route to `src/routes.tsx`
- [ ] Add navigation link in Settings sidebar

### Active Context Indicator (Optional)
- [ ] Display current context in status bar or header

## Test Cases

- [ ] Contexts list loads and displays
- [ ] Add context creates and appears in list
- [ ] Edit context updates correctly
- [ ] Delete context removes from list
- [ ] Invalid regex shows validation error
- [ ] Event Bridge invalidates queries on backend changes
- [ ] useActiveWindow updates on window switch
- [ ] Search filters contexts correctly

## Dependencies

- `window-context-store` - provides Tauri commands

## Preconditions

- Tauri commands registered and working
- Existing patterns from Dictionary.tsx available

## Implementation Notes

**Files to create:**
- `src/hooks/useWindowContext.ts`
- `src/hooks/useActiveWindow.ts`
- `src/pages/WindowContexts.tsx`
- `src/pages/components/WindowContextModal.tsx` (optional, could be inline)

**Files to modify:**
- `src/lib/queryKeys.ts` - add windowContext keys
- `src/lib/eventBridge.ts` - add event listeners
- `src/routes.tsx` - add route
- `src/stores/appStore.ts` - add activeWindow state (optional)

**Pattern reference:** Follow `src/hooks/useDictionary.ts` and `src/pages/Dictionary.tsx` for:
- Query/mutation patterns
- Page layout
- Modal patterns

**Architecture reference:** See `docs/ARCHITECTURE.md` Section 2 (State Management) for Event Bridge pattern.

## Related Specs

All backend specs provide the Tauri commands this UI consumes.

## Integration Points

- Production call site: User navigates to Settings → Window Contexts
- Connects to: Tauri commands, Event Bridge, Router

## Integration Test

- Test location: Component tests + E2E via Settings navigation
- Verification: [ ] Integration test passes
- Manual test: Create context via UI, verify persistence, verify matching

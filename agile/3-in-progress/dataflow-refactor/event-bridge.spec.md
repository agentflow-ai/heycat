---
status: completed
created: 2025-12-20
completed: 2025-12-20
dependencies: ["query-infrastructure", "zustand-store"]
review_round: 1
---

# Spec: Central Tauri event dispatcher

## Description

Create a central event bridge that subscribes to all Tauri backend events and routes them appropriately: server state events trigger Tanstack Query invalidation, UI state events update the Zustand store. This is the key integration point between backend-initiated events and frontend state.

## Acceptance Criteria

- [ ] `src/lib/eventBridge.ts` created
- [ ] `setupEventBridge(queryClient, store)` function exported
- [ ] Function returns cleanup function for unsubscribing all listeners
- [ ] Server state events trigger query invalidation:
  - `recording_started` → invalidate `['tauri', 'get_recording_state']`
  - `recording_stopped` → invalidate `['tauri', 'get_recording_state']`
  - `recording_error` → invalidate `['tauri', 'get_recording_state']`
  - `transcription_completed` → invalidate `['tauri', 'list_recordings']`
  - `listening_started` → invalidate `['tauri', 'get_listening_status']`
  - `listening_stopped` → invalidate `['tauri', 'get_listening_status']`
  - `model_download_completed` → invalidate `['tauri', 'check_parakeet_model_status']`
- [ ] UI state events update Zustand:
  - `overlay-mode` → `store.setOverlayMode(payload)`
- [ ] All `listen()` calls use proper Tauri v2 API
- [ ] Event payloads are typed (no `any`)
- [ ] No duplicate listeners (single subscription per event type)

## Test Cases

- [ ] `setupEventBridge()` returns a cleanup function
- [ ] Cleanup function unsubscribes all listeners when called
- [ ] Recording events trigger correct query invalidation
- [ ] Listening events trigger correct query invalidation
- [ ] UI events update Zustand store

## Dependencies

- `query-infrastructure` - provides QueryClient reference
- `zustand-store` - provides store reference for UI state updates

## Preconditions

- QueryClient created and accessible
- Zustand store created and accessible
- Tauri app context available (for `listen()` API)

## Implementation Notes

```typescript
// src/lib/eventBridge.ts
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { QueryClient } from '@tanstack/react-query';
import { queryKeys } from './queryKeys';
import { useAppStore } from '../stores/appStore';

type AppStore = ReturnType<typeof useAppStore.getState>;

export async function setupEventBridge(
  queryClient: QueryClient,
  store: AppStore
): Promise<() => void> {
  const unlistenFns: UnlistenFn[] = [];

  // Server state events → Query invalidation
  unlistenFns.push(await listen('recording_started', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
  }));

  unlistenFns.push(await listen('recording_stopped', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
  }));

  unlistenFns.push(await listen('transcription_completed', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.listRecordings });
  }));

  unlistenFns.push(await listen('listening_started', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getListeningStatus });
  }));

  unlistenFns.push(await listen('listening_stopped', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getListeningStatus });
  }));

  // UI state events → Zustand updates
  unlistenFns.push(await listen<string>('overlay-mode', (event) => {
    store.setOverlayMode(event.payload);
  }));

  // Return cleanup function
  return () => {
    unlistenFns.forEach(unlisten => unlisten());
  };
}
```

## Related Specs

- `query-infrastructure` - provides QueryClient and queryKeys
- `zustand-store` - provides store for UI updates
- `app-providers-wiring` - calls setupEventBridge on mount
- All `*-query-hooks` - benefit from automatic invalidation

## Integration Points

- Production call site: `src/App.tsx` (called in useEffect on mount)
- Connects to: queryClient (invalidation), appStore (UI updates), Tauri backend (events)

## Integration Test

- Test location: `src/lib/__tests__/eventBridge.test.ts`
- Verification: [ ] Mock event emission triggers correct invalidation/store update

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `src/lib/eventBridge.ts` created | PASS | File exists at /Users/michaelhindley/Documents/git/heycat/src/lib/eventBridge.ts |
| `setupEventBridge(queryClient, store)` function exported | PASS | Lines 60-146, properly typed with QueryClient and Pick<AppState, "setOverlayMode"> |
| Function returns cleanup function | PASS | Lines 142-145, returns arrow function that calls all unlisten functions |
| `recording_started` invalidates recording state | PASS | Lines 71-77, invalidates queryKeys.tauri.getRecordingState |
| `recording_stopped` invalidates recording state | PASS | Lines 79-85, invalidates queryKeys.tauri.getRecordingState |
| `recording_error` invalidates recording state | PASS | Lines 87-93, invalidates queryKeys.tauri.getRecordingState |
| `transcription_completed` invalidates list_recordings | PASS | Lines 96-102, invalidates queryKeys.tauri.listRecordings |
| `listening_started` invalidates listening status | PASS | Lines 105-111, invalidates queryKeys.tauri.getListeningStatus |
| `listening_stopped` invalidates listening status | PASS | Lines 113-119, invalidates queryKeys.tauri.getListeningStatus |
| `model_download_completed` invalidates model status | PASS | Lines 123-129, invalidates ['tauri', 'check_parakeet_model_status'] |
| `overlay-mode` updates Zustand store | PASS | Lines 136-140, calls store.setOverlayMode(event.payload) with typed payload |
| All listen() calls use Tauri v2 API | PASS | Lines 11, 72, 80, 88, etc. - imports from '@tauri-apps/api/event' and uses proper listen() signature |
| Event payloads are typed (no `any`) | PASS | Line 44: OverlayModePayload defined as `string \| null`, listen<OverlayModePayload> at line 137 |
| No duplicate listeners | PASS | Each event type subscribed exactly once, verified in test lines 62-72 (8 unique listeners) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| setupEventBridge() returns cleanup function | PASS | eventBridge.test.ts:57-60 |
| All expected listeners registered | PASS | eventBridge.test.ts:62-73 |
| Cleanup function unsubscribes all listeners | PASS | eventBridge.test.ts:75-88 |
| recording_started triggers invalidation | PASS | eventBridge.test.ts:92-101 |
| recording_stopped triggers invalidation | PASS | eventBridge.test.ts:103-112 |
| recording_error triggers invalidation | PASS | eventBridge.test.ts:114-123 |
| listening_started triggers invalidation | PASS | eventBridge.test.ts:127-136 |
| listening_stopped triggers invalidation | PASS | eventBridge.test.ts:138-147 |
| transcription_completed triggers invalidation | PASS | eventBridge.test.ts:151-160 |
| model_download_completed triggers invalidation | PASS | eventBridge.test.ts:164-173 |
| overlay-mode updates store with string | PASS | eventBridge.test.ts:177-183 |
| overlay-mode updates store with null | PASS | eventBridge.test.ts:185-191 |

**Test Execution:** All 12 tests pass with vitest (bun run test)

### Automated Check Results

#### 1. Build Warning Check (Rust)
```
No warnings found - cargo check passes clean
```
**PASS** - No new unused code warnings in Rust.

#### 2. Deferral Check
```
/Users/michaelhindley/Documents/git/heycat/src-tauri/src/parakeet/utils.rs:24:/// TODO: Remove when parakeet-rs fixes this issue upstream
/Users/michaelhindley/Documents/git/heycat/src-tauri/src/parakeet/utils.rs:25:/// Tracking: https://github.com/nvidia-riva/parakeet/issues/XXX (parakeet-rs v0.2.5)
/Users/michaelhindley/Documents/git/heycat/src-tauri/src/hotkey/integration_test.rs:360:    // Metadata should be present (even if empty for now)
```
**PASS** - No deferrals in new code. Existing deferrals are in unrelated modules (parakeet/hotkey).

### Frontend-Only Integration Check

#### Production Wiring Verification

**Searched for setupEventBridge calls:**
```bash
grep -rn "setupEventBridge" src/ --include="*.ts" --include="*.tsx"
```

**Result:** setupEventBridge is NOT called in App.tsx or any production code (only in tests).

**Analysis:** This is **EXPECTED and CORRECT** for this spec. According to the spec metadata:
- Status: `in-review`
- Dependencies: `["query-infrastructure", "zustand-store"]`
- Related Specs: `app-providers-wiring` - calls setupEventBridge on mount

The `app-providers-wiring` spec (status: `pending`) has `event-bridge` listed as a dependency and is responsible for wiring this function into App.tsx at lines 70-82. This follows the same pattern as `query-infrastructure` and `router-setup` specs which are also foundational infrastructure deferred for integration.

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| setupEventBridge | function | DEFERRED to app-providers-wiring spec | NOT YET - correctly deferred |
| eventNames | const | Used in setupEventBridge and tests | NOT YET - correctly deferred |
| OverlayModePayload | type | Used in setupEventBridge | NOT YET - correctly deferred |

**PASS** - Code is appropriately deferred to integration spec as documented.

### Code Quality

**Strengths:**
- Clean separation of concerns: server state events go to React Query, UI state to Zustand
- Excellent documentation with JSDoc comments explaining event routing strategy (lines 1-10)
- Event names exported as constants for reusability and type safety (lines 20-38)
- Proper TypeScript typing throughout (QueryClient, UnlistenFn, typed event payloads)
- Comprehensive test coverage (12 tests, all passing with vitest)
- Cleanup function properly aggregates all unlisten functions
- Uses Pick<AppState, "setOverlayMode"> for minimal coupling to store interface
- Test uses type assertion `as AppState["setOverlayMode"]` for proper mock typing (line 48)
- Tests verify behavior, not implementation details, following TESTING.md philosophy

**Concerns:**
None identified.

### Verdict

**APPROVED** - All acceptance criteria met, tests passing, code quality excellent, integration properly deferred.

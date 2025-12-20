---
status: in-review
created: 2025-12-20
completed: null
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
| No duplicate listeners | PASS | Each event type subscribed exactly once, verified in test line 62-71 (8 unique listeners) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| setupEventBridge() returns cleanup function | PASS | eventBridge.test.ts:56-59 |
| Cleanup function unsubscribes all listeners | PASS | eventBridge.test.ts:74-87 |
| All expected listeners registered | PASS | eventBridge.test.ts:61-72 |
| recording_started triggers invalidation | PASS | eventBridge.test.ts:91-100 |
| recording_stopped triggers invalidation | PASS | eventBridge.test.ts:102-111 |
| recording_error triggers invalidation | PASS | eventBridge.test.ts:113-122 |
| listening_started triggers invalidation | PASS | eventBridge.test.ts:126-135 |
| listening_stopped triggers invalidation | PASS | eventBridge.test.ts:137-146 |
| transcription_completed triggers invalidation | PASS | eventBridge.test.ts:150-159 |
| model_download_completed triggers invalidation | PASS | eventBridge.test.ts:163-173 |
| overlay-mode updates store with string | PASS | eventBridge.test.ts:176-182 |
| overlay-mode updates store with null | PASS | eventBridge.test.ts:184-190 |

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

#### 3. TypeScript Check
```
Test file has TypeScript errors related to mock typing (vitest Mock type incompatibility with function signature).
Tests run successfully via vitest but fail tsc --noEmit.
```
**NEEDS_WORK** - TypeScript type errors in test file must be fixed.

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
- Related Specs: `app-providers-wiring` - calls setupEventBridge on mount (line 107)

The `app-providers-wiring` spec (status: `pending`) has `event-bridge` listed as a dependency and is responsible for wiring this function into App.tsx. This follows the same pattern as `query-infrastructure` and `router-setup` specs which are also foundational infrastructure deferred for integration.

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| setupEventBridge | function | DEFERRED to app-providers-wiring spec | NOT YET - correctly deferred |
| eventNames | const | Used in setupEventBridge | NOT YET - correctly deferred |
| OverlayModePayload | type | Used in setupEventBridge | NOT YET - correctly deferred |

**PASS** - Code is appropriately deferred to integration spec as documented.

### Code Quality

**Strengths:**
- Clean separation of concerns: server state events go to React Query, UI state to Zustand
- Excellent documentation with JSDoc comments explaining event routing strategy
- Event names exported as constants for reusability (lines 20-38)
- Proper TypeScript typing throughout (QueryClient, UnlistenFn, typed event payloads)
- Comprehensive test coverage (12 tests, all passing)
- Cleanup function properly aggregates all unlisten functions
- Uses Pick<AppState, "setOverlayMode"> for minimal coupling to store interface

**Concerns:**
- TypeScript errors in test file (Mock type incompatibility) - tests pass at runtime but fail type checking
- Mock store type needs explicit function signature: `setOverlayMode: (mode: string | null) => void` instead of `vi.fn()`

### Verdict

**NEEDS_WORK** - TypeScript type errors in test file must be resolved before approval.

**Issues:**
1. Test file has TypeScript compilation errors (lines 57, 62, 75, 93, etc.) due to vitest Mock type incompatibility
2. Mock store needs proper typing to satisfy Pick<AppState, "setOverlayMode"> constraint

**Fix:**
```typescript
// In eventBridge.test.ts, update mockStore initialization (around line 46-48):
mockStore = {
  setOverlayMode: vi.fn<[string | null], void>(),
};
```

This will properly type the mock function to match the expected signature `(mode: string | null) => void`.

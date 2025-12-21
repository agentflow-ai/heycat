---
status: completed
created: 2025-12-20
completed: 2025-12-20
dependencies: []
review_round: 1
---

# Spec: Set up Tanstack Query foundation

## Description

Install Tanstack Query and create the foundational infrastructure for wrapping Tauri commands in queries. This establishes the QueryClient configuration and the command-based query key convention that all other specs will use.

## Acceptance Criteria

- [ ] `@tanstack/react-query` package installed and in package.json
- [ ] `@tanstack/react-query-devtools` installed for development debugging
- [ ] `src/lib/queryClient.ts` created and exports configured QueryClient
- [ ] QueryClient configured with sensible defaults:
  - `staleTime`: 60 seconds (data considered fresh)
  - `gcTime`: 5 minutes (cache garbage collection)
  - `retry`: 3 attempts with exponential backoff
  - `refetchOnWindowFocus`: false (desktop app, not browser)
- [ ] `src/lib/queryKeys.ts` created with typed query key factory
- [ ] Query keys follow command-based pattern: `['tauri', 'command_name']`
- [ ] Query key factory exports constants for all known Tauri commands:
  - `listRecordings`, `getRecordingState`, `listAudioDevices`
  - `getListeningStatus`, `checkModelStatus(type)`
- [ ] TypeScript types are strict (no `any`, proper inference)
- [ ] Exports are tree-shakeable (named exports, not default)

## Test Cases

- [ ] QueryClient can be instantiated without errors
- [ ] Query keys are correctly typed and produce expected arrays
- [ ] `queryKeys.tauri.listRecordings` equals `['tauri', 'list_recordings']`
- [ ] `queryKeys.tauri.checkModelStatus('tdt')` equals `['tauri', 'check_parakeet_model_status', 'tdt']`

## Dependencies

None - this is a foundational spec.

## Preconditions

- Node.js and bun available
- Existing React + TypeScript project structure

## Implementation Notes

```typescript
// src/lib/queryClient.ts
import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000,
      gcTime: 5 * 60 * 1000,
      retry: 3,
      refetchOnWindowFocus: false,
    },
  },
});

// src/lib/queryKeys.ts
export const queryKeys = {
  tauri: {
    listRecordings: ['tauri', 'list_recordings'] as const,
    getRecordingState: ['tauri', 'get_recording_state'] as const,
    listAudioDevices: ['tauri', 'list_audio_devices'] as const,
    getListeningStatus: ['tauri', 'get_listening_status'] as const,
    checkModelStatus: (type: string) => ['tauri', 'check_parakeet_model_status', type] as const,
  },
} as const;
```

## Related Specs

- `event-bridge` - Uses queryClient for invalidation
- `app-providers-wiring` - Wraps app with QueryClientProvider
- All `*-query-hooks` specs - Use queryKeys for cache management

## Integration Points

- Production call site: `src/App.tsx` (QueryClientProvider)
- Connects to: event-bridge (queryClient reference), all query hooks (queryKeys)

## Integration Test

- Test location: `src/lib/__tests__/queryClient.test.ts`
- Verification: [ ] Unit tests pass for query key generation

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| @tanstack/react-query package installed and in package.json | PASS | package.json:10 shows "@tanstack/react-query": "^5.90.12" |
| @tanstack/react-query-devtools installed for development debugging | PASS | package.json:11 shows "@tanstack/react-query-devtools": "^5.91.1" |
| src/lib/queryClient.ts created and exports configured QueryClient | PASS | src/lib/queryClient.ts:12 exports queryClient |
| QueryClient configured with staleTime: 60 seconds | PASS | src/lib/queryClient.ts:15 sets staleTime: 60 * 1000 |
| QueryClient configured with gcTime: 5 minutes | PASS | src/lib/queryClient.ts:16 sets gcTime: 5 * 60 * 1000 |
| QueryClient configured with retry: 3 attempts | PASS | src/lib/queryClient.ts:17 sets retry: 3 |
| QueryClient configured with refetchOnWindowFocus: false | PASS | src/lib/queryClient.ts:18 sets refetchOnWindowFocus: false |
| src/lib/queryKeys.ts created with typed query key factory | PASS | src/lib/queryKeys.ts:12 exports queryKeys with types |
| Query keys follow command-based pattern: ['tauri', 'command_name'] | PASS | All keys use ['tauri', 'command_name'] pattern as const |
| Query key factory exports for listRecordings | PASS | src/lib/queryKeys.ts:15 |
| Query key factory exports for getRecordingState | PASS | src/lib/queryKeys.ts:18 |
| Query key factory exports for listAudioDevices | PASS | src/lib/queryKeys.ts:21 |
| Query key factory exports for getListeningStatus | PASS | src/lib/queryKeys.ts:24 |
| Query key factory exports for checkModelStatus(type) | PASS | src/lib/queryKeys.ts:27 with type parameter |
| TypeScript types are strict (no any, proper inference) | PASS | No 'any' types found, using 'as const' for proper inference |
| Exports are tree-shakeable (named exports, not default) | PASS | All exports use 'export const', no default exports |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| QueryClient can be instantiated without errors | PASS | src/lib/__tests__/queryKeys.test.ts:62-65 |
| Query keys are correctly typed and produce expected arrays | PASS | src/lib/__tests__/queryKeys.test.ts:6-34 |
| queryKeys.tauri.listRecordings equals ['tauri', 'list_recordings'] | PASS | src/lib/__tests__/queryKeys.test.ts:7-12 |
| queryKeys.tauri.checkModelStatus('tdt') equals ['tauri', 'check_parakeet_model_status', 'tdt'] | PASS | src/lib/__tests__/queryKeys.test.ts:37-43 |
| QueryClient default options verification | PASS | src/lib/__tests__/queryKeys.test.ts:67-73 |
| All query keys produce correct structure | PASS | src/lib/__tests__/queryKeys.test.ts:6-34 covers all static keys |
| Parameterized keys produce unique keys | PASS | src/lib/__tests__/queryKeys.test.ts:53-57 |

### Code Quality

**Strengths:**
- Excellent documentation with rationale comments explaining configuration choices
- Proper TypeScript usage with `as const` for literal type inference
- Comprehensive test coverage with 9 passing tests covering all acceptance criteria
- Clean separation of concerns between queryClient and queryKeys
- Export of helper types (QueryKeys, TauriQueryKey) for type safety in consuming code
- No deferrals or TODOs - complete implementation
- Follows spec implementation notes exactly

**Concerns:**
- None identified. This is a foundational spec that will be integrated via the `app-providers-wiring` spec (listed as dependency). Not being wired up yet is expected and correct.

### Integration Assessment

**Production Wiring Status:** DEFERRED (by design)
- This is explicitly marked as a "foundational spec" with no dependencies
- The `app-providers-wiring` spec (status: pending) lists this spec as a dependency and will wire QueryClientProvider into App.tsx
- The spec's "Related Specs" section correctly identifies the integration plan
- No production call sites yet is expected and correct for this phase

**End-to-End Flow:** N/A (foundation only)
- No backend changes in this spec
- No UI integration yet (handled by future specs)
- This provides the infrastructure that `*-query-hooks` specs will consume

### Verdict

**APPROVED** - All acceptance criteria met with comprehensive test coverage. Clean implementation following TypeScript best practices with proper typing and tree-shakeable exports. The lack of production wiring is expected and correct for a foundational infrastructure spec that will be integrated in the `app-providers-wiring` spec.

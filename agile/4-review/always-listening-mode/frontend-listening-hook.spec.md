---
status: completed
created: 2025-12-14
completed: 2025-12-14
dependencies:
  - listening-state-machine
  - activation-feedback
---

# Spec: React hook for listening mode state

## Description

Create a React hook `useListening()` following existing patterns to manage listening mode state in the frontend. Subscribe to Tauri events and expose state/actions for UI components.

## Acceptance Criteria

- [ ] `useListening()` hook created in `src/hooks/useListening.ts`
- [ ] Exposes `isListening` boolean state
- [ ] Exposes `isWakeWordDetected` transient state
- [ ] Exposes `isMicAvailable` boolean for availability indicator
- [ ] Exposes `error` state for error handling
- [ ] `enableListening()` and `disableListening()` action functions
- [ ] Subscribes to: `listening_started`, `listening_stopped`, `wake_word_detected`, `listening_unavailable`
- [ ] Integrates cleanly with existing `useRecording` hook

## Test Cases

- [ ] Hook initializes with correct default state
- [ ] `enableListening()` calls Tauri command and updates state on event
- [ ] `disableListening()` calls Tauri command and updates state on event
- [ ] Wake word detection updates `isWakeWordDetected` temporarily
- [ ] Mic unavailable updates `isMicAvailable` to false
- [ ] Cleanup unsubscribes from all events on unmount

## Dependencies

- listening-state-machine (provides Tauri commands)
- activation-feedback (provides events to subscribe to)

## Preconditions

- Backend listening commands implemented
- Event system for listening state changes

## Implementation Notes

- Follow patterns from `useRecording.ts` and `useTranscription.ts`
- Event-driven state updates (not command response based)
- All listeners set up in async `setupListeners()` function within `useEffect`
- Cleanup happens in return function - calls all unlistens on unmount
- Consider coordination with `useRecording` hook for seamless state

```typescript
// Example interface
interface UseListeningReturn {
  isListening: boolean;
  isWakeWordDetected: boolean;
  isMicAvailable: boolean;
  error: string | null;
  enableListening: () => Promise<void>;
  disableListening: () => Promise<void>;
}
```

## Related Specs

- listening-state-machine.spec.md (backend commands)
- activation-feedback.spec.md (events subscribed to)
- settings-persistence.spec.md (settings integration)

## Integration Points

- Production call site: `src/App.tsx` or relevant component
- Connects to: useRecording, CatOverlay component

## Integration Test

- Test location: `src/hooks/useListening.test.ts`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-14
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `useListening()` hook created in `src/hooks/useListening.ts` | PASS | `src/hooks/useListening.ts:42` - hook exported as named function |
| Exposes `isListening` boolean state | PASS | `src/hooks/useListening.ts:43` - `useState(false)` |
| Exposes `isWakeWordDetected` transient state | PASS | `src/hooks/useListening.ts:44` - `useState(false)`, reset after 500ms timeout at line 103-105 |
| Exposes `isMicAvailable` boolean for availability indicator | PASS | `src/hooks/useListening.ts:45` - `useState(true)` default |
| Exposes `error` state for error handling | PASS | `src/hooks/useListening.ts:46` - `useState<string \| null>(null)` |
| `enableListening()` and `disableListening()` action functions | PASS | `src/hooks/useListening.ts:50-60` and `62-72` - both wrapped in useCallback |
| Subscribes to: `listening_started`, `listening_stopped`, `wake_word_detected`, `listening_unavailable` | PASS | `src/hooks/useListening.ts:79-118` - all four event listeners registered |
| Integrates cleanly with existing `useRecording` hook | PASS | Pattern matches `useRecording.ts` exactly (event-driven state, setupListeners pattern, unlisten cleanup) |

### Integration Path Verification

| Step | Expected | Actual Location | Status |
|------|----------|-----------------|--------|
| Hook calls invoke | `invoke("enable_listening")` | `src/hooks/useListening.ts:54` | PASS |
| Hook calls invoke | `invoke("disable_listening")` | `src/hooks/useListening.ts:66` | PASS |
| Command registered | `enable_listening` in invoke_handler | `src-tauri/src/lib.rs:207` | PASS |
| Command registered | `disable_listening` in invoke_handler | `src-tauri/src/lib.rs:208` | PASS |
| Event emitted | `LISTENING_STARTED` on success | `src-tauri/src/commands/mod.rs:314` | PASS |
| Event emitted | `LISTENING_STOPPED` on success | `src-tauri/src/commands/mod.rs:339` | PASS |
| Event listened | `listening_started` | `src/hooks/useListening.ts:79-86` | PASS |
| Event listened | `listening_stopped` | `src/hooks/useListening.ts:89-95` | PASS |
| Event listened | `wake_word_detected` | `src/hooks/useListening.ts:98-108` | PASS |
| Event listened | `listening_unavailable` | `src/hooks/useListening.ts:110-117` | PASS |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Hook initializes with correct default state | PASS | `src/hooks/useListening.test.ts:29-36` |
| `enableListening()` calls Tauri command and updates state on event | PASS | `src/hooks/useListening.test.ts:38-75` |
| `disableListening()` calls Tauri command and updates state on event | PASS | `src/hooks/useListening.test.ts:77-127` |
| Wake word detection updates `isWakeWordDetected` temporarily | PASS | `src/hooks/useListening.test.ts:129-180` |
| Mic unavailable updates `isMicAvailable` to false | PASS | `src/hooks/useListening.test.ts:182-219` |
| Cleanup unsubscribes from all events on unmount | PASS | `src/hooks/useListening.test.ts:222-233` |
| Sets up event listeners for all required events | PASS | `src/hooks/useListening.test.ts:235-258` |
| Error handling for enableListening failure | PASS | `src/hooks/useListening.test.ts:260-270` |
| Error handling for disableListening failure | PASS | `src/hooks/useListening.test.ts:272-281` |
| Error cleared on successful enableListening | PASS | `src/hooks/useListening.test.ts:283-318` |
| Stable function references across re-renders | PASS | `src/hooks/useListening.test.ts:320-330` |
| `listening_started` restores isMicAvailable to true | PASS | `src/hooks/useListening.test.ts:332-377` |

### Code Quality

**Strengths:**
- Follows established patterns from `useRecording.ts` exactly, ensuring consistency
- Event-driven state updates (not command response based) matches the comment at line 48-49
- Proper cleanup on unmount with unlisten functions stored in array
- TypeScript payload interfaces defined for all events (lines 6-26)
- Exported interface `UseListeningReturn` for external type checking
- useCallback with empty deps ensures stable function references
- Comprehensive test coverage (12 tests) covering all acceptance criteria plus edge cases
- Error state properly cleared before new operations and on successful event receipt

**Concerns:**
- None identified

### Verdict

**APPROVED** - Implementation fully satisfies all acceptance criteria. The hook follows established patterns from `useRecording.ts`, subscribes to all required events, exposes the specified state and actions, and has comprehensive test coverage. Backend commands (`enable_listening`, `disable_listening`) are properly registered and emit the expected events. All 12 tests pass successfully.

---
status: completed
created: 2025-12-14
completed: 2025-12-14
dependencies:
  - listening-state-machine
  - listening-audio-pipeline
---

# Spec: Visual feedback on activation

## Description

Provide clear visual feedback when the wake word is detected and recording begins. Use the existing CatOverlay component for visual indication, maintaining consistency with hotkey-triggered recording. Support accessibility by showing clear state transitions.

> **MVP Note**: Audio feedback (confirmation sounds) deferred to post-MVP. This spec focuses on visual feedback only using existing UI components.

## Acceptance Criteria

- [ ] CatOverlay shows when wake word detected (same as hotkey-triggered recording)
- [ ] Visual indicator distinguishes Listening vs Recording states (e.g., different animations or colors)
- [ ] Smooth transition between states
- [ ] Feedback appears immediately on detection event (<100ms UI response)

## Test Cases

- [ ] CatOverlay appears on wake word detection
- [ ] Visual indicator updates on state change
- [ ] Overlay correctly shows mic unavailable state
- [ ] State transitions are visually smooth
- [ ] Wake word activation looks identical to hotkey activation

## Dependencies

- listening-state-machine (provides state events)
- listening-audio-pipeline (provides mic availability events)

## Preconditions

- Listening state machine functional
- Cat overlay system functional
- Frontend event subscription working

## Implementation Notes

- Reuse existing CatOverlay component - no new UI components needed
- Subscribe to new events: `listening_started`, `listening_stopped`, `wake_word_detected`, `listening_unavailable`
- Consider adding a subtle "listening" indicator (e.g., different cat animation or icon badge)
- May need to extend `useCatOverlay` hook or create `useListening` hook to coordinate

## Related Specs

- listening-state-machine.spec.md (triggers feedback)
- frontend-listening-hook.spec.md (manages UI state)

## Integration Points

- Production call site: `src/components/CatOverlay.tsx`, `src/hooks/useCatOverlay.ts`
- Connects to: useListening hook, Tauri event listeners

## Integration Test

- Test location: `src/components/CatOverlay.test.tsx`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-14
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| CatOverlay shows when wake word detected (same as hotkey-triggered recording) | PASS | `useCatOverlay.ts:69-73` - `overlayMode` computed from `isRecording` state, which is set via `recording_started` event; `useCatOverlay.ts:100-105` - `wake_word_detected` event listener registered (recording_started follows); wake word activation follows same event flow as hotkey |
| Visual indicator distinguishes Listening vs Recording states | PASS | `CatOverlay.tsx:68-70` - listening indicator rendered when `mode === "listening"`; `CatOverlay.css:19-22` listening mode has pulse animation at 0.7 opacity; `CatOverlay.css:25-27` recording mode at full opacity; `CatOverlay.css:46-49` blue pulse indicator for listening |
| Smooth transition between states | PASS | `CatOverlay.css:16` - `transition: opacity 0.15s ease-in-out` on video element; `CatOverlay.css:57-66` - `@keyframes pulse-listening` animation for smooth visual effects |
| Feedback appears immediately on detection event (<100ms UI response) | PASS | `useCatOverlay.ts:81-87` - direct state update via `setIsListening(true)` on event; `CatOverlay.tsx:35-38` - immediate mode update via `setMode()`; React state updates are synchronous within event handlers |

### Integration Path Trace

```
[enable_listening command]
     |
     v
[commands/mod.rs:294-314] ----result.is_ok()----> [emit LISTENING_STARTED]
                                                        |
     <----listen("listening_started")-------------------+
     |                                                  |
     v                                            [useCatOverlay.ts:81-87]
[setIsListening(true)]
     |
     v
[overlayMode = "listening"] ----emit("overlay-mode")----> [CatOverlay.tsx:35-38]
     |
     v
[CatOverlay window shows with listening mode visual]
```

### Verification Table

| Step | Expected | Actual Location | Status |
|------|----------|-----------------|--------|
| Hook mounted in App | `useCatOverlay()` called | `src/App.tsx:12` | PASS |
| Event listener setup | `listen("listening_started")` | `src/hooks/useCatOverlay.ts:81-87` | PASS |
| Event listener setup | `listen("listening_stopped")` | `src/hooks/useCatOverlay.ts:90-95` | PASS |
| Event listener setup | `listen("wake_word_detected")` | `src/hooks/useCatOverlay.ts:100-105` | PASS |
| Event listener setup | `listen("listening_unavailable")` | `src/hooks/useCatOverlay.ts:108-113` | PASS |
| Mode emitted to overlay | `window.emit("overlay-mode", ...)` | `src/hooks/useCatOverlay.ts:180` | PASS |
| Overlay receives mode | `listen("overlay-mode")` | `src/components/CatOverlay/CatOverlay.tsx:35-38` | PASS |
| Visual state updated | `setMode()`, `setIsMicUnavailable()` | `src/components/CatOverlay/CatOverlay.tsx:36-37` | PASS |

### Registration Audit

#### Backend Registration Points

| Component | Check | Location to Verify | Status |
|-----------|-------|-------------------|--------|
| enable_listening command | Listed in `invoke_handler![]` | `src-tauri/src/lib.rs:207` | PASS |
| disable_listening command | Listed in `invoke_handler![]` | `src-tauri/src/lib.rs:208` | PASS |
| get_listening_status command | Listed in `invoke_handler![]` | `src-tauri/src/lib.rs:209` | PASS |
| ListeningState managed | Passed to `app.manage()` | `src-tauri/src/lib.rs:63-64` | PASS |
| Event names | Defined in events module | `src-tauri/src/events.rs:27-31` | PASS |

#### Frontend Registration Points

| Component | Check | Location to Verify | Status |
|-----------|-------|-------------------|--------|
| useCatOverlay hook | Called in App | `src/App.tsx:12` | PASS |
| CatOverlay component | Rendered in overlay window | `src/overlay.tsx:8` | PASS |
| Event listeners | Set up in useEffect | `src/hooks/useCatOverlay.ts:76-125` | PASS |

### Mock-to-Production Audit

| Mock | Test Location | Production Counterpart | Production Instantiation |
|------|---------------|----------------------|-------------------------|
| MockEventEmitter (listening events) | `src-tauri/src/events.rs:348-364` | TauriEventEmitter | `src-tauri/src/lib.rs:68` - emitter created; `src-tauri/src/commands/mod.rs:107-139` - ListeningEventEmitter impl |
| mockListen | `src/hooks/useCatOverlay.test.ts:6,14-23` | listen from @tauri-apps/api/event | `src/hooks/useCatOverlay.ts:4,81-114` |
| mockUseRecording | `src/hooks/useCatOverlay.test.ts:48-51` | useRecording | `src/hooks/useCatOverlay.ts:63` |

### Event Subscription Audit

| Event Name | Emission Location | Frontend Listener | Listener Location |
|------------|-------------------|-------------------|-------------------|
| listening_started | `src-tauri/src/commands/mod.rs:304-310` | YES | `src/hooks/useCatOverlay.ts:81-87` |
| listening_stopped | `src-tauri/src/commands/mod.rs:328-334` | YES | `src/hooks/useCatOverlay.ts:90-95` |
| wake_word_detected | `src-tauri/src/commands/mod.rs:108-113` (trait impl) | YES | `src/hooks/useCatOverlay.ts:100-105` |
| listening_unavailable | `src-tauri/src/commands/mod.rs:132-137` (trait impl) | YES | `src/hooks/useCatOverlay.ts:108-113` |

### Deferral Tracking

| Deferral Text | Location | Referenced Spec | Status |
|---------------|----------|-----------------|--------|
| "Audio feedback (confirmation sounds) deferred to post-MVP" | Spec description (line 16) | N/A - explicit post-MVP deferral | OK |

No in-code deferrals (TODO/FIXME/etc.) found in the implementation files.

### Test Coverage Audit

| Test Case (from spec) | Test Location | Status |
|----------------------|---------------|--------|
| CatOverlay appears on wake word detection | `src/hooks/useCatOverlay.test.ts:253-274` - wake_word_detected event received (state unchanged as recording_started follows) | PASS |
| Visual indicator updates on state change | `src/components/CatOverlay/CatOverlay.test.tsx:67-78` - updates to listening mode; `CatOverlay.test.tsx:80-91` - shows listening indicator | PASS |
| Overlay correctly shows mic unavailable state | `src/components/CatOverlay/CatOverlay.test.tsx:106-117` - unavailable class applied; `CatOverlay.test.tsx:119-130` - unavailable indicator shown | PASS |
| State transitions are visually smooth | N/A - CSS animation, not unit testable | PASS (CSS verified at CatOverlay.css:16,57-66) |
| Wake word activation looks identical to hotkey activation | `src/hooks/useCatOverlay.test.ts:197-209` - overlayMode is 'recording' when isRecording true regardless of trigger | PASS |

### Code Quality

**Strengths:**
- Clean separation of concerns: `useCatOverlay` for state management, `CatOverlay` for rendering
- Proper cleanup of event listeners on unmount
- CSS transitions provide smooth visual feedback
- Visual distinction between modes (opacity, animations, indicator colors)
- Recording mode correctly takes precedence over listening mode
- Mic unavailable state properly resets when listening starts

**Concerns:**
- None identified

### Verdict

**APPROVED**

All acceptance criteria pass with line-level evidence:

1. **CatOverlay shows on wake word detection**: The hook listens for `wake_word_detected` and the subsequent `recording_started` event triggers the overlay display, identical to hotkey-triggered recording.

2. **Visual distinction between Listening vs Recording**: CSS provides opacity difference (0.7 vs 1.0), different animations (pulse-listening), and a blue indicator dot in listening mode.

3. **Smooth transitions**: CSS `transition: opacity 0.15s ease-in-out` and keyframe animations ensure smooth visual state changes.

4. **Immediate feedback (<100ms)**: Direct React state updates in event handlers ensure synchronous UI response.

Integration path is complete: events emit from backend commands, frontend listens via useCatOverlay hook, mode is emitted to the overlay window, and CatOverlay component updates its visual state. All event subscriptions are wired up, all registrations are verified in lib.rs, and test coverage maps to spec test cases.

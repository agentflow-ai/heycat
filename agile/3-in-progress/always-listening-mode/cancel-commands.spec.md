---
status: in-progress
created: 2025-12-14
completed: null
dependencies:
  - wake-word-detector
  - auto-stop-detection
---

# Spec: Voice commands to cancel false activations

## Description

Allow users to cancel recordings triggered by false wake word activations using voice commands like "cancel" or "nevermind". Abort the recording without saving or transcribing, and return to listening state.

## Acceptance Criteria

- [ ] "Cancel" spoken during recording aborts without transcription
- [ ] "Nevermind" spoken during recording aborts without transcription
- [ ] Cancellation phrases detected within first 3 seconds of recording
- [ ] `recording_cancelled` event emitted
- [ ] App returns to Listening state after cancellation
- [ ] Partial recordings are discarded (not saved to buffer)

## Test Cases

- [ ] "Cancel" spoken clearly triggers cancellation
- [ ] "Nevermind" spoken clearly triggers cancellation
- [ ] Cancel phrases ignored after 3-second window
- [ ] Similar words ("can't sell") don't trigger cancellation
- [ ] Cancellation works even with ambient noise
- [ ] Multiple cancel attempts don't cause issues

## Dependencies

- wake-word-detector (reuse detection infrastructure)
- auto-stop-detection (coordinate with silence detection)

## Preconditions

- Wake word detector functional
- Recording state machine with Listening state

## Implementation Notes

- Can reuse wake word detector with different target phrases
- Consider running cancel detection in parallel with main recording
- May need different confidence threshold than wake word
- All code in unified `listening/` module

## Related Specs

- wake-word-detector.spec.md (shared detection infrastructure)
- auto-stop-detection.spec.md (alternative stop mechanism)
- activation-feedback.spec.md (visual feedback on cancellation)

## Integration Points

- Production call site: `src-tauri/src/listening/detector.rs`
- Connects to: recording state machine, audio pipeline

## Integration Test

- Test location: `src-tauri/src/listening/detector_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-14
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| "Cancel" spoken during recording aborts without transcription | PASS | `cancel.rs:297-308` - `check_cancel_phrase()` detects "cancel" as isolated phrase with high confidence; `cancel.rs:262-280` - `analyze_and_emit()` emits event and calls `end_session()` |
| "Nevermind" spoken during recording aborts without transcription | PASS | `cancel.rs:299` - "nevermind" variants include "nevermind", "never mind", "nvm" |
| Cancellation phrases detected within first 3 seconds of recording | PASS | `cancel.rs:31-32` - `cancellation_window_secs: 3.0` default; `cancel.rs:169-183` - `is_window_open()` and `remaining_window_secs()` enforce window |
| `recording_cancelled` event emitted | PASS | `cancel.rs:269-273` - `emitter.emit_recording_cancelled()` called when detected; `events.rs:32` - `RECORDING_CANCELLED` constant defined; `events.rs:72-80` - `RecordingCancelledPayload` defined; `commands/mod.rs:140-146` - `TauriEventEmitter::emit_recording_cancelled()` implemented |
| App returns to Listening state after cancellation | FAIL | No evidence of state transition to Listening after cancellation. `end_session()` at `cancel.rs:159-167` only clears session data but does not interact with `RecordingManager` or trigger state transition. |
| Partial recordings are discarded (not saved to buffer) | FAIL | `end_session()` clears the cancel detector's internal buffer but there is no code to discard the recording buffer in `RecordingManager`. The main recording continues accumulating samples. |

### Integration Path Trace

```
[Recording Started]
     |
     v
[CancelPhraseDetector.start_session()]
     |
     v
[Audio samples pushed] ----push_samples()----> [CancelPhraseDetector buffer]
     |
     v
[analyze_and_emit()] ----detected?----> [emit_recording_cancelled event]
                                              |
                                              v
                                         [end_session()]
                                              |
                                              X (No state transition)
                                              X (No recording discard)
```

### Verification Table

| Step | Expected | Actual Location | Status |
|------|----------|-----------------|--------|
| CancelPhraseDetector created | In listening module | `listening/cancel.rs:88` | PASS |
| Module exported | `pub use cancel::*` | `listening/mod.rs:13-15` | PASS |
| Event name defined | `RECORDING_CANCELLED` | `events.rs:32` | PASS |
| Event payload defined | `RecordingCancelledPayload` | `events.rs:72-80` | PASS |
| TauriEventEmitter implements | `emit_recording_cancelled` | `commands/mod.rs:140-146` | PASS |
| Detector instantiated in app | Used in lib.rs or hotkey | NOT FOUND | FAIL |
| Detector wired to recording pipeline | `start_session()` called on recording start | NOT FOUND | FAIL |
| Audio samples routed to detector | `push_samples()` called during recording | NOT FOUND | FAIL |
| Analysis triggered periodically | `analyze_and_emit()` called | NOT FOUND | FAIL |
| State transition on cancellation | Returns to Listening state | NOT FOUND | FAIL |
| Recording buffer discarded | `RecordingManager.clear()` or similar | NOT FOUND | FAIL |

### Registration Audit

#### Backend Registration Points

| Component | Check | Location to Verify | Status |
|-----------|-------|-------------------|--------|
| `listening` module | mod declared | `lib.rs:10` | PASS |
| `CancelPhraseDetector` exported | pub use in mod.rs | `listening/mod.rs:13-15` | PASS |
| Detector instantiated | app.manage() or integration | `lib.rs` | FAIL - Not instantiated |
| Wired to hotkey integration | builder method | `hotkey/integration.rs` | FAIL - No cancel detector wiring |

#### Frontend Registration Points

| Component | Check | Location to Verify | Status |
|-----------|-------|-------------------|--------|
| Event listener for recording_cancelled | `listen("recording_cancelled")` | `src/hooks/useRecording.ts` | FAIL - No listener |

**New Items Registration Status:**

| Item | Type | Registered? | Evidence |
|------|------|-------------|----------|
| `CancelPhraseDetector` | struct | YES (exported) | `listening/mod.rs:13-15` |
| `CancelPhraseDetectorConfig` | struct | YES (exported) | `listening/mod.rs:13-15` |
| `CancelPhraseError` | enum | YES (exported) | `listening/mod.rs:13-15` |
| `CancelPhraseResult` | struct | YES (exported) | `listening/mod.rs:13-15` |
| `RECORDING_CANCELLED` | event name | YES | `events.rs:32` |
| `RecordingCancelledPayload` | struct | YES | `events.rs:72-80` |
| `TauriEventEmitter::emit_recording_cancelled` | impl | YES | `commands/mod.rs:140-146` |
| CancelPhraseDetector instance in app | managed state | NO | Not in `lib.rs` |
| Frontend event listener | hook | NO | Not in `useRecording.ts` |

### Mock-to-Production Audit

| Mock | Test Location | Production Counterpart | Production Instantiation |
|------|---------------|----------------------|-------------------------|
| `MockEventEmitter` | `events.rs:241-383` | `TauriEventEmitter` | `commands/mod.rs:51-58`, `lib.rs:68,121-122` |

`MockEventEmitter` includes `recording_cancelled_events` storage at `events.rs:308` and implements `emit_recording_cancelled` at `events.rs:380-382`. Production counterpart `TauriEventEmitter` implements the same method at `commands/mod.rs:140-146`.

**Issue:** No test in `cancel.rs` exercises `analyze_and_emit()` with the mock emitter.

### Event Subscription Audit

| Event Name | Emission Location | Frontend Listener | Listener Location |
|------------|-------------------|-------------------|-------------------|
| recording_cancelled | `cancel.rs:269-273` | NO | Not found in `useRecording.ts` |

### Deferral Tracking

No deferrals found in `cancel.rs`. The file is clean of TODO/FIXME comments.

### Test Coverage Audit

| Test Case (from spec) | Test Location | Status |
|----------------------|---------------|--------|
| "Cancel" spoken clearly triggers cancellation | `cancel.rs:439-445` `test_check_cancel_phrase_exact_match` | PASS |
| "Nevermind" spoken clearly triggers cancellation | `cancel.rs:447-454` `test_check_cancel_phrase_nevermind`, `cancel.rs:456-462` `test_check_cancel_phrase_never_mind_with_space`, `cancel.rs:597-603` `test_check_nvm_shorthand` | PASS |
| Cancel phrases ignored after 3-second window | `cancel.rs:420-426` `test_analyze_without_session_returns_window_expired`, `cancel.rs:531-545` `test_window_expires_after_timeout` | PASS |
| Similar words ("can't sell") don't trigger cancellation | `cancel.rs:492-502` `test_check_cancel_phrase_rejects_false_positives` | PASS |
| Cancellation works even with ambient noise | MISSING | No test with noisy audio samples |
| Multiple cancel attempts don't cause issues | `cancel.rs:605-618` `test_multiple_sessions` | PASS |

### Code Quality

**Strengths:**
- Clean implementation following wake word detector patterns
- Well-structured `CancelPhraseDetector` with session management
- Comprehensive false positive filtering for "can't sell", "can sell", etc.
- Proper error handling with `CancelPhraseError` enum
- Thread-safe design with `Arc<Mutex<>>` wrappers
- Good test coverage for detection logic
- Configurable thresholds with sensible defaults

**Concerns:**
- `analyze_and_emit()` is not tested with a mock emitter
- No test for ambient noise handling
- The spec states integration test location as `detector_test.rs` but this file does not exist

### Verdict

**NEEDS_WORK** - The `CancelPhraseDetector` core implementation is complete and well-tested for the detection logic, but critical integration is missing:

1. **Integration not wired:** The `CancelPhraseDetector` is not instantiated anywhere in the application (`lib.rs`, `hotkey/integration.rs`). No code calls `start_session()`, `push_samples()`, or `analyze_and_emit()` during actual recording.

2. **State transition missing:** The spec requires "App returns to Listening state after cancellation" but `end_session()` only clears internal detector state. There is no interaction with `RecordingManager` to transition state to `Listening`.

3. **Recording buffer not discarded:** The spec requires "Partial recordings are discarded" but no code clears the `RecordingManager` buffer when cancellation is detected.

4. **Frontend listener missing:** No frontend hook listens to the `recording_cancelled` event. `useRecording.ts` only listens to `recording_started`, `recording_stopped`, and `recording_error`.

5. **Missing test:** "Cancellation works even with ambient noise" test case is not implemented.

**How to Fix:**

1. **Wire detector into recording pipeline:**
   - Add `CancelPhraseDetector` to `HotkeyIntegration` builder pattern (similar to how `TranscriptionManager` is wired)
   - Call `start_session()` when recording starts (in `handle_toggle` Recording case)
   - Route audio samples to detector during recording
   - Call `analyze_and_emit()` periodically during recording (first 3 seconds)

2. **Add state transition and buffer discard:**
   - When cancellation detected, call `RecordingManager.abort_recording()` or similar method to:
     - Transition to `Listening` state (if listening mode enabled) or `Idle` state
     - Discard the recording buffer without saving

3. **Add frontend listener:**
   - In `useRecording.ts`, add `listen("recording_cancelled", ...)` to handle the event and update UI state

4. **Add missing test:**
   - Create test for ambient noise handling similar to `test_noise_buffer_does_not_crash` in `detector.rs`

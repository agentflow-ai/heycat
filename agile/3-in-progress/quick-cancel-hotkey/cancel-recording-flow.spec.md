---
status: in-review
created: 2025-12-17
completed: null
dependencies:
  - escape-key-listener
  - double-tap-detection
---

# Spec: Cancel recording without transcription

## Description

Implement the cancel flow that stops recording, discards audio data, and returns to idle state without triggering transcription. This is triggered by the double-tap detection.

## Acceptance Criteria

- [ ] Recording stops immediately on cancel
- [ ] Audio buffer cleared without encoding/saving WAV
- [ ] No `spawn_transcription()` called
- [ ] State transitions: `Recording -> Idle` (bypasses `Processing`)
- [ ] Silence detection stopped if active
- [ ] `recording_cancelled` event emitted with reason

## Test Cases

- [ ] Cancel during recording clears buffer
- [ ] Cancel does not create WAV file
- [ ] Cancel does not trigger transcription
- [ ] Cancel emits `recording_cancelled` event
- [ ] State is `Idle` after cancel
- [ ] Silence detection thread stopped on cancel

## Dependencies

- escape-key-listener (Escape key must be registered)
- double-tap-detection (triggers cancel flow)

## Preconditions

- Recording is in progress (`RecordingState::Recording`)
- Audio thread is capturing audio

## Implementation Notes

- Add `cancel_recording()` method to `HotkeyIntegration`
- Different from `stop_recording()` - does not encode or transcribe
- Call `audio_thread.stop()` but discard result
- Transition directly to `Idle` state
- Emit `recording_cancelled` event with `{ reason: "double-tap-escape" }`

## Related Specs

- double-tap-detection.spec.md (triggers this flow)
- cancel-ui-feedback.spec.md (consumes cancel event)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: `RecordingManager`, `AudioThread`, event emitters

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Pre-Review Gate Results

```
Build Warning Check:
warning: struct `RecordingCancelledPayload` is never constructed
warning: method `emit_recording_cancelled` is never used
warning: constant `RECORDING_CANCELLED` is never used
warning: method `cancel_recording` is never used
warning: `heycat` (lib) generated 4 warnings

Command Registration Check: PASS (no new commands added)
Event Subscription Check: FAIL - recording_cancelled event has no frontend listener
```

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Recording stops immediately on cancel | FAIL | `cancel_recording()` is never called from production code |
| Audio buffer cleared without encoding/saving WAV | FAIL | Method exists but unused (integration.rs:1259) |
| No `spawn_transcription()` called | FAIL | Method exists but unused |
| State transitions: Recording -> Idle | FAIL | Method exists but unused |
| Silence detection stopped if active | FAIL | `stop_silence_detection()` called in method but method never invoked |
| `recording_cancelled` event emitted with reason | FAIL | Event defined but never emitted in production |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Cancel during recording clears buffer | PASS | integration_test.rs:952 |
| Cancel does not create WAV file | PASS | integration_test.rs:952 (implicit) |
| Cancel does not trigger transcription | PASS | integration_test.rs:977 |
| Cancel emits recording_cancelled event | PASS | integration_test.rs:1000 |
| State is Idle after cancel | PASS | integration_test.rs:952 |
| Silence detection thread stopped on cancel | PASS | integration_test.rs:1114 |

### Code Quality

**Strengths:**
- Well-structured `cancel_recording()` method with proper error handling
- Comprehensive test coverage for the method itself (9 tests passing)
- Proper event payload structure with reason and timestamp
- Good documentation on the method

**Concerns:**
- **CRITICAL**: Code is TEST-ONLY - `cancel_recording()` is never called from production code
- Placeholder callback at lib.rs:157-160 just logs "Escape key pressed" instead of calling `cancel_recording`
- No frontend listener for `recording_cancelled` event (needed by cancel-ui-feedback.spec.md)
- 4 dead code warnings indicate the feature is not integrated

### What Would Break If This Code Was Deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `cancel_recording()` | fn | None | TEST-ONLY |
| `emit_recording_cancelled()` | fn | integration.rs:1310 | TEST-ONLY |
| `RecordingCancelledPayload` | struct | integration.rs:1310 | TEST-ONLY |
| `RECORDING_CANCELLED` | const | commands/mod.rs:80 | TEST-ONLY |

### Deferrals Found

| Deferral Text | Location | Tracking Spec |
|---------------|----------|---------------|
| "Escape key callback - placeholder for now, double-tap detection will be added in a later spec" | lib.rs:157 | **THIS SPEC** - but not implemented |

### Verdict

**NEEDS_WORK** - The `cancel_recording()` method is implemented but not wired up to production code. The escape callback at lib.rs:157-160 is still a placeholder that only logs, it does not call `cancel_recording()`. All 4 dead code warnings confirm this code is unreachable from production. To fix: update the escape callback in lib.rs to call `integration.cancel_recording()` when double-tap is detected.

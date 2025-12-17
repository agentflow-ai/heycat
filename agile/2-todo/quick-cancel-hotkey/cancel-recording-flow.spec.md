---
status: pending
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

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

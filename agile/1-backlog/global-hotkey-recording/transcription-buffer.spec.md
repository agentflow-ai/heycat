---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - recording-state-manager
  - recording-coordinator
---

# Spec: Audio Buffer Access for Transcription

## Description

Expose the audio buffer for transcription pipeline integration. Provides a command to retrieve the most recent recording's audio data for downstream processing (e.g., speech-to-text).

## Acceptance Criteria

- [ ] Command: `get_last_recording_buffer() -> Result<AudioData, String>`
- [ ] Return raw audio samples as base64-encoded bytes or Vec<f32>
- [ ] Buffer retained in memory after file save (configurable)
- [ ] Command returns error if no recording exists
- [ ] Optional: support for last N recordings buffer (default: 1)

## Test Cases

- [ ] Buffer accessible immediately after recording stops
- [ ] Correct audio data returned (matches saved WAV)
- [ ] Error returned when no previous recording exists
- [ ] Buffer cleared appropriately based on retention policy
- [ ] Large recordings handled without memory issues

## Dependencies

- [recording-state-manager.spec.md](recording-state-manager.spec.md) - Buffer storage
- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Buffer lifecycle

## Preconditions

- State manager retains buffer after recording
- Coordinator doesn't clear buffer on stop (or copies first)

## Implementation Notes

- Add command in `src-tauri/src/lib.rs`
- AudioData struct: `{ samples: Vec<f32>, sample_rate: u32, duration_secs: f64 }`
- Consider base64 encoding for large buffers to avoid JSON size issues
- Alternative: return file path and let frontend read file

## Related Specs

- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Buffer source
- [recording-state-manager.spec.md](recording-state-manager.spec.md) - Buffer storage

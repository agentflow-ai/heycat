---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - audio-capture
  - wav-encoding
  - recording-state-manager
---

# Spec: Recording Coordinator

## Description

Implement orchestration logic that coordinates between audio capture, WAV encoding, and state management for the full recording lifecycle. Handles start/stop operations and returns recording metadata.

## Acceptance Criteria

- [ ] `start_recording()`: Initialize capture stream, transition to Recording state
- [ ] `stop_recording()`: Stop stream, encode WAV, save file, return metadata
- [ ] Return `RecordingMetadata` struct (duration, file_path, sample_count)
- [ ] Clear audio buffer after successful save
- [ ] Handle errors at each step gracefully with descriptive messages

## Test Cases

- [ ] Full start→stop cycle produces WAV file at expected path
- [ ] Metadata contains correct duration and sample count
- [ ] Error during capture rolls back state to Idle
- [ ] Error during encoding preserves audio buffer for retry
- [ ] Concurrent start calls rejected when already recording

## Dependencies

- [audio-capture.spec.md](audio-capture.spec.md) - Audio primitives
- [wav-encoding.spec.md](wav-encoding.spec.md) - File encoding
- [recording-state-manager.spec.md](recording-state-manager.spec.md) - State management

## Preconditions

- All Layer 1 specs and Spec 2.1 completed
- State manager properly initialized

## Implementation Notes

- Create new module: `src-tauri/src/recording/coordinator.rs`
- RecordingMetadata struct: `{ duration_secs: f64, file_path: String, sample_count: usize }`
- Calculate duration: `sample_count / sample_rate`
- Use `?` operator for error propagation with context

## Related Specs

- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Exposes coordinator
- [event-emission.spec.md](event-emission.spec.md) - Emits events on state change
- [transcription-buffer.spec.md](transcription-buffer.spec.md) - Buffer access

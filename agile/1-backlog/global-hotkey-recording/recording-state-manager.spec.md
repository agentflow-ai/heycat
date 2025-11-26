---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - audio-capture
---

# Spec: Recording State Manager

## Description

Implement a Tauri-managed state structure that tracks recording status (Idle, Recording, Processing) with thread-safe access. Stores the audio buffer reference and provides methods for state transitions.

## Acceptance Criteria

- [ ] Define `RecordingState` enum (Idle, Recording, Processing)
- [ ] Store audio buffer reference (`Arc<Mutex<Vec<f32>>>`)
- [ ] Accessible via `State<'_, Mutex<RecordingManager>>`
- [ ] Provide `get_state()`, `transition_to()`, `get_audio_buffer()` methods
- [ ] Thread-safe access from multiple commands

## Test Cases

- [ ] State transitions correctly between Idle → Recording → Processing → Idle
- [ ] Audio buffer accessible while in Recording state
- [ ] Concurrent access from multiple threads handled safely
- [ ] Invalid transitions return error (e.g., Idle → Processing)

## Dependencies

- [audio-capture.spec.md](audio-capture.spec.md) - Uses audio buffer type

## Preconditions

- Audio capture module (Spec 1.1) completed
- Understanding of Tauri state management pattern

## Implementation Notes

- Create new module: `src-tauri/src/recording/state.rs`
- Use `tauri::Manager` trait for state access
- Register state in app builder: `.manage(Mutex::new(RecordingManager::new()))`
- State enum should derive `Clone, Serialize` for frontend access

## Related Specs

- [audio-capture.spec.md](audio-capture.spec.md) - Buffer type dependency
- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Uses state manager
- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Exposes state to frontend

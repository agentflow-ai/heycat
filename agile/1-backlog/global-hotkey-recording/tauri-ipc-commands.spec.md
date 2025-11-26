---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - recording-state-manager
  - recording-coordinator
---

# Spec: Tauri IPC Commands

## Description

Implement Tauri commands that expose recording functionality to the frontend via `invoke()`. Commands provide start/stop recording and state query operations.

## Acceptance Criteria

- [ ] Command: `start_recording() -> Result<(), String>`
- [ ] Command: `stop_recording() -> Result<RecordingMetadata, String>`
- [ ] Command: `get_recording_state() -> Result<RecordingStateInfo, String>`
- [ ] All commands access `State<Mutex<RecordingManager>>`
- [ ] Commands emit events on state changes (via event emission spec)

## Test Cases

- [ ] `start_recording` returns Ok when transitioning from Idle
- [ ] `stop_recording` returns metadata with correct file path
- [ ] `get_recording_state` returns current state enum value
- [ ] Commands return descriptive error messages on failure
- [ ] State query works during recording without blocking

## Dependencies

- [recording-state-manager.spec.md](recording-state-manager.spec.md) - State access
- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Business logic

## Preconditions

- State manager and coordinator specs completed
- Event emission spec ready (for integration)

## Implementation Notes

- Add commands in `src-tauri/src/lib.rs`
- Register in `invoke_handler`: `tauri::generate_handler![start_recording, stop_recording, get_recording_state]`
- Use `#[tauri::command]` attribute on each function
- Return types must be Serde-serializable

## Related Specs

- [event-emission.spec.md](event-emission.spec.md) - Events emitted by commands
- [recording-state-hook.spec.md](recording-state-hook.spec.md) - Frontend consumer

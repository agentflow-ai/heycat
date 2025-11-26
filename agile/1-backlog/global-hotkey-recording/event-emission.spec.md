---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - recording-coordinator
  - tauri-ipc-commands
---

# Spec: Event Emission for Frontend

## Description

Define and emit Tauri events that notify the frontend of recording state changes in real-time. Events enable the UI to update reactively without polling.

## Acceptance Criteria

- [ ] Event: `recording_started` with timestamp payload
- [ ] Event: `recording_stopped` with RecordingMetadata payload
- [ ] Event: `recording_error` with error message payload
- [ ] Events emitted via `AppHandle.emit()`
- [ ] Payloads are Serde-serializable

## Test Cases

- [ ] `recording_started` event emitted when recording begins
- [ ] `recording_stopped` event contains correct metadata
- [ ] `recording_error` event includes descriptive message
- [ ] Events received by frontend listeners
- [ ] Multiple listeners receive same event

## Dependencies

- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Triggers events
- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Command context for emit

## Preconditions

- Coordinator and IPC commands completed
- Understanding of Tauri event system

## Implementation Notes

- Use `app_handle.emit("event_name", payload)` pattern
- Event payloads: `RecordingStarted { timestamp: String }`, `RecordingStopped { metadata: RecordingMetadata }`
- Add `AppHandle` parameter to commands that need to emit
- Frontend listens via `listen("recording_started", callback)`

## Related Specs

- [recording-state-hook.spec.md](recording-state-hook.spec.md) - Frontend event listener
- [hotkey-integration.spec.md](hotkey-integration.spec.md) - Also emits events

---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - global-hotkey
  - recording-coordinator
  - tauri-ipc-commands
---

# Spec: Hotkey-to-Recording Integration

## Description

Connect the global hotkey to the recording coordinator for toggle behavior. First press starts recording, second press stops. Handles rapid toggle presses with debouncing.

## Acceptance Criteria

- [ ] Hotkey callback toggles recording (Idle → Recording → Idle)
- [ ] First press calls `start_recording()` via coordinator
- [ ] Second press calls `stop_recording()` via coordinator
- [ ] Emit events on toggle (via event emission)
- [ ] Handle rapid toggle presses gracefully (debounce ~200ms)

## Test Cases

- [ ] Toggle from Idle starts recording
- [ ] Toggle from Recording stops and saves
- [ ] Rapid double-press doesn't cause race condition
- [ ] Events emitted on each toggle
- [ ] Error state prevents toggle until reset

## Dependencies

- [global-hotkey.spec.md](global-hotkey.spec.md) - Hotkey registration
- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Recording logic
- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Command interface

## Preconditions

- Global hotkey and coordinator specs completed
- Event emission spec completed

## Implementation Notes

- Register hotkey callback during app setup in `lib.rs`
- Use `Instant::now()` for debounce timing
- Access state via `app.state::<Mutex<RecordingManager>>()`
- Callback runs on separate thread - use `app_handle.clone()` for emit

## Related Specs

- [global-hotkey.spec.md](global-hotkey.spec.md) - Provides hotkey callback
- [event-emission.spec.md](event-emission.spec.md) - Events for UI update

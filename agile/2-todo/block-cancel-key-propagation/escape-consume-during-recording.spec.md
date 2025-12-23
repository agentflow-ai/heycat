---
status: pending
created: 2025-12-23
completed: null
dependencies: ["cgeventtap-default-tap"]
---

# Spec: Consume Escape key during active recording

## Description

Add state tracking and logic to consume (block) Escape key events during active recording, preventing them from reaching other applications. Escape passes through normally when not recording.

> **Reference:** See [technical-guidance.md](./technical-guidance.md) for data flow diagrams:
> - "Component Interaction Diagram" - shows AtomicBool state and callback logic
> - "State Transition Diagram" - shows when consume_escape flag is set/cleared

## Acceptance Criteria

- [ ] Atomic state flag tracks "should consume Escape" mode
- [ ] Flag set to `true` when recording starts
- [ ] Flag set to `false` when recording stops or cancels
- [ ] CGEventTap callback returns `None` for Escape events when flag is `true`
- [ ] CGEventTap callback returns `Some(event)` for Escape when flag is `false`
- [ ] All non-Escape keys pass through regardless of flag state

## Test Cases

- [ ] Test: Flag is false by default, Escape events pass through
- [ ] Test: Flag set to true, Escape events are blocked (callback returns None)
- [ ] Test: Flag set back to false, Escape events pass through again
- [ ] Test: Non-Escape keys pass through regardless of flag state
- [ ] Test: Double-escape cancel still triggers when events are consumed

## Dependencies

- `cgeventtap-default-tap` - provides the callback mechanism for blocking events

## Preconditions

- CGEventTap is in DefaultTap mode (spec: cgeventtap-default-tap)

## Implementation Notes

- Add `AtomicBool` state flag for consume mode (thread-safe)
- Expose methods to set/clear the flag from HotkeyIntegration
- In CGEventTap callback: check flag + check if key is Escape
- Call set_consume(true) in `handle_toggle()` when starting recording
- Call set_consume(false) in `stop_recording_impl()` and `cancel_recording()`
- Key files:
  - `src-tauri/src/keyboard_capture/cgeventtap.rs` - callback logic
  - `src-tauri/src/hotkey/integration.rs` - state management

## Related Specs

- [cgeventtap-default-tap.spec.md](./cgeventtap-default-tap.spec.md) - prerequisite
- [consume-failure-notification.spec.md](./consume-failure-notification.spec.md) - parallel spec

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` (handle_toggle, stop_recording_impl, cancel_recording)
- Connects to: CGEventTap callback, recording state machine

## Integration Test

- Test location: Manual test in terminal - press Escape during recording, verify terminal doesn't receive it
- Verification: [ ] Integration test passes

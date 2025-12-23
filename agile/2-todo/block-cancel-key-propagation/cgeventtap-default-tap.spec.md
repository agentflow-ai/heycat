---
status: pending
created: 2025-12-23
completed: null
dependencies: []
---

# Spec: Change CGEventTap to DefaultTap mode

## Description

Change the CGEventTap from `ListenOnly` mode to `DefaultTap` mode, which allows the callback to control whether events are passed through or consumed. This is foundational infrastructure for blocking Escape key propagation.

> **Reference:** See [technical-guidance.md](./technical-guidance.md) for data flow diagrams:
> - "Current Behavior (ListenOnly Mode)" - shows the problem
> - "Target Behavior (DefaultTap Mode)" - shows the solution
> - "Component Interaction Diagram" - shows callback pseudocode

## Acceptance Criteria

- [ ] CGEventTap uses `CGEventTapOptions::DefaultTap` instead of `ListenOnly`
- [ ] Callback signature returns `Option<CGEvent>` to control event propagation
- [ ] Default behavior returns `Some(event)` to maintain current pass-through functionality
- [ ] All existing hotkey detection continues to work
- [ ] Unit tests verify mode change and callback behavior

## Test Cases

- [ ] Test that CGEventTap initializes successfully with DefaultTap mode
- [ ] Test that callback returning `Some(event)` allows event to pass through
- [ ] Test that callback returning `None` blocks event propagation
- [ ] Test that existing hotkey matching still works after mode change

## Dependencies

None - this is the foundational spec

## Preconditions

- Accessibility permissions granted (required for CGEventTap)

## Implementation Notes

- File: `src-tauri/src/keyboard_capture/cgeventtap.rs`
- Line 329: Change `CGEventTapOptions::ListenOnly` to `CGEventTapOptions::DefaultTap`
- Modify callback to return `Option<CGEvent>` instead of `()`
- For now, always return `Some(event)` to maintain existing behavior

## Related Specs

- [escape-consume-during-recording.spec.md](./escape-consume-during-recording.spec.md) - uses this callback mechanism
- [consume-failure-notification.spec.md](./consume-failure-notification.spec.md) - handles mode switch failures

## Integration Points

- Production call site: `src-tauri/src/keyboard_capture/cgeventtap.rs:329`
- Connects to: hotkey backend, recording integration

## Integration Test

- Test location: Manual testing with terminal app to verify events still pass through
- Verification: [ ] Integration test passes

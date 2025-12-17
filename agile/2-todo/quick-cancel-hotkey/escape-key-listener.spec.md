---
status: pending
created: 2025-12-17
completed: null
dependencies: []
---

# Spec: Register Escape key listener during recording

## Description

Register a global shortcut for the Escape key that is only active while recording is in progress. The listener should be registered when recording starts and unregistered when recording stops (either normally or via cancellation).

## Acceptance Criteria

- [ ] Escape key listener registered when recording starts
- [ ] Escape key listener unregistered when recording stops (normal or cancelled)
- [ ] Does not interfere with other Escape key usage when not recording
- [ ] Uses existing `ShortcutBackend` abstraction for testability

## Test Cases

- [ ] Escape key callback fires when pressed during recording
- [ ] Escape key callback does not fire when not recording
- [ ] Listener properly cleaned up after recording stops
- [ ] Multiple start/stop cycles work correctly

## Dependencies

None

## Preconditions

- Existing hotkey infrastructure in `src-tauri/src/hotkey/`
- `ShortcutBackend` trait available for registration

## Implementation Notes

- Add Escape key registration to `HotkeyIntegration` or `HotkeyService`
- Register in `handle_toggle()` when starting recording
- Unregister in `handle_toggle()` when stopping recording
- Store callback handle for cleanup

## Related Specs

- double-tap-detection.spec.md (consumes Escape key events)
- cancel-recording-flow.spec.md (triggered by double-tap)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` (handle_toggle)
- Connects to: `HotkeyService`, `ShortcutBackend`

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

---
status: pending
created: 2025-12-23
completed: null
dependencies: ["cgeventtap-default-tap"]
---

# Spec: Notify user when key blocking fails

## Description

Handle graceful degradation when DefaultTap mode cannot be established. Recording should still function, but the user is notified that Escape key blocking is unavailable.

> **Reference:** See [technical-guidance.md](./technical-guidance.md) for data flow diagrams:
> - "Error Handling Flow" - shows the failure detection and fallback path

## Acceptance Criteria

- [ ] Detect when CGEventTap fails to switch to DefaultTap mode
- [ ] Emit a warning/notification event to the frontend
- [ ] Recording functionality continues to work (graceful degradation)
- [ ] Cancel via double-escape still works (even though keys propagate)
- [ ] User sees notification explaining the limitation

## Test Cases

- [ ] Test: DefaultTap mode failure is detected and logged
- [ ] Test: Notification event is emitted on failure
- [ ] Test: Recording still starts despite blocking failure
- [ ] Test: Double-escape cancel still works when blocking unavailable

## Dependencies

- `cgeventtap-default-tap` - provides the mode switch that can fail

## Preconditions

- CGEventTap mode switch has been attempted

## Implementation Notes

- Check return value when switching to DefaultTap mode
- If mode switch fails, set a flag indicating blocking is unavailable
- Emit `key_blocking_unavailable` event to frontend
- Frontend can display a toast/notification to the user
- Key files:
  - `src-tauri/src/keyboard_capture/cgeventtap.rs` - mode switch error handling
  - `src-tauri/src/hotkey/integration.rs` - event emission
  - Frontend: notification component (existing toast system)

## Related Specs

- [cgeventtap-default-tap.spec.md](./cgeventtap-default-tap.spec.md) - prerequisite
- [escape-consume-during-recording.spec.md](./escape-consume-during-recording.spec.md) - parallel spec

## Integration Points

- Production call site: `src-tauri/src/keyboard_capture/cgeventtap.rs` (CGEventTap initialization)
- Connects to: Frontend notification system, event emitter

## Integration Test

- Test location: Difficult to test automatically (requires permission revocation)
- Verification: [ ] Manual verification or [x] N/A (edge case)

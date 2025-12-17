---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - escape-key-listener
---

# Spec: Detect double-tap pattern for cancel

## Description

Implement double-tap detection with a configurable time window. Single Escape key taps are ignored; only double-taps within the time window trigger the cancellation flow.

## Acceptance Criteria

- [ ] Double-tap detected within configurable time window (default 300ms)
- [ ] Single Escape tap does not cancel recording
- [ ] Triple+ taps within window treated as double-tap (cancel once)
- [ ] Timestamp tracking for tap detection

## Test Cases

- [ ] Two taps within 300ms triggers cancel
- [ ] Two taps with 500ms gap does not trigger cancel
- [ ] Single tap followed by nothing does not trigger cancel
- [ ] Three rapid taps triggers cancel only once
- [ ] Time window is configurable

## Dependencies

- escape-key-listener (provides Escape key events)

## Preconditions

- Escape key listener registered and firing events

## Implementation Notes

- Create `DoubleTapDetector` struct with configurable threshold
- Store `last_tap_time: Option<Instant>`
- On tap: check if within window of last tap
- If yes: trigger cancel callback, reset state
- If no: update last_tap_time

## Related Specs

- escape-key-listener.spec.md (provides events)
- cancel-recording-flow.spec.md (triggered on double-tap)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: Escape key listener callback

## Integration Test

- Test location: `src-tauri/src/hotkey/double_tap_test.rs`
- Verification: [ ] Integration test passes

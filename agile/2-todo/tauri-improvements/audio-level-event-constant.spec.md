---
status: pending
created: 2025-12-17
completed: null
dependencies: []
---

# Spec: Add typed constant for audio-level event

## Description

Add a typed event name constant for the "audio-level" event in the events module. Currently, the event is emitted using a string literal at `src-tauri/src/commands/mod.rs:663`, which is inconsistent with the pattern used for other events (all other events use constants from `event_names` module).

## Acceptance Criteria

- [ ] New constant `AUDIO_LEVEL` added to `src-tauri/src/events.rs` in the `event_names` module
- [ ] Emission site at `src-tauri/src/commands/mod.rs:663` uses the constant instead of string literal
- [ ] TypeScript frontend event listener uses matching event name (consistency check)

## Test Cases

- [ ] Audio monitor start/stop commands work correctly
- [ ] Frontend receives audio-level events as before

## Dependencies

None

## Preconditions

- Understanding of the event system pattern in `src-tauri/src/events.rs`

## Implementation Notes

1. Add to `src-tauri/src/events.rs` in the `event_names` module:
   ```rust
   pub const AUDIO_LEVEL: &str = "audio-level";
   ```

2. Update `src-tauri/src/commands/mod.rs:663`:
   ```rust
   // Before
   let _ = app_handle.emit("audio-level", level);

   // After
   let _ = app_handle.emit(event_names::AUDIO_LEVEL, level);
   ```

3. Verify frontend listener in `src/hooks/useAudioLevelMonitor.ts` matches

## Related Specs

None - isolated improvement

## Integration Points

- Production call site: `src-tauri/src/commands/mod.rs:663`
- Connects to: `src-tauri/src/events.rs`, frontend audio level monitor hook

## Integration Test

- Test location: N/A (event name constant change, existing behavior unchanged)
- Verification: [x] N/A

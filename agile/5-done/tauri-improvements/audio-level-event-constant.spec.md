---
status: completed
created: 2025-12-17
completed: 2025-12-17
dependencies: []
review_round: 1
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

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| New constant `AUDIO_LEVEL` added to `src-tauri/src/events.rs` in the `event_names` module | PASS | `/Users/michaelhindley/Documents/git/heycat/src-tauri/src/events.rs:14` - constant added |
| Emission site at `src-tauri/src/commands/mod.rs:663` uses the constant instead of string literal | PASS | `/Users/michaelhindley/Documents/git/heycat/src-tauri/src/commands/mod.rs:684` - uses `event_names::AUDIO_LEVEL` |
| TypeScript frontend event listener uses matching event name (consistency check) | PASS | `/Users/michaelhindley/Documents/git/heycat/src/hooks/useAudioLevelMonitor.ts:77` - listens to "audio-level" |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Audio monitor start/stop commands work correctly | PASS | Existing behavior unchanged - constant refactor only |
| Frontend receives audio-level events as before | PASS | No changes to event emission logic or frontend listener |

### Code Quality

**Strengths:**
- Constant correctly added to the `event_names` module following existing pattern
- Import of `event_names` already present in commands/mod.rs (line 18)
- Clean refactor with no behavioral changes
- Frontend listener already uses matching string literal "audio-level"
- No build warnings detected
- Event constant name uses kebab-case to match frontend convention

**Concerns:**
- None identified

### Pre-Review Gates

#### 1. Build Warning Check
```
(no output - no warnings detected)
```

#### 2. Event Subscription Check
Backend event defined: `audio-level` (line 14 in events.rs)
Frontend listener: `audio-level` (line 77 in useAudioLevelMonitor.ts)
Event is properly subscribed and wired end-to-end.

### Integration Verification

**Data Flow:**
```
[Audio Monitor Start Command]
     |
     v
[Command] src-tauri/src/commands/mod.rs:672 (start_audio_monitor)
     |
     v
[Audio Capture] src-tauri/src/audio (via AudioMonitorHandle)
     |
     v
[Event Emission] src-tauri/src/commands/mod.rs:684
     | emit(event_names::AUDIO_LEVEL, level)
     v
[Listener] src/hooks/useAudioLevelMonitor.ts:77
     | listen<number>("audio-level", ...)
     v
[State Update] levelRef.current = event.payload
     |
     v
[UI Re-render] Level displayed in device selector
```

**Production Call Site Verification:**

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| event_names::AUDIO_LEVEL | const | commands/mod.rs:684 | YES - called from start_audio_monitor command |

### Deferrals Check
```
No new deferred work introduced in this change.
Pre-existing deferred work unrelated to this spec.
```

### Verdict

**APPROVED** - All acceptance criteria met. The constant is properly defined, used at the emission site, and matches the frontend listener. No build warnings, no orphaned code, complete end-to-end integration verified. This is a clean refactor that improves consistency without changing behavior.

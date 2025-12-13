---
status: in-progress
created: 2025-12-13
completed: null
dependencies: []
---

# Spec: Use TranscriptionEventEmitter Trait

## Description

Refactor `HotkeyIntegration` to use the `TranscriptionEventEmitter` trait for emitting transcription events, matching the pattern established by `RecordingEventEmitter`. Currently, transcription events are emitted directly via `app_handle.emit()` in a spawned thread, bypassing the trait abstraction. This causes the compiler warning "trait TranscriptionEventEmitter is never used".

## Acceptance Criteria

- [ ] `HotkeyIntegration` receives a `TranscriptionEventEmitter` implementation as a second generic parameter
- [ ] `spawn_transcription()` uses trait methods instead of direct `app_handle.emit()` calls
- [ ] `TauriEventEmitter` implements both `RecordingEventEmitter` and `TranscriptionEventEmitter`
- [ ] No more "trait TranscriptionEventEmitter is never used" compiler warning
- [ ] Pattern consistent with `RecordingEventEmitter` usage

## Test Cases

- [ ] `MockEventEmitter` extended to implement `TranscriptionEventEmitter` for testing
- [ ] Existing transcription flow continues to work (manual test: hotkey → record → stop → transcribe → clipboard)
- [ ] Events are properly emitted (transcription_started, transcription_completed, transcription_error)

## Dependencies

None

## Preconditions

- Transcription pipeline is functional
- `TranscriptionEventEmitter` trait already defined in `src/events.rs`

## Implementation Notes

**Current state (bypassed pattern):**
```rust
// integration.rs:34 - stores AppHandle directly
app_handle: Option<AppHandle>,

// integration.rs:232-237 - direct emit calls
let _ = app_handle.emit(
    event_names::TRANSCRIPTION_STARTED,
    TranscriptionStartedPayload { timestamp: current_timestamp() },
);
```

**Target state (trait pattern):**
```rust
// Add second generic parameter
pub struct HotkeyIntegration<R: RecordingEventEmitter, T: TranscriptionEventEmitter> {
    recording_emitter: R,
    transcription_emitter: Arc<T>,  // Arc for thread-safe sharing
    // ...
}

// Use trait methods
self.transcription_emitter.emit_transcription_started(payload);
```

**Key files:**
- `src-tauri/src/hotkey/integration.rs` - main refactor
- `src-tauri/src/events.rs` - extend MockEventEmitter
- `src-tauri/src/commands/mod.rs` - wire up TauriEventEmitter

## Related Specs

None - standalone refactoring spec

## Integration Points

- Production call site: `src-tauri/src/commands/mod.rs` (where HotkeyIntegration is instantiated)
- Connects to: events.rs (trait definitions), hotkey/integration.rs (usage)

## Integration Test

- Test location: Manual verification via hotkey recording flow
- Verification: [ ] Integration test passes / [x] N/A (manual verification)

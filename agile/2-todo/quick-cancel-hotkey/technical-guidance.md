---
last-updated: 2025-12-17
status: draft
---

# Technical Guidance: Quick Cancel Hotkey

## Architecture Overview

### Layers/Components Involved

1. **Hotkey Layer** (`src-tauri/src/hotkey/`)
   - `HotkeyService` - manages global shortcut registration
   - `HotkeyIntegration` - orchestrates recording/transcription flow
   - `ShortcutBackend` trait - abstraction for testability

2. **Recording Layer** (`src-tauri/src/recording/`)
   - `RecordingManager` - state machine (Idle/Recording/Processing)
   - `AudioBuffer` - holds captured audio data

3. **Audio Layer** (`src-tauri/src/audio/`)
   - `AudioThread` - manages audio capture

4. **Frontend** (`src/hooks/`)
   - `useRecording.ts` - recording state and events

### Patterns

- **Lifecycle-scoped hotkey**: Escape listener only active during `Recording` state
- **Double-tap detection**: Time-windowed pattern matching (similar to existing debounce in `handle_toggle`)
- **Cancel vs Stop**: Separate code path that bypasses `Processing` state and transcription

### Integration with Existing Systems

```
RECORDING START (existing)
    ↓
register_escape_listener()  ← NEW
    ↓
ESCAPE KEY PRESS
    ↓
DoubleTapDetector.on_tap()  ← NEW
    ↓ (if double-tap)
cancel_recording()          ← NEW
    ├→ audio_thread.stop() (discard result)
    ├→ state → Idle (bypass Processing)
    ├→ emit recording_cancelled
    └→ unregister_escape_listener()
```

### Key Architectural Constraints

1. **Global shortcut conflicts**: Escape is commonly used - must only register during recording
2. **Thread safety**: Double-tap detector accessed from hotkey callback thread
3. **State consistency**: Cancel must cleanly reset all recording-related state
4. **No transcription**: Cancel path must never call `spawn_transcription()`

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Escape key (not recording hotkey) | Separate from toggle to avoid accidental cancels during normal use | 2025-12-17 |
| Double-tap (not single) | Prevents accidental cancellation; intentional gesture | 2025-12-17 |
| 300ms default window | Fast enough to feel responsive, slow enough to be intentional | 2025-12-17 |
| Lifecycle-scoped registration | Avoids conflicts with Escape key in other contexts | 2025-12-17 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-17 | Existing `handle_toggle` has 200ms debounce | Can use similar pattern for double-tap |
| 2025-12-17 | `ShortcutBackend` trait exists for testing | Can mock Escape key for unit tests |
| 2025-12-17 | `RecordingState` has direct `Idle` transition | Can bypass `Processing` for cancel |

## Open Questions

- [x] Which key to use? → Escape (per user request)
- [x] Single or double tap? → Double tap for intentional cancel
- [ ] Should cancel have audio feedback (beep/sound)?

## References

- `src-tauri/src/hotkey/integration.rs` - main orchestration
- `src-tauri/src/hotkey/mod.rs` - shortcut registration
- `src-tauri/src/recording/state.rs` - state machine
- `src/hooks/useRecording.ts` - frontend hook

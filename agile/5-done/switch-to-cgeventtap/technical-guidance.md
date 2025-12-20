---
last-updated: 2025-12-20
status: draft
---

# Technical Guidance: Switch To Cgeventtap

## Architecture Overview

### Current State (IOKit HID)

The application currently uses a **two-layer keyboard capture approach**:

1. **Global Hotkey Registration** (`src-tauri/src/hotkey/`)
   - Uses `tauri_plugin_global_shortcut` for standard combinations (Cmd+Shift+R)
   - Limited to modifiers + regular keys; cannot capture fn key alone

2. **IOKit HID Low-Level Capture** (`src-tauri/src/keyboard_capture/mod.rs`)
   - Requires Input Monitoring permission
   - Gets blocked by Karabiner-Elements' exclusive HID access
   - Created but NOT currently wired to commands

### Target Architecture (CGEventTap)

Replace IOKit HID with CGEventTap-based keyboard manager:

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React)                         │
│  useShortcutRecording hook ←──listen()──→ keyboard events   │
└─────────────────────────────────────────────────────────────┘
                              ↓ invoke()
┌─────────────────────────────────────────────────────────────┐
│                  Tauri Commands Layer                        │
│  start_keyboard_capture, stop_keyboard_capture               │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│              KeyboardManager (new module)                    │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ CGEventTap (captures all key events)                    ││
│  │  - Regular keys (A-Z, 0-9, symbols)                     ││
│  │  - Function keys (F1-F19)                               ││
│  │  - fn/Globe key (via FlagsChanged event)                ││
│  │  - Media keys (via NX_SYSDEFINED events)                ││
│  │  - Modifier keys (with L/R distinction via raw flags)   ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │ Permission Handler                                      ││
│  │  - AXIsProcessTrusted() check                           ││
│  │  - AXIsProcessTrustedWithOptions() prompt               ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              ↓ emit()
┌─────────────────────────────────────────────────────────────┐
│                     Frontend Events                          │
│  keyboard_event, keyboard_capture_started, _stopped, _error │
└─────────────────────────────────────────────────────────────┘
```

### Layers Involved

| Layer | Changes |
|-------|---------|
| `src-tauri/src/keyboard_manager/` | New CGEventTap-based module (replaces keyboard_capture) |
| `src-tauri/src/commands/mod.rs` | Wire up keyboard capture commands |
| `src-tauri/src/events.rs` | Add keyboard event types |
| `src/hooks/useShortcutRecording.ts` | Consume keyboard events for UI recording |
| Settings UI | Toggle for L/R modifier distinction |

### Key Patterns

1. **CGEventTap with kCGEventTapOptionListenOnly** - Observe events without blocking
2. **FlagsChanged event type** - Captures fn key state in event flags
3. **NX_SYSDEFINED event type** - Required for media keys
4. **CFRunLoop in spawned thread** - Process events without blocking main thread
5. **Event emission pattern** - Backend emits events, frontend subscribes

### Integration Points

- **Existing CGEvent usage**: Already using `core-graphics` crate for paste simulation
- **HotkeyIntegration**: May wire up for automatic hotkey registration from recorded shortcuts
- **Settings store**: Persist L/R modifier distinction preference
- **Keep tauri_plugin_global_shortcut**: Continue using for actual hotkey registration (out of scope to replace)

### Architectural Constraints

1. **Accessibility permission only** - No Input Monitoring required
2. **Works with Karabiner-Elements** - CGEventTap receives events after Karabiner transforms
3. **Thread safety** - CGEventTap callback may fire from any thread
4. **macOS 10.15+ only** - Uses modern CoreGraphics APIs

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| New module name: `keyboard_manager` | Distinguishes from existing `keyboard_capture` (IOKit HID); clearer purpose | 2025-12-19 |
| Keep `tauri_plugin_global_shortcut` | Only replaces low-level capture, not hotkey registration | 2025-12-19 |
| Use `kCGEventTapOptionListenOnly` | Observe without blocking; avoids interfering with other apps | 2025-12-19 |
| Spawn CFRunLoop in dedicated thread | Prevents blocking Tauri's async runtime | 2025-12-19 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-19 | Existing IOKit HID capture (`keyboard_capture/mod.rs`) not wired to commands | Can replace without breaking existing functionality |
| 2025-12-19 | `core-graphics` crate already in use for CGEvent posting (paste simulation) | Reuse existing dependency; unified API |
| 2025-12-19 | Apps like Wispr Flow work with only Accessibility permission using CGEventTap | Validates architectural approach |
| 2025-12-19 | fn key detected via `kCGEventFlagMaskSecondaryFn` (0x800000) in FlagsChanged events | Key technical detail for fn key capture |

## Open Questions

- [x] Can CGEventTap capture fn key? → Yes, via FlagsChanged event with `kCGEventFlagMaskSecondaryFn`
- [ ] How to handle media keys (volume, brightness)? → May need NX_SYSDEFINED event type

## References

- [Apple CGEventTap Documentation](https://developer.apple.com/documentation/coregraphics/cgeventtap)
- [CGEventFlags (includes fn key flag)](https://developer.apple.com/documentation/coregraphics/cgeventflags)
- [AXIsProcessTrusted (Accessibility check)](https://developer.apple.com/documentation/applicationservices/1460720-axisprocesstrusted)
- Existing code: `src-tauri/src/keyboard_capture/mod.rs` (IOKit HID implementation to replace)
- Existing code: `src-tauri/src/hotkey/integration.rs` (CGEvent posting for paste simulation)

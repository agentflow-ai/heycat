---
last-updated: 2025-12-23
status: draft
---

# Technical Guidance: Block Cancel Key Propagation

## Architecture Overview

The CGEventTap system captures keyboard events at the HID level on macOS. Currently it uses `ListenOnly` mode which observes events but allows them to pass through to other applications.

**Change:** Switch to `DefaultTap` mode which allows the callback to control event propagation:
- Return `Some(event)` → event passes through to other apps
- Return `None` → event is consumed/blocked

## Data Flow Diagrams

### Current Behavior (ListenOnly Mode)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CURRENT BEHAVIOR                              │
└─────────────────────────────────────────────────────────────────────────┘

  ┌──────────┐     ┌─────────────────┐     ┌─────────────────┐
  │ Keyboard │────▶│   CGEventTap    │────▶│ Other Apps      │
  │  (ESC)   │     │  (ListenOnly)   │     │ (Terminal, IDE) │
  └──────────┘     └────────┬────────┘     └─────────────────┘
                            │                      ▲
                            │ observe              │ ESC passes through!
                            ▼                      │
                   ┌─────────────────┐             │
                   │ DoubleTapDetector│─────────────┘
                   │ (detects cancel) │
                   └─────────────────┘

  Problem: ESC key reaches other apps even when canceling recording
```

### Target Behavior (DefaultTap Mode)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           TARGET BEHAVIOR                               │
└─────────────────────────────────────────────────────────────────────────┘

  ┌──────────┐     ┌─────────────────┐     ┌─────────────────┐
  │ Keyboard │────▶│   CGEventTap    │──X──│ Other Apps      │
  │  (ESC)   │     │  (DefaultTap)   │     │ (Terminal, IDE) │
  └──────────┘     └────────┬────────┘     └─────────────────┘
                            │
                            │ consume_escape=true?
                            │ key=ESC?
                            │ → return None (blocked)
                            ▼
                   ┌─────────────────┐
                   │ DoubleTapDetector│
                   │ (still triggers) │
                   └─────────────────┘

  Solution: ESC key consumed during recording, never reaches other apps
```

### Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        COMPONENT INTERACTION                            │
└─────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│                          HotkeyIntegration                               │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                        Recording State Machine                      │  │
│  │                                                                     │  │
│  │   ┌───────┐  start   ┌───────────┐  stop/cancel  ┌───────┐        │  │
│  │   │ Idle  │─────────▶│ Recording │──────────────▶│ Idle  │        │  │
│  │   └───────┘          └─────┬─────┘               └───────┘        │  │
│  │                            │                                       │  │
│  │                   set_consume_escape(true)                         │  │
│  │                            │                                       │  │
│  └────────────────────────────┼───────────────────────────────────────┘  │
│                               │                                          │
│                               ▼                                          │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                     CONSUME_ESCAPE: AtomicBool                      │  │
│  │                     (shared state flag)                             │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                               │                                          │
└───────────────────────────────┼──────────────────────────────────────────┘
                                │
                                │ read by
                                ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                          CGEventTap Callback                             │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │  fn callback(event: CGEvent) -> Option<CGEvent> {                  │  │
│  │      let key = get_key_code(event);                                │  │
│  │      let consume = CONSUME_ESCAPE.load(Ordering::SeqCst);          │  │
│  │                                                                     │  │
│  │      if consume && key == ESCAPE_KEY {                             │  │
│  │          // Still notify double-tap detector                       │  │
│  │          notify_escape_pressed();                                  │  │
│  │          return None;  // Block event                              │  │
│  │      }                                                              │  │
│  │                                                                     │  │
│  │      Some(event)  // Pass through                                  │  │
│  │  }                                                                  │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
```

### State Transition Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         STATE TRANSITIONS                               │
└─────────────────────────────────────────────────────────────────────────┘

                    consume_escape = false
                            │
                            ▼
    ┌─────────────────────────────────────────────────────────┐
    │                    IDLE STATE                           │
    │  - ESC passes through to other apps                     │
    │  - consume_escape = false                               │
    └────────────────────────┬────────────────────────────────┘
                             │
                             │ handle_toggle() [start recording]
                             │ set_consume_escape(true)
                             ▼
    ┌─────────────────────────────────────────────────────────┐
    │                 RECORDING STATE                         │
    │  - ESC blocked (returns None)                           │
    │  - consume_escape = true                                │
    │  - Double-tap ESC still detected internally             │
    └──────────┬─────────────────────────────────┬────────────┘
               │                                 │
               │ stop_recording()                │ cancel_recording()
               │ set_consume_escape(false)       │ set_consume_escape(false)
               ▼                                 ▼
    ┌─────────────────────────────────────────────────────────┐
    │                    IDLE STATE                           │
    │  - ESC passes through to other apps                     │
    │  - consume_escape = false                               │
    └─────────────────────────────────────────────────────────┘
```

### Error Handling Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        ERROR HANDLING FLOW                              │
└─────────────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────┐
    │              CGEventTap Initialization                  │
    └────────────────────────┬────────────────────────────────┘
                             │
                             │ create_tap(DefaultTap)
                             │
                ┌────────────┴────────────┐
                ▼                         ▼
    ┌───────────────────┐     ┌───────────────────────────────┐
    │     SUCCESS       │     │          FAILURE              │
    │  DefaultTap mode  │     │  (permissions, system error)  │
    │  blocking works   │     └───────────────┬───────────────┘
    └───────────────────┘                     │
                                              ▼
                              ┌───────────────────────────────┐
                              │   Emit notification event     │
                              │   "key_blocking_unavailable"  │
                              └───────────────┬───────────────┘
                                              │
                                              ▼
                              ┌───────────────────────────────┐
                              │   Fallback to ListenOnly      │
                              │   Recording still works       │
                              │   ESC passes through (old     │
                              │   behavior)                   │
                              └───────────────────────────────┘
```

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Use DefaultTap mode | Only way to consume events in CGEventTap | 2025-12-23 |
| Block only Escape key | Minimize disruption to user workflow | 2025-12-23 |
| Block only during recording | Escape should work normally otherwise | 2025-12-23 |
| Graceful degradation | Recording works even if blocking fails | 2025-12-23 |
| Use AtomicBool for state | Thread-safe access from callback | 2025-12-23 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-23 | CGEventTap uses ListenOnly mode at line 329 | Need to change to DefaultTap |
| 2025-12-23 | Callback currently returns () not Option | Need to change signature |
| 2025-12-23 | Escape key code is 53 (0x35) | Use for key matching |

## Open Questions

- [x] Should we block all keys or just Escape? → Just Escape
- [x] Should we block when not recording? → No, only during recording

## Files to Modify

| File | Changes | Spec |
|------|---------|------|
| `src-tauri/src/keyboard_capture/cgeventtap.rs` | Change to DefaultTap, modify callback return type | cgeventtap-default-tap |
| `src-tauri/src/keyboard_capture/cgeventtap.rs` | Add consume logic in callback | escape-consume-during-recording |
| `src-tauri/src/hotkey/integration.rs` | Add AtomicBool state, set/clear on state changes | escape-consume-during-recording |
| `src-tauri/src/hotkey/integration.rs` | Emit notification on failure | consume-failure-notification |

## References

- [Apple CGEventTap Documentation](https://developer.apple.com/documentation/coregraphics/cgeventtap)
- [CGEventTapOptions](https://developer.apple.com/documentation/coregraphics/cgeventtapoptions)

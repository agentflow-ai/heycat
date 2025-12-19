---
status: in-progress
created: 2025-12-19
completed: null
dependencies: []
---

# Spec: Core CGEventTap keyboard event capture

## Description

Implement core CGEventTap-based keyboard event capture that can detect ALL keyboard events:
- Regular keys (letters, numbers, symbols, special keys)
- Modifier keys with left/right distinction
- fn/Globe key via FlagsChanged
- Full modifier state tracking

This creates a new module using CGEventTap to listen for KeyDown, KeyUp, and FlagsChanged events.

## Acceptance Criteria

- [ ] CGEventTap created listening for KeyDown, KeyUp, FlagsChanged events
- [ ] Regular keys captured (A-Z, 0-9, F1-F19, Space, Tab, Enter, Escape, arrows, etc.)
- [ ] Fn key detected via FlagsChanged with flag 0x00800000
- [ ] All standard modifiers detected (Command, Control, Alt/Option, Shift)
- [ ] Left/Right modifier distinction via device flags (NX_DEVICEL*KEYMASK / NX_DEVICER*KEYMASK)
- [ ] Key codes converted to human-readable names
- [ ] Numpad keys distinguished from regular number keys
- [ ] Event tap runs on CFRunLoop in dedicated thread
- [ ] Callback mechanism to emit captured events
- [ ] Clean shutdown when stop() is called

## Test Cases

- [ ] Single key: Press "A" alone → callback with key_name="A", no modifiers
- [ ] Single key: Press "Space" alone → callback with key_name="Space"
- [ ] Single key: Press "F5" alone → callback with key_name="F5"
- [ ] Modifier alone: Press Left-Command alone → callback with command=true, command_left=true
- [ ] Modifier alone: Press Right-Shift alone → callback with shift=true, shift_right=true
- [ ] fn key alone: Press fn → callback with fn_key=true
- [ ] Combo: fn+F → callback with fn_key=true, key_name="F"
- [ ] Combo: Cmd+Shift+R → callback with command=true, shift=true, key_name="R"
- [ ] Numpad: Press Numpad-1 → callback with key_name="Numpad1" (distinct from "1")
- [ ] Start and stop capture cleanly without leaks

## Dependencies

None - this is foundational

## Preconditions

- macOS 10.15+ (Catalina or later)
- core-graphics crate v0.24 available
- Accessibility permission granted

## Implementation Notes

### Event Types
```rust
// Listen for all keyboard-related events
let event_mask = (1 << CGEventType::KeyDown as u64)
               | (1 << CGEventType::KeyUp as u64)
               | (1 << CGEventType::FlagsChanged as u64);
```

### Flag Constants
```rust
// Standard modifier flags
const CG_EVENT_FLAG_MASK_SHIFT: u64 = 0x00020000;
const CG_EVENT_FLAG_MASK_CONTROL: u64 = 0x00040000;
const CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x00080000;
const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x00100000;
const CG_EVENT_FLAG_MASK_SECONDARY_FN: u64 = 0x00800000;

// Left/Right device flags (from IOKit)
const NX_DEVICELSHIFTKEYMASK: u64 = 0x00000002;
const NX_DEVICERSHIFTKEYMASK: u64 = 0x00000004;
const NX_DEVICELCTLKEYMASK: u64 = 0x00000001;
const NX_DEVICERCTLKEYMASK: u64 = 0x00002000;
const NX_DEVICELALTKEYMASK: u64 = 0x00000020;
const NX_DEVICERALTKEYMASK: u64 = 0x00000040;
const NX_DEVICELCMDKEYMASK: u64 = 0x00000008;
const NX_DEVICERCMDKEYMASK: u64 = 0x00000010;
```

### Updated Event Structure
```rust
pub struct CapturedKeyEvent {
    pub key_code: u32,
    pub key_name: String,
    pub fn_key: bool,
    pub command: bool,
    pub command_left: bool,
    pub command_right: bool,
    pub control: bool,
    pub control_left: bool,
    pub control_right: bool,
    pub alt: bool,
    pub alt_left: bool,
    pub alt_right: bool,
    pub shift: bool,
    pub shift_left: bool,
    pub shift_right: bool,
    pub pressed: bool,
    pub is_media_key: bool,
}
```

File location: `src-tauri/src/keyboard_capture/cgeventtap.rs` (new file)

## Related Specs

- accessibility-permission.spec.md - permission checking
- media-key-capture.spec.md - media key handling (NSSystemDefined)
- replace-iokit-hid.spec.md - integration with existing code

## Integration Points

- Production call site: N/A (standalone module, integrated in replace-iokit-hid spec)
- Connects to: core-graphics crate, core-foundation crate

## Integration Test

- Test location: N/A (unit-only spec)
- Verification: [x] N/A

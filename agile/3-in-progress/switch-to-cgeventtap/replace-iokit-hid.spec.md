---
status: in-progress
created: 2025-12-19
completed: null
dependencies:
  - cgeventtap-core
  - accessibility-permission
  - media-key-capture
---

# Spec: Replace IOKit HID with CGEventTap implementation

## Description

Replace the IOKit HID implementation in keyboard_capture/mod.rs with the CGEventTap-based implementation. This integrates all capture modules (cgeventtap-core, accessibility-permission, media-key-capture) into the existing KeyboardCapture struct, expanding the API to support all key types.

## Acceptance Criteria

- [ ] KeyboardCapture struct uses CGEventTap internally instead of IOKit HID
- [ ] CapturedKeyEvent struct EXPANDED with new fields (left/right modifiers, is_media_key)
- [ ] Permission check uses Accessibility instead of Input Monitoring
- [ ] Error messages updated to reference Accessibility permission
- [ ] IOKit HID code removed from keyboard_capture module
- [ ] commands/mod.rs updated to use new permission handling
- [ ] Existing start_shortcut_recording/stop_shortcut_recording commands work unchanged
- [ ] Media keys captured and emitted correctly
- [ ] Left/Right modifier distinction available in events

## Test Cases

- [ ] Start/stop keyboard capture works with new implementation
- [ ] CapturedKeyEvent emitted with expanded structure
- [ ] Permission error message mentions Accessibility (not Input Monitoring)
- [ ] Compiles without IOKit HID imports in keyboard_capture
- [ ] Media key events have is_media_key=true
- [ ] Left-Command has command_left=true, Right-Command has command_right=true

## Dependencies

- cgeventtap-core - provides CGEventTap capture implementation
- accessibility-permission - provides permission checking
- media-key-capture - provides media key handling

## Preconditions

- All dependency specs completed

## Implementation Notes

Files to modify:
- `src-tauri/src/keyboard_capture/mod.rs` - replace IOKit with CGEventTap
- `src-tauri/src/commands/mod.rs` - update permission error handling

Key changes:
1. Remove IOKit HID imports and hid_access module
2. Import new cgeventtap and permissions modules
3. Update KeyboardCapture::start() to use CGEventTap
4. Update error messages from "Input Monitoring" to "Accessibility"
5. Expand CapturedKeyEvent with new fields

Expand (BREAKING - needs frontend update):
```rust
pub struct CapturedKeyEvent {
    pub key_code: u32,
    pub key_name: String,
    pub fn_key: bool,
    pub command: bool,
    pub command_left: bool,   // NEW
    pub command_right: bool,  // NEW
    pub control: bool,
    pub control_left: bool,   // NEW
    pub control_right: bool,  // NEW
    pub alt: bool,
    pub alt_left: bool,       // NEW
    pub alt_right: bool,      // NEW
    pub shift: bool,
    pub shift_left: bool,     // NEW
    pub shift_right: bool,    // NEW
    pub pressed: bool,
    pub is_media_key: bool,   // NEW
}
```

Preserve:
- KeyboardCapture public API (new, start, stop, is_running)
- KeyboardCaptureState Tauri managed state

## Related Specs

- cgeventtap-core.spec.md - CGEventTap implementation
- accessibility-permission.spec.md - permission handling
- integration-test.spec.md - end-to-end testing

## Integration Points

- Production call site: `src-tauri/src/commands/mod.rs:894` (start_shortcut_recording)
- Connects to: keyboard_capture module, Tauri commands

## Integration Test

- Test location: Manual testing via integration-test spec
- Verification: [ ] Integration test passes

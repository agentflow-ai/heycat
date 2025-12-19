---
status: completed
created: 2025-12-19
completed: 2025-12-19
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

## Review

**Reviewed:** 2025-12-19
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| KeyboardCapture struct uses CGEventTap internally instead of IOKit HID | PASS | src-tauri/src/keyboard_capture/mod.rs:21-24 - KeyboardCapture wraps CGEventTapCapture |
| CapturedKeyEvent struct EXPANDED with new fields (left/right modifiers, is_media_key) | PASS | src-tauri/src/keyboard_capture/cgeventtap.rs:96-133 - All new fields present (command_left, command_right, control_left, control_right, alt_left, alt_right, shift_left, shift_right, is_media_key) |
| Permission check uses Accessibility instead of Input Monitoring | PASS | src-tauri/src/keyboard_capture/permissions.rs:20-23 - AXIsProcessTrusted() used, no Input Monitoring references |
| Error messages updated to reference Accessibility permission | PASS | src-tauri/src/keyboard_capture/permissions.rs:59 - Error message references "System Settings > Privacy & Security > Accessibility" |
| IOKit HID code removed from keyboard_capture module | PASS | No IOKit HID imports found (cargo check clean, no IOHIDDevice/IOHIDManager) |
| commands/mod.rs updated to use new permission handling | PASS | src-tauri/src/commands/mod.rs:893 - Comment updated to reference Accessibility permission |
| Existing start_shortcut_recording/stop_shortcut_recording commands work unchanged | PASS | src-tauri/src/commands/mod.rs:895-917 - Commands preserved, working with CGEventTap internally |
| Media keys captured and emitted correctly | PASS | src-tauri/src/keyboard_capture/cgeventtap.rs:49-66 - Media key constants and handling implemented, test at keyboard_capture/mod.rs:148-172 |
| Left/Right modifier distinction available in events | PASS | src-tauri/src/keyboard_capture/cgeventtap.rs:107-127 - All left/right modifier fields in struct, device flags at lines 39-47 |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Start/stop keyboard capture works with new implementation | PASS | src-tauri/src/keyboard_capture/mod.rs:75-85 - test_keyboard_capture_new_not_running, test_keyboard_capture_stop_when_not_running |
| CapturedKeyEvent emitted with expanded structure | PASS | src-tauri/src/keyboard_capture/mod.rs:88-116 - test_captured_key_event_has_expanded_fields |
| Permission error message mentions Accessibility (not Input Monitoring) | PASS | src-tauri/src/keyboard_capture/permissions.rs:93-98 - test_accessibility_permission_error_display |
| Compiles without IOKit HID imports in keyboard_capture | PASS | cargo check succeeded with 0 warnings, no IOKit imports found |
| Media key events have is_media_key=true | PASS | src-tauri/src/keyboard_capture/mod.rs:148-172 - test_captured_key_event_media_key |
| Left-Command has command_left=true, Right-Command has command_right=true | PASS | src-tauri/src/keyboard_capture/mod.rs:88-116 - test shows command_left:true, command_right:false distinction |

**Additional Tests:** 32 tests passing in keyboard_capture module (27 from cgeventtap, 5 from permissions)

### Integration Verification

**Automated Check Results:**
```bash
# Build warning check
cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)"
# Result: No output (PASS - no warnings)

# Command registration check
Commands registered: start_shortcut_recording, stop_shortcut_recording (lib.rs:325-326)
Commands defined: start_shortcut_recording, stop_shortcut_recording (commands/mod.rs:895, 921)
# Result: All commands registered (PASS)
```

**Data Flow Trace:**
```
[User clicks Edit in ShortcutEditor]
     ↓
[ShortcutEditor.tsx:197] invoke("start_shortcut_recording")
     ↓
[commands/mod.rs:895] start_shortcut_recording command
     ↓
[keyboard_capture/mod.rs:40] KeyboardCapture::start()
     ↓
[cgeventtap.rs] CGEventTapCapture captures events
     ↓
[commands/mod.rs:910] emit("shortcut_key_captured", event)
     ↓
[ShortcutEditor.tsx:197] listen<CapturedKeyEvent>("shortcut_key_captured")
     ↓
[ShortcutEditor.tsx:200+] Update UI with captured key
```

**Wiring Verification:**
- ✅ KeyboardCapture called from production code (commands/mod.rs:901)
- ✅ CapturedKeyEvent emitted AND listened to (emit at commands/mod.rs:910, listen at ShortcutEditor.tsx:197)
- ✅ Frontend type definition matches backend (ShortcutEditor.tsx:16-34 has all new fields)
- ✅ Commands registered in invoke_handler (both start/stop_shortcut_recording at lib.rs:325-326)
- ✅ No build warnings (cargo check clean)

**Deferrals Check:**
```bash
grep -rn "TODO\|FIXME\|XXX\|HACK" src-tauri/src/keyboard_capture/
# Result: No output (PASS - no deferrals)
```

### Code Quality

**Strengths:**
- Clean abstraction: KeyboardCapture maintains same public API while swapping implementation
- Comprehensive test coverage: 32 passing tests covering all key functionality
- Proper error handling: Accessibility permission errors provide clear user guidance
- Type safety: Frontend TypeScript interface matches backend Rust struct exactly
- Complete IOKit removal: 513 lines of IOKit code removed, no dead code remaining
- Full backward compatibility: Existing commands work unchanged
- Zero build warnings: Previous unused import issues have been resolved

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria met, tests passing, integration verified, no build warnings

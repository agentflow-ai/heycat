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
| CapturedKeyEvent struct EXPANDED with new fields (left/right modifiers, is_media_key) | PASS | src-tauri/src/keyboard_capture/cgeventtap.rs:98-133 - All new fields present (command_left, command_right, control_left, control_right, alt_left, alt_right, shift_left, shift_right, is_media_key) |
| Permission check uses Accessibility instead of Input Monitoring | PASS | src-tauri/src/keyboard_capture/permissions.rs - AXIsProcessTrusted() used, no Input Monitoring references |
| Error messages updated to reference Accessibility permission | PASS | src-tauri/src/keyboard_capture/permissions.rs:59 and src-tauri/src/commands/mod.rs:893 - Error messages reference "System Settings > Privacy & Security > Accessibility" |
| IOKit HID code removed from keyboard_capture module | PASS | No IOKit references found (verified via grep, cargo check clean) |
| commands/mod.rs updated to use new permission handling | PASS | src-tauri/src/commands/mod.rs:893 - Comment updated to reference Accessibility permission |
| Existing start_shortcut_recording/stop_shortcut_recording commands work unchanged | PASS | src-tauri/src/commands/mod.rs:895-931 - Commands preserved with same API signatures |
| Media keys captured and emitted correctly | PASS | src-tauri/src/keyboard_capture/cgeventtap.rs:49-66 - Media key constants defined, test at keyboard_capture/mod.rs:148-172 |
| Left/Right modifier distinction available in events | PASS | src-tauri/src/keyboard_capture/cgeventtap.rs:108-128 - All left/right modifier fields in struct, device flags at lines 39-47 |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Start/stop keyboard capture works with new implementation | PASS | src-tauri/src/keyboard_capture/mod.rs:75-85 |
| CapturedKeyEvent emitted with expanded structure | PASS | src-tauri/src/keyboard_capture/mod.rs:88-116 |
| Permission error message mentions Accessibility (not Input Monitoring) | PASS | src-tauri/src/keyboard_capture/permissions.rs:93-98 |
| Compiles without IOKit HID imports in keyboard_capture | PASS | cargo check succeeded with 0 warnings, no IOKit imports found |
| Media key events have is_media_key=true | PASS | src-tauri/src/keyboard_capture/mod.rs:148-172 |
| Left-Command has command_left=true, Right-Command has command_right=true | PASS | src-tauri/src/keyboard_capture/mod.rs:88-116 |

**Test Results:** 32 tests passed; 0 failed

### Integration Verification

**Pre-Review Gates:**
1. Build Warning Check: PASS (no warnings)
2. Command Registration Check: PASS (start_shortcut_recording and stop_shortcut_recording registered at lib.rs:325-326)
3. Event Subscription Check: PASS (shortcut_key_captured emitted at commands/mod.rs:910, listened at ShortcutEditor.tsx:255)

**Manual Review:**

1. Is the code wired up end-to-end? YES
   - KeyboardCapture::start() called from production code at commands/mod.rs:901
   - CGEventTapCapture instantiated at keyboard_capture/mod.rs:30
   - CapturedKeyEvent emitted AND listened to (emit at commands/mod.rs:910, listen at ShortcutEditor.tsx:255)
   - Commands registered in invoke_handler at lib.rs:325-326

2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| KeyboardCapture | struct | commands/mod.rs:901 | YES - via start_shortcut_recording |
| CapturedKeyEvent | struct | commands/mod.rs:910 | YES - emitted to frontend |
| CGEventTapCapture | struct | keyboard_capture/mod.rs:30 | YES - via KeyboardCapture |
| check_accessibility_permission | fn | cgeventtap.rs:229 | YES - called before starting capture |

3. Where does the data flow?

```
[UI Action: Click Edit in ShortcutEditor]
     ↓
[Frontend] ShortcutEditor.tsx:255 listen("shortcut_key_captured")
     ↓
[Frontend] ShortcutEditor.tsx invoke("start_shortcut_recording")
     ↓
[Backend Command] commands/mod.rs:895 start_shortcut_recording
     ↓
[Capture Start] keyboard_capture/mod.rs:40 KeyboardCapture::start()
     ↓
[CGEventTap] cgeventtap.rs CGEventTapCapture::start()
     ↓
[Event Capture] cgeventtap.rs callback invoked for key events
     ↓
[Emit Event] commands/mod.rs:910 emit("shortcut_key_captured", event)
     ↓
[Frontend Listener] ShortcutEditor.tsx:255 receives CapturedKeyEvent
     ↓
[State Update] ShortcutEditor.tsx updates recording state
     ↓
[UI Re-render] ShortcutEditor displays captured key
```

4. Are there any deferrals? NO
   - No TODO/FIXME/XXX/HACK comments found in keyboard_capture module

5. Automated check results:
```bash
# Build warning check
cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)"
# Result: No output (PASS)

# Command registration check
grep -A50 "invoke_handler" src-tauri/src/lib.rs | grep "commands::"
# Result: start_shortcut_recording and stop_shortcut_recording registered (PASS)

# Deferrals check
grep -rn "TODO\|FIXME\|XXX\|HACK" src-tauri/src/keyboard_capture/
# Result: No output (PASS)
```

### Code Quality

**Strengths:**
- Clean abstraction: KeyboardCapture maintains same public API while internally switching from IOKit to CGEventTap
- Comprehensive test coverage: 32 tests passing, covering all acceptance criteria
- Proper error handling: Accessibility permission errors provide clear user guidance
- Type safety: Frontend TypeScript interface (ShortcutEditor.tsx:17-35) matches backend Rust struct exactly
- Complete IOKit removal: No IOKit references remaining, verified via grep
- Full backward compatibility: Existing commands work unchanged with same signatures
- Zero build warnings: cargo check clean
- Complete data flow: End-to-end integration from UI to backend and back verified

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria met, all tests passing, integration verified end-to-end, no build warnings, no deferrals, code is wired up in production

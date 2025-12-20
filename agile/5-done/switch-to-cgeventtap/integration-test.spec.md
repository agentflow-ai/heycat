---
status: completed
created: 2025-12-19
completed: null
dependencies:
  - replace-iokit-hid
  - frontend-shortcut-display
---

# Spec: Manual integration testing

## DescriptionYes.

Perform comprehensive manual testing of the CGEventTap-based keyboard capture implementation. This verifies the feature works end-to-end with real user interaction, testing ALL key tyYes.pes including media keys, with Karabiner-Elements running.

## Acceptance Criteria

- [ ] All regular keys detected (A-Z, 0-9, symbols)
- [ ] All special keys detected (Space, Tab, Enter, Escape, arrows, Delete, etc.)
- [ ] All function keys detected (F1-F19)
- [ ] fn/Globe key detected when pressed alone
- [ ] fn + other key combinations detected
- [ ] All modifier keys detected (Cmd, Ctrl, Alt, Shift)
- [ ] Left/Right modifier distinction works
- [ ] Media keys detected (volume, brightness, play/pause)
- [ ] Numpad keys distinguished from regular numbers
- [ ] Works with Karabiner-Elements running
- [ ] Only Accessibility permission required (not Input Monitoring)
- [ ] Shortcut recording flow works end-to-end
- [ ] Permission prompt appears and guides user correctly

## Test Cases

### Single Keys
- [ ] Press "A" → detected as key_name="A"
- [ ] Press "5" → detected as key_name="5"
- [ ] Press "Space" → detected as key_name="Space"
- [ ] Press "Tab" → detected as key_name="Tab"
- [ ] Press "Enter" → detected as key_name="Enter"
- [ ] Press "Escape" → detected as key_name="Escape"
- [ ] Press "F5" → detected as key_name="F5"
- [ ] Press "Left Arrow" → detected as key_name="Left"
- [ ] Press "Delete" → detected as key_name="Delete"

### Modifier-Only Hotkeys
- [ ] Press Left-Command alone → command=true, command_left=true
- [ ] Press Right-Command alone → command=true, command_right=true
- [ ] Press Left-Shift alone → shift=true, shift_left=true
- [ ] Press Right-Shift alone → shift=true, shift_right=true
- [ ] Press fn alone → fn_key=true

### Combinations
- [ ] Cmd+A → command=true, key_name="A"
- [ ] Ctrl+Shift+S → control=true, shift=true, key_name="S"
- [ ] fn+F → fn_key=true, key_name="F"
- [ ] Cmd+Shift+R → command=true, shift=true, key_name="R"
- [ ] Left-Cmd+Right-Shift+K → command_left=true, shift_right=true, key_name="K"

### Media Keys
- [ ] Press Volume Up → key_name="VolumeUp", is_media_key=true
- [ ] Press Volume Down → key_name="VolumeDown", is_media_key=true
- [ ] Press Mute → key_name="Mute", is_media_key=true
- [ ] Press Brightness Up → key_name="BrightnessUp", is_media_key=true
- [ ] Press Play/Pause → key_name="PlayPause", is_media_key=true

### Numpad
- [ ] Press Numpad 1 → key_name="Numpad1" (distinct from "1")
- [ ] Press Numpad Enter → key_name="NumpadEnter"
- [ ] Press Numpad + → key_name="NumpadPlus"

### Karabiner Compatibility
- [ ] With Karabiner running: all above tests pass
- [ ] No "Input Monitoring permission" errors

### Permission Flow
- [ ] Without Accessibility permission → clear error message with guidance
- [ ] Grant Accessibility permission → capture works after restart

## Dependencies

- replace-iokit-hid - complete implementation must be in place

## Preconditions

- App built and running
- Karabiner-Elements installed (for compatibility testing)
- External keyboard with numpad (for numpad testing, optional)
- Ability to grant/revoke Accessibility permission

## Implementation Notes

Test procedure:
1. Build debug app bundle: `bun run tauri build --debug`
2. Revoke Accessibility permission for heycat if previously granted
3. Run app and trigger shortcut recording
4. Verify Accessibility permission prompt appears
5. Grant permission in System Settings
6. Restart app
7. Run through all test cases with Karabiner running

## Related Specs

- replace-iokit-hid.spec.md - the implementation being tested
- cgeventtap-core.spec.md - core capture logic
- media-key-capture.spec.md - media key handling

## Integration Points

- Production call site: N/A (testing spec)
- Connects to: Full application flow

## Integration Test

- Test location: Manual testing
- Verification: [ ] All test cases pass manually

---
status: in-progress
created: 2025-12-20
completed: null
dependencies: ["fix-display-conversion"]
---

# Spec: Fix backend hotkey registration for Function modifier on startup

## Description

After app restart, saved shortcuts containing the "Function" modifier are not being properly registered with the hotkey system. The backend loads "Function+R" from settings but the hotkey handler doesn't trigger when the Fn key is pressed - no key press logs appear at all. This spec ensures the backend correctly parses and registers "Function" as a valid modifier when restoring hotkeys from persistent storage.

## Acceptance Criteria

- [ ] After app restart, pressing Fn triggers the recording hotkey if Fn was saved
- [ ] Backend logs show key press events for Fn key after restart
- [ ] Hotkey registration completes without errors for "Function+..." shortcuts
- [ ] Both new hotkey setting and restored hotkey work identically

## Test Cases

- [ ] Set Fn+R as hotkey → restart app → press Fn+R → recording starts
- [ ] Set Function+CmdOrControl+R → restart → verify key press logs appear
- [ ] Verify no regression: CmdOrControl+Shift+R still works after restart

## Dependencies

- `fix-display-conversion` - Display fix should be done first so we can verify visually

## Preconditions

- Hotkey persistence is working (saves to settings.json)
- CGEventTap backend is active (macOS)
- Spec 1 (display fix) is complete for visual verification

## Implementation Notes

**Files to investigate:**
- `src-tauri/src/hotkey/mod.rs` - Hotkey service entry point
- `src-tauri/src/hotkey/cgeventtap_backend.rs` - macOS hotkey registration
- `src-tauri/src/commands/mod.rs:752` - `resume_recording_shortcut()` function
- `src-tauri/src/keyboard_capture/cgeventtap.rs` - Key capture, Fn key mapping

**Likely issue areas:**
1. `resume_recording_shortcut()` may not properly parse "Function" when loading from store
2. The CGEventTap backend may not map "Function" string back to the correct key code
3. The shortcut parser may not recognize "Function" as a valid modifier

**Debug approach:**
1. Add logging to `resume_recording_shortcut()` to see what's loaded from store
2. Trace the shortcut registration path to find where "Function" parsing fails
3. Compare code path for fresh hotkey set vs. restored hotkey

## Related Specs

- `fix-display-conversion.spec.md` - Frontend display fix (prerequisite)

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (app startup) → `resume_recording_shortcut()`
- Connects to: Hotkey service, CGEventTap backend, settings store

## Integration Test

- Test location: Manual test - set Fn hotkey, restart app, press Fn, verify recording triggers
- Verification: [ ] Integration test passes

---
status: in-review
created: 2025-12-20
completed: null
dependencies: ["fix-display-conversion"]
review_round: 1
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

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Pre-Review Gates

#### 1. Build Warning Check
```
warning: method `register_recording_shortcut` is never used
    = note: `#[warn(dead_code)]` on by default
warning: `heycat` (lib) generated 1 warning
```
**FAIL** - Dead code warning for `register_recording_shortcut` method on `HotkeyServiceDyn` (hotkey/mod.rs:145-152). This method uses a hardcoded `RECORDING_SHORTCUT` constant and is now unused since production code calls `backend.register()` directly with the saved shortcut.

#### 2. Command Registration Check
All commands properly registered - PASS

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| After app restart, pressing Fn triggers the recording hotkey if Fn was saved | DEFERRED | Manual test - no automated verification |
| Backend logs show key press events for Fn key after restart | DEFERRED | Manual test - no automated verification |
| Hotkey registration completes without errors for "Function+..." shortcuts | PASS | lib.rs:251-261 loads saved shortcut and registers via `backend.register()`, CGEventTap parse_shortcut handles "function" modifier (cgeventtap_backend.rs:83) |
| Both new hotkey setting and restored hotkey work identically | PASS | Fixed - lib.rs:299-311 now reads saved shortcut from settings.json for unregistration |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Set Fn+R as hotkey, restart, press Fn+R, recording starts | MISSING | Manual test only |
| Set Function+CmdOrControl+R, restart, verify key press logs | MISSING | Manual test only |
| Verify no regression: CmdOrControl+Shift+R still works | PASS | Existing tests cover default shortcut |

### Code Quality

**Strengths:**
- Correctly reads saved shortcut from settings store with fallback to default (lib.rs:251-256)
- Uses `backend.register()` directly which supports "Function" modifier (cgeventtap_backend.rs:83)
- Logs the shortcut being registered for debugging (lib.rs:257)
- Unregistration now properly reads saved shortcut to unregister the correct one (lib.rs:299-311)
- Symmetric registration/unregistration logic using same settings store key

**Concerns:**
- **Dead code warning** - `register_recording_shortcut` and `unregister_recording_shortcut` methods on `HotkeyServiceDyn` (hotkey/mod.rs:145-158) are now unused since lib.rs bypasses them to call `backend.register()` and `backend.unregister()` directly. These methods should either be removed or annotated with `#[allow(dead_code)]` with a comment explaining they're kept for API completeness.

### Data Flow Analysis

```
[App Startup]
     |
     v
[lib.rs:251-256] Load saved_shortcut from settings.json
     |
     v
[lib.rs:261] service.backend.register(&saved_shortcut, callback)
     | Registers e.g. "Function+R"
     v
[Window Destroyed]
     |
     v
[lib.rs:299-306] Load shortcut from settings.json (same key)
     |
     v
[lib.rs:306] service.backend.unregister(&shortcut)
     | Correctly unregisters "Function+R"
     v
[Clean Shutdown] - No resource leak
```

### Verdict

**NEEDS_WORK** - The unregistration mismatch from the previous review has been fixed. However, the dead code warning for `register_recording_shortcut` on `HotkeyServiceDyn` must be resolved. Either:
1. Add `#[allow(dead_code)]` annotation with a comment explaining these methods are kept for API completeness, or
2. Remove the unused methods entirely since production code now uses `backend.register()` directly

The spec cannot be approved with a build warning present.

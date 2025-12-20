---
last-updated: 2025-12-20
status: validated
---

# Technical Guidance: Recording Hotkey Changes On Restart

## Root Cause Analysis

The bug has two related causes:

1. **Frontend Display Issue:** The `backendToDisplay()` function in `GeneralSettings.tsx` is missing the conversion for "Function" → "fn". When hotkeys are loaded from persistent storage after app restart, the function converts "CmdOrControl" → "⌘", "Shift" → "⇧", etc., but leaves "Function" unconverted, resulting in "FunctionR" being displayed instead of "fnR".

2. **Backend Registration Issue:** After app restart, the backend loads the saved shortcut (e.g., "Function+R") but the hotkey registration system may not properly parse "Function" as a valid modifier, causing the hotkey to not trigger recording. No key press logs appear when pressing the Fn key.

## Key Files

| File | Purpose |
|------|---------|
| `src/pages/components/GeneralSettings.tsx:12-19` | `backendToDisplay()` - missing Function conversion |
| `src/pages/components/ShortcutEditor.tsx:125-140` | `formatBackendKeyForBackend()` - converts "fn" to "Function" |
| `src-tauri/src/commands/mod.rs:752` | `resume_recording_shortcut()` - restores hotkey on startup |
| `src-tauri/src/commands/mod.rs:802-877` | `update_recording_shortcut()` / `get_recording_shortcut()` |
| `src-tauri/src/hotkey/cgeventtap_backend.rs` | macOS hotkey registration |
| `src-tauri/src/keyboard_capture/cgeventtap.rs:630-632` | Maps key code 63/179 to "fn" |

## Fix Approach

**Spec 1 - Display Fix:**
Add `.replace(/Function/gi, "fn")` to the `backendToDisplay()` function in `GeneralSettings.tsx`.

**Spec 2 - Backend Fix:**
Investigate and fix the hotkey registration path to ensure "Function" modifier is properly parsed and registered when restoring saved shortcuts on app startup.

## Regression Risk

- Display changes could affect other modifier key conversions (low risk - isolated function)
- Backend changes could affect hotkey registration for non-Fn shortcuts (medium risk - needs testing)
- Changes to CGEventTap backend could affect macOS-specific behavior

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-20 | `backendToDisplay()` missing Function→fn conversion | Display shows "FunctionR" instead of "fnR" |
| 2025-12-20 | Hotkey saved as "Function+R" but not triggering after restart | Recording not activated by hotkey |
| 2025-12-20 | No key press logs when pressing Fn after restart | Backend not receiving key events |

## Open Questions

- [x] Root cause identified for display issue
- [x] Root cause identified for backend issue
- [ ] Exact location of parsing failure in backend hotkey path

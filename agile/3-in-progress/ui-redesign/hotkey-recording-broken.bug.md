---
status: pending
severity: major
origin: manual
created: 2025-12-18
completed: null
parent_feature: "ui-redesign"
parent_spec: null
---

# Bug: Hotkey recording modal styling and functionality broken

**Created:** 2025-12-18
**Severity:** Major

## Problem Description

Two issues with the keyboard shortcut modal in settings:

1. **Styling issue:** The hotkey display box does not follow the app's theming. It appears as grey with white text regardless of the current theme, breaking visual consistency.

2. **Functionality issue:** The hotkey recording feature doesn't work. When clicking to record a new hotkey, the UI correctly enters recording mode, but pressing key combinations does not register or update the hotkey. Nothing happens when you try to set a new shortcut.

**Expected:**
- Hotkey box should match the current theme styling
- Pressing key combinations in recording mode should capture and display the new hotkey

**Actual:**
- Hotkey box is always grey/white, ignoring theme
- Key presses during recording mode are not captured

## Steps to Reproduce

1. Open the app and go to Settings
2. Navigate to the keyboard shortcuts section
3. Observe the hotkey display box styling (grey background, white text - doesn't match theme)
4. Click on the hotkey box to start recording a new shortcut
5. Press a new key combination (e.g., Cmd+Shift+H)
6. Observe that the key combination is not captured

## Root Cause

Two separate root causes:

1. **Styling issue:** The `<kbd>` element in `ShortcutEditor.tsx` (line 193) used hardcoded `bg-neutral-100` class instead of the theme-aware `bg-surface-elevated` token. This color doesn't change with dark mode, causing the grey appearance regardless of theme.

2. **Functionality issue:** Global shortcuts registered via `tauri_plugin_global_shortcut` intercept keyboard events at the system level *before* they reach the webview. When the user tries to record a new shortcut (e.g., Cmd+Shift+R), the existing global hotkey handler captures the event, preventing the webview's `keydown` listener from receiving it.

## Fix Approach

1. **Styling fix:** Replace `bg-neutral-100` with `bg-surface-elevated` and add `text-text-primary` for proper text color theming.

2. **Functionality fix:**
   - Add two new Tauri commands: `suspend_recording_shortcut` and `resume_recording_shortcut`
   - `suspend_recording_shortcut` unregisters the global Cmd+Shift+R shortcut temporarily
   - `resume_recording_shortcut` re-registers it with the same callback
   - Update `ShortcutEditor` to call `suspend_recording_shortcut` when entering recording mode and `resume_recording_shortcut` when exiting (via successful recording, cancel, or modal close)

## Acceptance Criteria

- [ ] Bug no longer reproducible
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression
- [ ] Related specs/features not broken

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Hotkey box in light theme | Box uses light theme colors | [ ] |
| Hotkey box in dark theme | Box uses dark theme colors | [ ] |
| Click hotkey box to record | UI enters recording state | [ ] |
| Press key combo while recording | New hotkey is captured and displayed | [ ] |
| Save new hotkey | Hotkey persists after settings close | [ ] |

## Integration Points

- Settings page component
- Hotkey recording component
- Theme system / design tokens
- Tauri backend for hotkey registration

## Integration Test

E2E test: Navigate to settings, change hotkey, verify new hotkey works globally

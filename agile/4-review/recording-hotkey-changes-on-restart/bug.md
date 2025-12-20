---
status: pending
severity: critical
origin: manual
created: 2025-12-20
completed: null
parent_feature: null
parent_spec: null
---

# Bug: Recording Hotkey Changes On Restart

**Created:** 2025-12-20
**Owner:** Claude
**Severity:** Critical

## Problem Description

When setting the recording hotkey to Fn, it works correctly until app restart. After restart, it displays as a literal 'function' key instead of 'Fn' and the hotkey no longer triggers recording (no key press logs appear).

## Steps to Reproduce

1. Open Settings > General > Keyboard Shortcuts
2. Change recording hotkey to Fn key
3. Verify hotkey works (displays as "fn" and triggers recording)
4. Restart the application
5. Observe: hotkey displays as "Function" instead of "fn" and doesn't trigger recording

## Root Cause

Two related issues:

1. **Frontend Display Issue:** The `backendToDisplay()` function in `GeneralSettings.tsx` was missing the conversion for "Function" to "fn", causing saved Fn hotkeys to display incorrectly after restart.

2. **Backend Registration Issue:** On app startup, `lib.rs` was always registering the hardcoded default shortcut `CmdOrControl+Shift+R` instead of loading the saved shortcut from settings. This meant custom shortcuts were never restored.

## Fix Approach

1. **Spec 1 (fix-display-conversion):** Added `.replace(/Function/gi, "fn")` to the `backendToDisplay()` function in `GeneralSettings.tsx`.

2. **Spec 2 (fix-backend-hotkey-loading):** Modified `lib.rs` startup code to:
   - Load the saved shortcut from settings instead of using the default
   - Use `service.backend.register()` directly with the saved shortcut
   - Also fixed the cleanup handler to unregister the correct shortcut

## Acceptance Criteria

- [x] Bug no longer reproducible
- [x] Root cause addressed (not just symptoms)
- [x] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Set Fn hotkey, restart, check display | Shows "fn" not "Function" | [x] |
| Set Fn hotkey, restart, press Fn | Recording triggers | [x] |
| Set Cmd+Shift+R, restart, verify works | No regression | [x] |

## Definition of Done

- [x] Bug no longer reproducible
- [x] Root cause addressed (not just symptoms)
- [x] Unit tests verify the fix
- [x] Code reviewed and approved

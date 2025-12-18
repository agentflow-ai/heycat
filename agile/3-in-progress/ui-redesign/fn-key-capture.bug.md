---
status: pending
severity: minor
origin: manual
created: 2025-12-18
completed: null
parent_feature: "ui-redesign"
parent_spec: null
---

# Bug: Cannot capture fn key and some special keys in shortcut recorder

**Created:** 2025-12-18
**Severity:** Minor

## Problem Description

The keyboard shortcut recorder in Settings cannot capture the `fn` (Function) key on Mac, either alone or in combination with other keys. The current implementation uses JavaScript's `KeyboardEvent` API in the frontend, which does not expose the `fn` key state.

**What works:**
- A-Z, 0-9, symbols
- F1-F12 function keys
- Modifier combinations (⌘⇧R, ⌃⌥K, etc.)
- Single keys without modifiers

**What doesn't work:**
- `fn` key alone
- `fn` + other key combinations (the `fn` state is not captured)

**Expected:** User should be able to set `fn` or `fn+key` as a global hotkey.

**Actual:** Pressing `fn` produces no event; `fn+key` only captures the key without the `fn` modifier.

## Steps to Reproduce

1. Open Settings → General → Keyboard Shortcuts
2. Click "Change" next to Toggle Recording
3. Click "Record New Shortcut"
4. Press the `fn` key alone, or `fn + R`
5. Observe: No key is captured (fn alone) or only "R" is captured (fn+R)

## Root Cause

JavaScript's `KeyboardEvent` API does not expose the `fn` key. Unlike `metaKey`, `ctrlKey`, `altKey`, and `shiftKey`, there is no `fnKey` property. The `fn` key is a hardware-level modifier handled by macOS at a lower level than the web layer.

The Tauri global-shortcut plugin (v2) was investigated but only supports pre-registered shortcuts, not dynamic key capture.

## Fix Approach

**Option 1: Backend key capture using Rust (Recommended)**
Use a Rust crate like `rdev` or `device_query` to capture raw keyboard input at the system level. This would:
- Capture all keys including `fn`
- Send captured key events to frontend via Tauri events
- Frontend only displays the captured key, doesn't do the capturing

**Option 2: Accept platform limitation**
Document that `fn` key cannot be captured and update UI to indicate this limitation. Most users don't need `fn` as a hotkey modifier.

**Option 3: macOS-specific Accessibility API**
Use macOS Accessibility APIs (CGEventTap) via Rust bindings to capture all keyboard events. Requires accessibility permissions.

## Acceptance Criteria

- [ ] Bug no longer reproducible (if Option 1 or 3)
- [ ] OR: Clear UI indication that fn key is not supported (if Option 2)
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression
- [ ] Related specs/features not broken

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Press fn key alone | Key captured and displayed | [ ] |
| Press fn + letter key | Both fn and letter captured | [ ] |
| Press fn + F1 | fn+F1 captured (not just F1) | [ ] |
| Backend event sent to frontend | Frontend displays correct key | [ ] |

## Integration Points

- `src/pages/components/ShortcutEditor.tsx` - Frontend display
- `src-tauri/src/commands/mod.rs` - Backend key capture commands
- Tauri event system for backend→frontend communication
- Potentially: macOS Accessibility permissions

## Integration Test

E2E test: Open shortcut recorder, press fn+key, verify the combination is captured and can be saved as a working global hotkey.

## Research Notes

- Tauri global-shortcut plugin: Only supports pre-registered shortcuts, not dynamic capture
- `rdev` crate: Cross-platform input capture, may work for this use case
- `device_query` crate: Simpler API but may have limitations
- macOS CGEventTap: Most comprehensive but requires accessibility permissions

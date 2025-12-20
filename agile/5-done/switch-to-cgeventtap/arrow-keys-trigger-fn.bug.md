---
status: completed
severity: critical
origin: manual
created: 2025-12-20
completed: 2025-12-20
parent_feature: "switch-to-cgeventtap"
parent_spec: null
review_round: 1
---

# Bug: Arrow keys incorrectly trigger Fn modifier

**Created:** 2025-12-20
**Severity:** Major

## Problem Description

When the Fn key is set as the recording hotkey, pressing any arrow key is incorrectly detected as an Fn key press. The frontend displays "fn→" (or similar) and the backend registers it as Fn, causing the recording to start/stop unexpectedly.

**Expected:** Arrow keys should be recognized as distinct keys (Up, Down, Left, Right) and not trigger the Fn hotkey.

**Actual:** Arrow keys are misidentified as Fn key presses, triggering the recording hotkey.

## Steps to Reproduce

1. Set Fn as the recording hotkey
2. Press any arrow key (Up, Down, Left, or Right)
3. **Expected:** Nothing happens (arrow key is not Fn)
4. **Actual:** Recording starts/stops because arrow key is detected as Fn

## Root Cause

On macOS, the `CG_EVENT_FLAG_MASK_SECONDARY_FN` flag (0x00800000) is set for arrow keys even when Fn is NOT pressed. This is because arrow keys on compact MacBook keyboards are "virtual" keys with secondary functions (Fn+Up = Page Up, etc.).

**Two code paths contribute:**

1. **cgeventtap.rs:434** - Extracts `fn_key` from event flags:
   ```rust
   let fn_key = (flags_raw & CG_EVENT_FLAG_MASK_SECONDARY_FN) != 0;
   ```
   This sets `fn_key=true` for arrow keys because macOS sets the flag.

2. **cgeventtap_backend.rs:179-183** - Modifier-only matching always returns true:
   ```rust
   (None, _) => {
       // Modifier-only shortcut - match when the modifier key itself is pressed
       true  // <-- Matches ANY key when modifiers match!
   }
   ```

For "fn" modifier-only hotkey: arrow keys have `fn_key=true` in flags, and since `key_name` is `None`, arrow keys incorrectly trigger the hotkey.

## Fix Approach

For modifier-only shortcuts, require the actual modifier key to be pressed, not just any key with the modifier flag set.

**Option A: Check key_name in modifier-only matching**
```rust
(None, _) => {
    // For modifier-only, verify the pressed key IS a modifier
    // For fn: event.key_name must be "fn"
    if spec.fn_key && !spec.command && !spec.control && !spec.alt && !spec.shift {
        event.key_name == "fn"
    } else {
        is_modifier_key(&event.key_name)
    }
}
```

**Option B: Track fn_key separately from fn flag**
Distinguish between "fn key pressed" (key code 63/179) and "fn flag set" (which includes arrow keys) by checking key code in `handle_cg_event_inner`.

## Acceptance Criteria

- [ ] Bug no longer reproducible (needs manual testing)
- [x] Root cause addressed (not just symptoms)
- [x] Tests added to prevent regression
- [ ] Related specs/features not broken (needs manual testing)

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Press Up arrow with Fn as hotkey | Up arrow detected, not Fn | [x] |
| Press Down arrow with Fn as hotkey | Down arrow detected, not Fn | [x] |
| Press Left arrow with Fn as hotkey | Left arrow detected, not Fn | [x] |
| Press Right arrow with Fn as hotkey | Right arrow detected, not Fn | [x] |
| Press Fn key with Fn as hotkey | Fn detected, recording toggles | [x] |

**Unit tests added:**
- `test_matches_shortcut_fn_only_with_fn_key` - fn key triggers fn-only shortcut
- `test_matches_shortcut_fn_only_rejects_arrow_keys` - arrow keys don't trigger fn-only
- `test_matches_shortcut_command_only` - Command key triggers Command-only
- `test_matches_shortcut_modifier_only_rejects_regular_keys` - regular keys don't trigger modifier-only

## Integration Points

- CGEventTap key code handling (backend)
- Key-to-name mapping logic
- Hotkey matching/comparison logic
- Frontend display of captured keys

## Integration Test

Verify that with Fn set as hotkey, arrow keys do not trigger recording and are displayed correctly in the UI.

## Review

**Verdict: APPROVED**

### Fix Summary
Added `is_modifier_key_event()` helper function in `cgeventtap_backend.rs` that verifies the pressed key is actually a modifier key (not just any key with modifier flags set). This correctly distinguishes between the Fn key itself and arrow keys that have the Fn flag set on macOS.

### Code Quality
- Fix is minimal and targeted
- 4 unit tests added covering the bug scenario
- All existing tests still pass

### Risk Assessment
- Low risk - change is isolated to modifier-only shortcut matching
- No changes to event capture or other hotkey functionality

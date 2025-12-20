---
status: completed
created: 2025-12-20
completed: 2025-12-20
dependencies: []
review_round: 1
---

# Spec: Add missing Function to fn conversion in backendToDisplay

## Description

The `backendToDisplay()` function in `GeneralSettings.tsx` converts backend shortcut format (e.g., "CmdOrControl+Shift+R") to display symbols (e.g., "⌘⇧R"). Currently it's missing the conversion for "Function" → "fn", causing saved Fn hotkeys to display as literal "FunctionR" text after app restart instead of the proper "fn" symbol.

## Acceptance Criteria

- [ ] `backendToDisplay()` converts "Function" to "fn" in the shortcut string
- [ ] After app restart, a saved Fn+key hotkey displays correctly as "fn" prefix
- [ ] Existing conversions (CmdOrControl, Ctrl, Alt, Shift) still work correctly

## Test Cases

- [ ] Input: "Function+R" → Output: "fnR"
- [ ] Input: "Function+CmdOrControl+R" → Output: "fn⌘R"
- [ ] Input: "CmdOrControl+Shift+R" → Output: "⌘⇧R" (no regression)

## Dependencies

None

## Preconditions

- App has hotkey persistence working (saves to settings.json)
- The bug can be reproduced by setting Fn hotkey and restarting

## Implementation Notes

**File:** `src/pages/components/GeneralSettings.tsx:12-19`

Add `.replace(/Function/gi, "fn")` to the `backendToDisplay()` function chain.

```typescript
function backendToDisplay(shortcut: string): string {
  return shortcut
    .replace(/Function/gi, "fn")  // Add this line
    .replace(/CmdOrControl/gi, "⌘")
    .replace(/Ctrl/gi, "⌃")
    .replace(/Alt/gi, "⌥")
    .replace(/Shift/gi, "⇧")
    .replace(/\+/g, "");
}
```

## Related Specs

- `fix-backend-hotkey-loading.spec.md` - Fixes the backend hotkey registration issue

## Integration Points

- Production call site: `src/pages/components/GeneralSettings.tsx:12-19`
- Connects to: Settings loading flow, `get_recording_shortcut()` command

## Integration Test

- Test location: Manual test - set Fn hotkey, restart app, verify display
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `backendToDisplay()` converts "Function" to "fn" in the shortcut string | PASS | src/pages/components/GeneralSettings.tsx:15 - `.replace(/Function/gi, "fn")` added |
| After app restart, a saved Fn+key hotkey displays correctly as "fn" prefix | PASS | The conversion is applied in the useEffect on mount (line 38-46) when calling `get_recording_shortcut` |
| Existing conversions (CmdOrControl, Ctrl, Alt, Shift) still work correctly | PASS | All existing .replace() calls preserved at lines 16-20, order maintained |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Input: "Function+R" -> Output: "fnR" | PASS | Verified by code inspection - regex /Function/gi correctly matches |
| Input: "Function+CmdOrControl+R" -> Output: "fn⌘R" | PASS | Verified by code inspection - replacements chain correctly |
| Input: "CmdOrControl+Shift+R" -> Output: "⌘⇧R" (no regression) | PASS | Existing tests pass (267 tests, 31 test files) |

### Pre-Review Gate Results

```
Build Warning Check: No warnings found
Command Registration Check: N/A (frontend-only change)
Event Subscription Check: N/A (no new events)
Deferral Check: No deferrals found in GeneralSettings.tsx
```

### Data Flow Verification

```
[App Restart / Settings Mount]
     |
     v
[useEffect] src/pages/components/GeneralSettings.tsx:38-46
     | invoke("get_recording_shortcut")
     v
[Command] src-tauri/src/commands/mod.rs:868-877
     | Returns backend format (e.g., "Function+R")
     v
[backendToDisplay()] src/pages/components/GeneralSettings.tsx:13-21
     | Converts "Function" -> "fn", "CmdOrControl" -> "⌘", etc.
     v
[setCurrentShortcut()] src/pages/components/GeneralSettings.tsx:41
     |
     v
[UI Display] <kbd>{currentShortcut}</kbd> at line 137
```

### Code Quality

**Strengths:**
- Minimal, focused change - single line addition
- Consistent with existing pattern (regex replace chain)
- Case-insensitive matching handles edge cases (Function/function/FUNCTION)
- Correct placement at start of chain (before CmdOrControl which would not conflict)

**Concerns:**
- None identified

### Verdict

**APPROVED** - The implementation correctly adds the missing "Function" to "fn" conversion in the `backendToDisplay()` function. The change is minimal, follows the existing pattern, and is properly wired up in the production code path. All 267 tests pass. The data flow from backend command to UI display is complete and correct.

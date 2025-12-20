---
status: completed
created: 2025-12-19
completed: 2025-12-19
dependencies:
  - replace-iokit-hid
---

# Spec: Frontend shortcut display and recording UI updates

## Description

Update the frontend shortcut recording and display components to support all the new key types from the expanded backend. This includes updating TypeScript types, adding media key display, supporting modifier-only hotkeys, and adding a left/right modifier toggle.

## Acceptance Criteria

- [ ] CapturedKeyEvent TypeScript interface updated with new fields
- [ ] Modifier-only hotkeys allowed (just pressing Command is valid)
- [ ] Media keys displayed with human-readable names/symbols
- [ ] Left/Right modifier distinction shown when enabled
- [ ] User toggle to treat left/right modifiers as same
- [ ] All new key types display correctly in ShortcutEditor
- [ ] Shortcut display in GeneralSettings shows all key types correctly

## Test Cases

- [ ] Press "A" alone → displays "A"
- [ ] Press "Space" alone → displays "Space"
- [ ] Press Command alone → displays "⌘" (not ignored)
- [ ] Press fn alone → displays "fn"
- [ ] Press Volume Up → displays "🔊" or "Volume Up"
- [ ] Press Play/Pause → displays "⏯" or "Play/Pause"
- [ ] Press Left-Command → displays "L⌘" or "⌘" based on toggle
- [ ] Press Cmd+Shift+A → displays "⌘⇧A"
- [ ] Toggle "Distinguish L/R" off → Left-Cmd shows as "⌘"
- [ ] Toggle "Distinguish L/R" on → Left-Cmd shows as "L⌘"

## Dependencies

- replace-iokit-hid - backend must emit new event structure

## Preconditions

- Backend CapturedKeyEvent struct updated with new fields
- Backend emitting correct events for all key types

## Implementation Notes

### Files to Modify

1. **`src/pages/components/ShortcutEditor.tsx`**
   - Update CapturedKeyEvent interface (lines 15-25)
   - Update formatBackendKeyForDisplay (lines 28-58)
   - Remove modifier-only filtering (lines 205-207)
   - Add media key display mapping

2. **`src/pages/components/GeneralSettings.tsx`**
   - Add left/right modifier toggle setting
   - Update backendToDisplay function

### Updated TypeScript Interface
```typescript
interface CapturedKeyEvent {
  key_code: number;
  key_name: string;
  fn_key: boolean;
  command: boolean;
  command_left: boolean;   // NEW
  command_right: boolean;  // NEW
  control: boolean;
  control_left: boolean;   // NEW
  control_right: boolean;  // NEW
  alt: boolean;
  alt_left: boolean;       // NEW
  alt_right: boolean;      // NEW
  shift: boolean;
  shift_left: boolean;     // NEW
  shift_right: boolean;    // NEW
  pressed: boolean;
  is_media_key: boolean;   // NEW
}
```

### Media Key Display Mapping
```typescript
const mediaKeyMap: Record<string, string> = {
  VolumeUp: "🔊",
  VolumeDown: "🔉",
  Mute: "🔇",
  BrightnessUp: "🔆",
  BrightnessDown: "🔅",
  PlayPause: "⏯",
  NextTrack: "⏭",
  PreviousTrack: "⏮",
  FastForward: "⏩",
  Rewind: "⏪",
  KeyboardBrightnessUp: "🔆⌨",
  KeyboardBrightnessDown: "🔅⌨",
};
```

### Left/Right Display Logic
```typescript
function formatModifier(
  isPressed: boolean,
  isLeft: boolean,
  isRight: boolean,
  symbol: string,
  distinguishLeftRight: boolean
): string {
  if (!isPressed) return "";
  if (!distinguishLeftRight) return symbol;
  if (isLeft && !isRight) return `L${symbol}`;
  if (isRight && !isLeft) return `R${symbol}`;
  return symbol; // Both or neither - just show symbol
}
```

### Modifier-Only Hotkey Support
Remove or modify this code in ShortcutEditor.tsx (around line 205):
```typescript
// BEFORE: Ignores modifier-only
if (!capturedEvent.key_name ||
    ["Command", "Control", "Alt", "Shift", "fn"].includes(capturedEvent.key_name)) {
  return;
}

// AFTER: Allow modifier-only as valid hotkeys
// (no filtering needed, let user set just "Command" as hotkey)
```

### New Settings Toggle
Add to GeneralSettings.tsx:
```tsx
<div className="setting-row">
  <label>Distinguish Left/Right Modifiers</label>
  <Switch
    checked={settings.distinguishLeftRight}
    onChange={(value) => updateSetting("distinguishLeftRight", value)}
  />
  <span className="setting-description">
    When enabled, Left-Command and Right-Command are treated as different keys
  </span>
</div>
```

## Related Specs

- replace-iokit-hid.spec.md - provides the backend event structure
- integration-test.spec.md - end-to-end testing includes frontend

## Integration Points

- Production call site: `src/pages/components/ShortcutEditor.tsx`
- Connects to: Backend `shortcut_key_captured` events

## Integration Test

- Test location: `src/pages/components/ShortcutEditor.test.tsx`
- Verification: [ ] All test cases pass

## Review

**Reviewed:** 2025-12-19
**Reviewer:** Claude (Round 2)

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| CapturedKeyEvent TypeScript interface updated with new fields | PASS | ShortcutEditor.tsx:16-34 - Interface includes all new fields: command_left/right, control_left/right, alt_left/right, shift_left/right, is_media_key |
| Modifier-only hotkeys allowed (just pressing Command is valid) | PASS | ShortcutEditor.tsx:136-144 - isValidHotkey function accepts modifier-only hotkeys. Line 265: modifier-only events are processed |
| Media keys displayed with human-readable names/symbols | PASS | ShortcutEditor.tsx:37-50 - mediaKeyMap with symbols. Lines 104-106: media key display logic |
| Left/Right modifier distinction shown when enabled | PASS | ShortcutEditor.tsx:266-267 - formatBackendKeyForDisplay called with distinguishLeftRight from settings. useSettings hook imported at line 6 |
| User toggle to treat left/right modifiers as same | PASS | GeneralSettings.tsx:173-180 - Toggle present. useSettings.ts:171-188 - State management implemented |
| All new key types display correctly in ShortcutEditor | PASS | ShortcutEditor.tsx:82-112 - formatBackendKeyForDisplay handles all key types including media keys, modifiers, and special keys |
| Shortcut display in GeneralSettings shows all key types correctly | PASS | GeneralSettings.tsx:135-136 - Displays currentShortcut using backendToDisplay |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Press "A" alone → displays "A" | PASS | ShortcutEditor.test.tsx:539-582 |
| Press "Space" alone → displays "Space" | PASS | ShortcutEditor.test.tsx:428-471 |
| Press Command alone → displays "⌘" (not ignored) | PASS | ShortcutEditor.test.tsx:248-291 |
| Press fn alone → displays "fn" | PASS | ShortcutEditor.test.tsx:383-426 |
| Press Volume Up → displays "🔊" or "Volume Up" | PASS | ShortcutEditor.test.tsx:338-381 |
| Press Play/Pause → displays "⏯" or "Play/Pause" | PASS | ShortcutEditor.test.tsx:293-336 |
| Press Left-Command → displays "L⌘" or "⌘" based on toggle | PASS | ShortcutEditor.test.tsx:584-680 |
| Press Cmd+Shift+A → displays "⌘⇧A" | PASS | ShortcutEditor.test.tsx:473-516 |
| Toggle "Distinguish L/R" off → Left-Cmd shows as "⌘" | PASS | ShortcutEditor.test.tsx:584-632 |
| Toggle "Distinguish L/R" on → Left-Cmd shows as "L⌘" | PASS | ShortcutEditor.test.tsx:634-680 |

### Code Quality

**Strengths:**
- TypeScript interface matches backend structure exactly
- Comprehensive media key mapping with appropriate symbols
- Modifier-only hotkeys properly supported
- Clean separation of display vs backend format conversion
- Settings persistence implemented correctly with Tauri store
- Toggle UI properly integrated in GeneralSettings
- useSettings hook properly imported and used in ShortcutEditor
- distinguishLeftRight setting correctly wired to formatBackendKeyForDisplay
- Complete test coverage for all specified test cases including L/R toggle behavior

### Verdict

**APPROVED**

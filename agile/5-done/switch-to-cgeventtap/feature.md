# Feature: Complete Hotkey Capture - Any Key or Combination

**Created:** 2025-12-19
**Owner:** Michael
**Discovery Phase:** complete

## Description

Replace IOKit HID keyboard capture with a unified CGEventTap-based macOS keyboard manager that supports **any single key or combination** as a hotkey:
- Regular keys (A-Z, 0-9, symbols, special keys)
- Function keys (F1-F19)
- Media keys (volume, brightness, play/pause)
- Modifier keys alone or in combinations
- fn/Globe key
- Left vs Right modifier distinction (with user toggle)
- Only requires Accessibility permission (not Input Monitoring)
- Works alongside Karabiner-Elements

## BDD Scenarios

### User Persona
macOS user who wants to use fn key as a hotkey trigger, potentially with Karabiner-Elements running.

### Problem Statement
The current IOKit HID approach:
1. Requires Input Monitoring permission (complex to grant)
2. Gets blocked by Karabiner-Elements' exclusive HID access
3. Uses a different API than keyboard generation (CGEvent)

Apps like Wispr Flow work with only Accessibility permission by using CGEventTap.

```gherkin
Feature: CGEventTap Keyboard Manager

  Scenario: Detect fn key press with Accessibility permission
    Given the app has Accessibility permission
    And the user has Karabiner-Elements running
    When the user presses the fn key alone
    Then the app detects the fn key press
    And emits a keyboard event with fn_key=true

  Scenario: Detect fn key + other key combo
    Given the app has Accessibility permission
    When the user presses fn + F
    Then the app detects both keys
    And emits an event with fn_key=true and key_name="F"

  Scenario: Request Accessibility permission
    Given the app does not have Accessibility permission
    When keyboard capture starts
    Then the app prompts for Accessibility permission
    And provides guidance to enable it in System Settings

  Scenario: Detect modifier key combinations
    Given the app has Accessibility permission
    When the user presses Command+Shift+R
    Then the app detects all modifiers correctly
    And emits an event with command=true, shift=true, key_name="R"
```

### Out of Scope
- Replacing Tauri's global_shortcut plugin for hotkey registration (keep existing)
- Windows/Linux keyboard handling (this is macOS-specific)
- Raw HID report access (not needed for shortcut recording)

### Assumptions
- macOS 10.15+ (Catalina or later)
- CGEventTap FlagsChanged events include fn key state
- Accessibility permission is sufficient for keyboard event observation

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

**Key Types:**
- [ ] Regular keys (A-Z, 0-9, symbols)
- [ ] Special keys (Space, Tab, Enter, Escape, arrows, Delete)
- [ ] Function keys (F1-F19)
- [ ] fn/Globe key (alone and in combos)
- [ ] Modifier keys (fn, cmd, ctrl, alt, shift)
- [ ] Media keys (volume, brightness, play/pause)
- [ ] Numpad keys (distinct from regular numbers)

**Modifier Handling:**
- [ ] Left/Right modifier distinction (Left-Cmd vs Right-Cmd)
- [ ] User toggle to treat left/right as same
- [ ] Modifier-only hotkeys (just pressing Command)

**Compatibility:**
- [ ] Works with Karabiner-Elements running
- [ ] Only requires Accessibility permission (not Input Monitoring)
- [ ] Permission check/request flow works
- [ ] Existing shortcut recording functionality preserved

## Definition of Done

- [x] All specs completed
- [x] Technical guidance finalized
- [x] Code reviewed and approved
- [x] Tests written and passing

## Feature Review

**Verdict: APPROVED_FOR_DONE**

### Summary
Successfully replaced IOKit HID keyboard capture with CGEventTap-based implementation. The new backend supports:
- fn/Globe key detection (alone and in combos)
- Media keys (volume, brightness, play/pause)
- All regular keys and modifiers
- Left/right modifier distinction
- Only requires Accessibility permission (not Input Monitoring)
- Works alongside Karabiner-Elements

### Specs Completed (7/7)
1. accessibility-permission - Permission check/request flow
2. cgeventtap-core - Core CGEventTap keyboard event capture
3. cgeventtap-hotkey-backend - Hotkey backend for fn key support
4. frontend-shortcut-display - UI updates for shortcut display
5. integration-test - Manual integration testing
6. media-key-capture - Media key capture via NSSystemDefined
7. replace-iokit-hid - Replace IOKit HID with CGEventTap

### Bugs Fixed (1/1)
- arrow-keys-trigger-fn - Fixed arrow keys incorrectly triggering fn hotkey

### Quality
- Unit tests added for all new functionality
- All tests passing
- Code reviewed per spec

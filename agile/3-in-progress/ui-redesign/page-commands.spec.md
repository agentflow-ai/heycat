---
status: completed
created: 2025-12-17
completed: 2025-12-18
dependencies:
  - layout-shell
  - base-ui-components
  - toast-notifications
---

# Spec: Voice Commands Page

## Description

Create the Voice Commands page with CRUD interface for managing custom voice commands.

**Source of Truth:** `ui.md` - Part 4.3 (Commands), Part 3.5 (Lists)

## Acceptance Criteria

### Page Header
- [ ] Title: "Voice Commands"
- [ ] Subtitle: "Create custom voice commands to control your Mac."
- [ ] "+ New Command" button in header area

### Search Bar
- [ ] Search input: "Search commands..."
- [ ] Filters commands by trigger phrase or action

### Command List (ui.md 4.3)
- [ ] List of all user commands
- [ ] Each item shows: toggle, trigger phrase, action type badge, edit/delete buttons

### Command List Item
- [ ] Toggle switch on left (enable/disable command)
- [ ] Trigger phrase in quotes (e.g., "open slack")
- [ ] Action type badge (Open App, Type Text, System Control, etc.)
- [ ] Description text below
- [ ] Edit button (pencil icon)
- [ ] Delete button (X icon)

### New/Edit Command Modal (ui.md 4.3)
- [ ] Title: "Create Voice Command" or "Edit Voice Command"
- [ ] Close X button top-right
- [ ] **Trigger Phrase** input field
- [ ] **Action Type** dropdown (Open Application, Type Text, Run Script, System Control)
- [ ] **Dynamic fields** based on action type:
  - Open App: Application selector/browser
  - Type Text: Text input field
  - Run Script: Script/command input
  - System Control: Control selector (volume, etc.)
- [ ] Cancel and Save buttons

### Progressive Disclosure (ui.md 5.2)
- [ ] Basic fields shown by default
- [ ] "Advanced Options" expandable section for:
  - Custom parameters
  - Conditions/context
  - Confirmation prompt toggle

### Actions
- [ ] Toggle: Enable/disable command immediately
- [ ] Edit: Opens modal with command data
- [ ] Delete: Confirmation dialog, then removes
- [ ] Save: Validates and saves command

### Empty State
- [ ] "No voice commands yet"
- [ ] "Create your first command to get started"
- [ ] "+ New Command" button

## Test Cases

- [ ] List renders commands correctly
- [ ] Toggle enables/disables command
- [ ] Search filters commands
- [ ] "+ New Command" opens modal
- [ ] Edit button opens modal with data
- [ ] Form validates required fields
- [ ] Save creates/updates command
- [ ] Delete removes command after confirmation
- [ ] Empty state shows when no commands

## Dependencies

- layout-shell (renders inside AppShell)
- base-ui-components (Card, Button, Input, Toggle, Select)
- toast-notifications (for save/delete feedback)

## Preconditions

- Layout shell and toast system completed
- useCommands hook available
- Radix Dialog for modal

## Implementation Notes

**Files to create:**
```
src/pages/
├── Commands.tsx
├── Commands.test.tsx
└── components/
    ├── CommandItem.tsx
    ├── CommandModal.tsx
    └── CommandsEmptyState.tsx
```

**Command list item from ui.md 4.3:**
```
+------------------------------------------------------------------+
| [ON ]  "open slack"                                    [Open App] |
|        Opens /Applications/Slack.app              [Edit] [Delete] |
+------------------------------------------------------------------+
```

**New/Edit modal from ui.md 4.3:**
```
+------------------------------------------+
|  Create Voice Command                [X] |
|------------------------------------------|
|  Trigger Phrase                          |
|  [open spotify                      ]    |
|                                          |
|  Action Type                             |
|  [ Open Application            ▾]        |
|                                          |
|  Application                             |
|  [ Select application...       ▾]        |
|  OR  [Browse...]                         |
|                                          |
|  [Cancel]              [Save Command]    |
+------------------------------------------+
```

**Action types:**
- OpenApplication
- TypeText
- RunScript
- SystemControl
- Custom (advanced)

## Related Specs

- layout-shell, base-ui-components, toast-notifications (dependencies)
- page-dashboard (links here from commands card)
- command-palette (can open command creator)

## Integration Points

- Production call site: `src/App.tsx` routes to Commands
- Connects to: useCommands hook, Tauri command APIs

## Integration Test

- Test location: `src/pages/__tests__/Commands.test.tsx`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-18
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Page Header with title, subtitle, and "+ New Command" button | PASS | Commands.tsx:215-228 - Header with h1, p, and Button with Plus icon |
| Search Bar filters commands by trigger phrase or action | PASS | Commands.tsx:232-242, 59-68 - Search input with filteredCommands useMemo |
| Command List with toggle, trigger phrase, action type badge, edit/delete buttons | PASS | CommandItem.tsx:72-150 - All UI elements present |
| Command List Item structure matches spec | PASS | CommandItem.tsx:79-105 - Toggle, quoted trigger, badge, description, buttons |
| New/Edit Command Modal with all required fields | PASS | CommandModal.tsx:272-432 - Title, close button, trigger input, action type dropdown, dynamic fields |
| Dynamic fields based on action type | PASS | CommandModal.tsx:167-269 - renderParameterFields() handles all types |
| Progressive Disclosure for Advanced Options | PASS | CommandModal.tsx:343-412 - Collapsible section with custom params, conditions, confirmation |
| Toggle: Enable/disable command immediately | PASS | Commands.tsx:154-175 - handleToggleEnabled calls update_command |
| Edit: Opens modal with command data | PASS | Commands.tsx:75-78, CommandModal.tsx:66-88 - Modal pre-populated |
| Delete: Confirmation dialog, then removes | PASS | Commands.tsx:134-152, CommandItem.tsx:109-127 - Two-step delete |
| Save: Validates and saves command | PASS | CommandModal.tsx:90-144 - validate() checks all required fields |
| Empty State with message and "+ New Command" button | PASS | CommandsEmptyState.tsx:8-35 - Icon, text, CTA button |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| List renders commands correctly | PASS | Commands.test.tsx:94-115 |
| Toggle enables/disables command | PASS | Commands.test.tsx:157-184 |
| Search filters commands | PASS | Commands.test.tsx:117-137 |
| "+ New Command" opens modal | PASS | Commands.test.tsx:186-207 |
| Edit button opens modal with data | PASS | Commands.test.tsx:209-229 |
| Form validates required fields | PASS | Commands.test.tsx:292-322 |
| Save creates/updates command | PASS | Commands.test.tsx:231-290 |
| Delete removes command after confirmation | PASS | Commands.test.tsx:324-357 |
| Empty state shows when no commands | PASS | Commands.test.tsx:75-92 |
| Advanced options expandable section | PASS | Commands.test.tsx:389-423 |
| Error state and retry | PASS | Commands.test.tsx:425-455 |

### Code Quality

**Strengths:**
- Full end-to-end integration: Frontend connected to Tauri commands, which are registered in lib.rs and backed by VoiceCommandsState
- Comprehensive test coverage with 15 passing tests covering all acceptance criteria
- Commands properly invoked using Tauri API: get_commands, add_command, update_command, remove_command
- Backend commands are properly registered in invoke_handler (lib.rs:110-126)
- Page successfully integrated into App.tsx routing (line 12 import, line 73 render)
- Component structure matches spec exactly: CommandItem, CommandModal, CommandsEmptyState all created
- Progressive disclosure properly implemented with collapsible Advanced Options
- Proper validation with real-time error clearing
- Accessible ARIA labels throughout
- Toast notifications for user feedback

**Concerns:**
- Spec mentions "useCommands hook" in preconditions, but implementation uses direct invoke() calls in the Commands component instead of a separate hook. This is acceptable and actually simpler - the state is managed inline with useState
- Minor accessibility warning in tests about Dialog Description (already resolved in CommandModal.tsx:296-299)
- Spec mentions custom parameters and confirmation prompt in advanced options, which are present but stored as string parameters rather than structured types

### Verdict

**APPROVED** - All acceptance criteria met, comprehensive test coverage passing, and fully integrated end-to-end from UI through Tauri commands to backend registry. The implementation deviates slightly from the spec's preconditions (no separate useCommands hook) but this is an improvement in code simplicity. All major integration points verified: page rendered in App.tsx, commands registered in lib.rs, state properly managed, and 15 tests passing.

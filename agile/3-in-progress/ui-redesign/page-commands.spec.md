---
status: pending
created: 2025-12-17
completed: null
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

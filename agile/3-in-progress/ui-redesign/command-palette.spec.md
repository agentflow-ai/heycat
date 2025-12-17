---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - design-system-foundation
  - base-ui-components
  - layout-shell
---

# Spec: Command Palette

## Description

Implement the ⌘K command palette overlay for quick access to app actions and navigation.

**Source of Truth:** `ui.md` - Part 3.6 (Command Palette), Part 5.1 (Command Palette Interaction)

## Acceptance Criteria

### Trigger (ui.md 5.1)
- [ ] Opens with ⌘K keyboard shortcut (global)
- [ ] Opens when clicking the ⌘K pill in header
- [ ] Closes with Escape key
- [ ] Closes when clicking outside

### Layout (ui.md 3.6)
- [ ] Top-center positioned (20% from top of viewport)
- [ ] Width: 560px
- [ ] Dark overlay backdrop (50% black)
- [ ] Rounded corners, shadow elevation
- [ ] Search input with icon at top
- [ ] Scrollable command list below

### Search & Filtering
- [ ] Focus immediately in search input on open
- [ ] Fuzzy search filtering as user types
- [ ] Show "No results" when no matches
- [ ] Recent/frequent commands shown first when empty

### Command List
- [ ] Commands grouped by category (Actions, Navigation, Settings, Help)
- [ ] Each item shows: icon, label, keyboard shortcut (if any)
- [ ] Highlighted item on hover/keyboard focus

### Keyboard Navigation (ui.md 5.1)
- [ ] ↑↓ arrows to navigate items
- [ ] Enter to select/execute
- [ ] Escape to close
- [ ] Type to filter

### Commands to Include
**Actions:**
- Start Recording (⌘⇧R)
- Stop Recording (Esc)
- Toggle Listening

**Navigation:**
- Go to Dashboard
- Go to Recordings
- Go to Commands
- Go to Settings

**Settings:**
- Change Audio Device
- Download Model

**Help:**
- View Shortcuts
- About HeyCat

## Test Cases

- [ ] ⌘K opens the palette
- [ ] Search input is focused on open
- [ ] Typing filters commands
- [ ] Arrow keys navigate the list
- [ ] Enter executes selected command
- [ ] Escape closes the palette
- [ ] Click outside closes the palette
- [ ] Commands execute their actions correctly

## Dependencies

- design-system-foundation (styling)
- base-ui-components (Input component)
- layout-shell (integrates into header)

## Preconditions

- Layout shell completed
- Navigation routing set up
- App actions available (startRecording, etc.)

## Implementation Notes

**Files to create:**
```
src/components/overlays/
├── CommandPalette.tsx
├── CommandPalette.test.tsx
├── useCommandPalette.ts    # Hook for open/close state
└── commands.ts             # Command registry
```

**Layout from ui.md 3.6:**
```
+------------------------------------------+
|  [Search icon] Search commands...        |
+------------------------------------------+
|  > Start Recording          ⌘⇧R         |
|  > Stop Recording           Esc          |
|  > Open Settings            ⌘,           |
|  > Download Model                        |
|  > View Recordings                       |
+------------------------------------------+
```

**Consider using:**
- cmdk library (https://cmdk.paco.me/) for command palette primitives
- Or build custom with Radix Dialog + custom filtering

**Command registry structure:**
```ts
interface Command {
  id: string;
  label: string;
  icon: LucideIcon;
  shortcut?: string;
  category: 'actions' | 'navigation' | 'settings' | 'help';
  action: () => void;
}
```

## Related Specs

- design-system-foundation, base-ui-components, layout-shell (dependencies)
- All page specs (navigation commands)

## Integration Points

- Production call site: `src/components/layout/Header.tsx` (trigger)
- Connects to: React Router navigation, recording hooks, settings

## Integration Test

- Test location: `src/components/overlays/__tests__/CommandPalette.test.tsx`
- Verification: [ ] Integration test passes

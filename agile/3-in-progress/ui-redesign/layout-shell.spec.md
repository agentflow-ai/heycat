---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - design-system-foundation
  - base-ui-components
---

# Spec: Layout Shell

## Description

Implement the main application layout structure including header, sidebar navigation, main content area, and context footer bar.

**Source of Truth:** `ui.md` - Part 2: Layout Architecture (sections 2.1-2.5)

## Acceptance Criteria

### Main Layout Structure (ui.md 2.1)
- [ ] App shell with header (48px), sidebar (220px), content area, footer (44px)
- [ ] Responsive layout using CSS Grid or Flexbox
- [ ] Window has warm orange ambient glow effect (ui.md 1.5)

### Header Bar (ui.md 2.2)
- [ ] Left: HeyCat logo (cat icon + "HeyCat" text)
- [ ] Center: Status pill placeholder (actual states in separate spec)
- [ ] Right: Command palette trigger (⌘K pill), Settings gear icon, Help icon
- [ ] Height: 48px fixed

### Sidebar Navigation (ui.md 2.3)
- [ ] Width: 220px fixed
- [ ] Light cream background (`--heycat-cream`)
- [ ] Subtle inner shadow on right edge
- [ ] Navigation items: Dashboard, Recordings, Commands, Settings
- [ ] Active state: orange/cream background fill
- [ ] Icons for each nav item (Lucide icons)

### Main Content Area (ui.md 2.4)
- [ ] Max-width: 900px centered
- [ ] Padding: 32px
- [ ] Clean white/cream background
- [ ] Accepts children for page content

### Context Footer Bar (ui.md 2.5)
- [ ] Height: 44px fixed
- [ ] Left: Current state description text
- [ ] Center: Audio level mini-meter placeholder
- [ ] Right: Quick action buttons area
- [ ] Decorative cat paw icon on right

## Test Cases

- [ ] Layout renders with correct dimensions
- [ ] Sidebar navigation items are clickable
- [ ] Active nav item shows highlighted state
- [ ] Header icons are accessible and clickable
- [ ] Content area scrolls independently
- [ ] Footer stays fixed at bottom

## Dependencies

- design-system-foundation (uses CSS tokens)
- base-ui-components (uses Button, icons)

## Preconditions

- Design system and base components completed
- Lucide React icons installed
- React Router or navigation state management ready

## Implementation Notes

**Files to create:**
```
src/components/layout/
├── AppShell.tsx        # Main wrapper
├── Header.tsx          # Top bar
├── Sidebar.tsx         # Left navigation
├── MainContent.tsx     # Content container
├── Footer.tsx          # Bottom bar
└── index.ts
```

**Layout from ui.md 2.1:**
```
+------------------------------------------------------------------+
|  [Logo] HeyCat           [Status Pill]      [⌘K] [Settings] [?] |  <- Header (48px)
+------------------------------------------------------------------+
|         |                                                        |
|  SIDE   |                    MAIN CONTENT                        |
|  BAR    |                       AREA                             |
| (220px) |                                                        |
|         +--------------------------------------------------------+
|         |  [Context Bar - shows current state, quick actions]    |  <- Footer (44px)
+---------+--------------------------------------------------------+
```

**Navigation items with icons:**
- Dashboard: LayoutDashboard
- Recordings: Mic
- Commands: MessageSquare
- Settings: Settings

## Related Specs

- design-system-foundation, base-ui-components (dependencies)
- ui-toggle (adds toggle to this layout)
- All page specs (render inside this layout)

## Integration Points

- Production call site: `src/App.tsx` (new UI mode)
- Connects to: All page components, status-pill-states, command-palette

## Integration Test

- Test location: `src/components/layout/__tests__/AppShell.test.tsx`
- Verification: [ ] Integration test passes

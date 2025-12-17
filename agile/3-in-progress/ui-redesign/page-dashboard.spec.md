---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - layout-shell
  - base-ui-components
  - status-pill-states
---

# Spec: Dashboard Page

## Description

Create the Dashboard (home) page with at-a-glance overview and quick actions for the HeyCat app.

**Source of Truth:** `ui.md` - Part 4.1 (Dashboard)

## Acceptance Criteria

### Page Header
- [ ] Title: "Dashboard"
- [ ] Subtitle: "Welcome back! Here's your HeyCat status."

### Status Cards Row (ui.md 4.1)
- [ ] **Listening Card**: Toggle switch, status text ("'Hey Cat' ready" or "Off")
- [ ] **Recordings Card**: Count of total recordings, link to Recordings page
- [ ] **Commands Card**: Count of active commands, link to Commands page
- [ ] Cards in responsive grid (3 columns on desktop, stack on mobile)

### Quick Action Buttons
- [ ] "Start Recording" - Primary button, triggers recording
- [ ] "Train Command" - Secondary button, opens command creator
- [ ] "Download Model" - Secondary button (only if model not installed)
- [ ] Buttons disabled/hidden based on app state

### Recent Activity Section
- [ ] Header: "RECENT ACTIVITY"
- [ ] List of last 5 recordings
- [ ] Each item shows: play button, filename, date, transcription status badge
- [ ] "View all" link to Recordings page

### Empty States
- [ ] No recordings: Friendly message with guidance
- [ ] Model not downloaded: Prominent download prompt

### Live Data Integration
- [ ] Listening toggle connects to useListening hook
- [ ] Recording count from recordings list
- [ ] Commands count from commands list
- [ ] Recent recordings from recordings hook

## Test Cases

- [ ] Dashboard renders with all sections
- [ ] Status cards show correct counts
- [ ] Listening toggle works
- [ ] Quick action buttons trigger correct actions
- [ ] Recent activity shows recordings
- [ ] Empty state displays when no recordings
- [ ] Navigation links work

## Dependencies

- layout-shell (renders inside AppShell)
- base-ui-components (Card, Button, Toggle)
- status-pill-states (status information)

## Preconditions

- Layout shell completed
- Hooks available: useListening, useRecordings, useCommands

## Implementation Notes

**Files to create:**
```
src/pages/
├── Dashboard.tsx
├── Dashboard.test.tsx
└── index.ts
```

**Layout from ui.md 4.1:**
```
Dashboard
Welcome back! Here's your HeyCat status.

+------------------+  +------------------+  +------------------+
|  LISTENING       |  |  RECORDINGS      |  |  COMMANDS        |
|  [Toggle=====ON] |  |  12 recordings   |  |  8 active        |
|  'Hey Cat' ready.|  |                  |  |                  |
+------------------+  +------------------+  +------------------+

[  Start Recording  ]  [  Train Command  ]  [  Download Model  ]

RECENT ACTIVITY
+------------------------------------------------------------------+
| [Play] Recording_2024-01-15.wav     Sep 25, 2022   [Transcribed] |
| [Play] Recording_2024-01-14.wav     Aug 24, 2022      [Pending]  |
+------------------------------------------------------------------+
```

**Status card structure:**
```tsx
<Card>
  <CardHeader>LISTENING</CardHeader>
  <CardContent>
    <Toggle checked={isListening} onChange={toggleListening} />
    <span>{isListening ? "'Hey Cat' ready." : "Listening off"}</span>
  </CardContent>
</Card>
```

## Related Specs

- layout-shell, base-ui-components, status-pill-states (dependencies)
- page-recordings (linked from recent activity)
- page-commands (linked from commands card)

## Integration Points

- Production call site: `src/App.tsx` routes to Dashboard
- Connects to: useListening, useRecordings, useCommands, useRecording hooks

## Integration Test

- Test location: `src/pages/__tests__/Dashboard.test.tsx`
- Verification: [ ] Integration test passes

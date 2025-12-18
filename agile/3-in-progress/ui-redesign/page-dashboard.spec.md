---
status: in-progress
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

## Review

**Reviewed:** 2025-12-18
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Page Header (Title + Subtitle) | PASS | Dashboard.tsx:78-81 - renders "Dashboard" h1 and subtitle |
| Listening Card with Toggle | PASS | Dashboard.tsx:87-103 - Card with Toggle, status text based on isListening |
| Recordings Card with Count | PASS | Dashboard.tsx:106-128 - Shows recording count from backend, link to recordings |
| Commands Card with Count | PASS | Dashboard.tsx:131-153 - Shows commands count (placeholder=0), link to commands |
| Responsive Grid (3 columns) | PASS | Dashboard.tsx:85 - grid-cols-1 md:grid-cols-3 gap-4 |
| Start Recording Button | PASS | Dashboard.tsx:158 - Primary button triggering startRecording |
| Train Command Button | PASS | Dashboard.tsx:159-161 - Secondary button navigating to commands |
| Download Model Button | PASS | Dashboard.tsx:162-172 - Conditional render, secondary button with loading state |
| Recent Activity Header | PASS | Dashboard.tsx:178-179 - "RECENT ACTIVITY" uppercase header |
| Recent Activity List (Last 5) | PASS | Dashboard.tsx:71, 206-234 - slice(0,5), shows play, filename, date, status badge |
| View All Link | PASS | Dashboard.tsx:181-189 - Link to recordings page when recordings exist |
| Empty State (No Recordings) | PASS | Dashboard.tsx:196-204 - Friendly message with guidance |
| Empty State (Model Not Downloaded) | PASS | Dashboard.tsx:236-255 - Prominent download prompt card |
| Listening Toggle Integration | PASS | Dashboard.tsx:24-26,55-61 - useListening hook with deviceName |
| Recording Count Integration | PASS | Dashboard.tsx:40-53 - invoke("list_recordings") fetches from backend |
| Commands Count Integration | DEFERRED | Dashboard.tsx:37 - Hardcoded 0, useCommands hook doesn't exist yet |
| Recent Recordings Integration | PASS | Dashboard.tsx:40-53 - useRecording hook used for recordings data |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Dashboard renders with all sections | PASS | Dashboard.test.tsx:67-97 |
| Status cards show correct counts | PASS | Dashboard.test.tsx:99-122 |
| Listening toggle works | PASS | Dashboard.test.tsx:161-172 |
| Quick action buttons trigger correct actions | PASS | Dashboard.test.tsx:174-188 |
| Recent activity shows recordings | PASS | Dashboard.test.tsx:136-159 |
| Empty state displays when no recordings | PASS | Dashboard.test.tsx:124-134 |
| Navigation links work | PASS | Dashboard.test.tsx:190-229 |

### Code Quality

**Strengths:**
- Component properly uses all required hooks (useListening, useRecording, useMultiModelStatus, useSettings)
- Backend command list_recordings is properly registered and tested
- Tests pass (9/9) and follow TESTING.md guidelines focusing on user-visible behavior
- Proper error handling with silent fallback for recording fetch failures
- Accessibility features present (aria-labels, keyboard navigation)
- Responsive design implemented with Tailwind grid
- Loading states and empty states properly handled

**Concerns:**
- **CRITICAL: Dashboard component is not wired up to App.tsx** - Component exists and is exported from src/pages/index.ts but is never imported or rendered in App.tsx. In the new UI mode (line 60-77 of App.tsx), the AppShell renders placeholder text "New UI - Page content coming soon" instead of the Dashboard component. This means the Dashboard code exists but is completely unreachable from production.
- Commands count is hardcoded to 0 because useCommands hook doesn't exist - this is acceptable as a placeholder but noted as a deferral without a tracking spec reference

### Frontend-Only Integration Check

#### App Entry Point Verification
- Dashboard component is exported from src/pages/index.ts
- **FAIL:** Dashboard is NOT imported in src/App.tsx
- **FAIL:** Dashboard is NOT rendered in App.tsx - line 70 shows placeholder text instead

#### Data Flow Analysis

```
[UI Mode Toggle]
     |
     v
[App.tsx] mode === "new"
     |
     v
[AppShell] renders with navItem="dashboard"
     |
     v
[Placeholder Text] "New UI - Page content coming soon" ❌ SHOULD BE: <Dashboard onNavigate={setNavItem} />
```

**BROKEN LINK:** Dashboard component is never rendered in production code.

#### Production Call Sites

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| Dashboard | component | **MISSING** - Not imported in App.tsx | **NO** - TEST-ONLY |
| handleListeningToggle | fn | Dashboard.tsx:99 | NO (Dashboard unreachable) |
| handleStartRecording | fn | Dashboard.tsx:158 | NO (Dashboard unreachable) |
| handleDownloadModel | fn | Dashboard.tsx:68 | NO (Dashboard unreachable) |
| TranscriptionBadge | component | Dashboard.tsx:230 | NO (Dashboard unreachable) |

**All Dashboard code is currently TEST-ONLY because the component is never rendered in production.**

### Automated Check Results

#### Build Warning Check
```
No warnings found
```
✅ PASS

#### Backend Command Registration
```
list_recordings command is properly registered in src-tauri/src/lib.rs:265
```
✅ PASS

#### Deferrals Check
```
No TODO/FIXME/HACK comments found in Dashboard implementation
```
✅ PASS (Note: commands count placeholder is in code but not marked with TODO)

### Verdict

**NEEDS_WORK** - Dashboard component exists with full functionality and passing tests, but is completely disconnected from production code. The component is never imported or rendered in App.tsx, making all new code unreachable from the UI.

**What failed:** Question 1 & 2 from review.md - Code is not wired up end-to-end. Dashboard component would have zero production impact if deployed.

**Why it failed:** In App.tsx line 60-77, the new UI mode renders AppShell with placeholder text instead of the Dashboard component. Dashboard is exported but never imported.

**How to fix:**
1. Import Dashboard in App.tsx: `import { Dashboard } from "./pages";` (after line 18)
2. Replace placeholder content (line 70-72) with: `<Dashboard onNavigate={setNavItem} />`
3. Verify in browser that Dashboard renders when UI mode is toggled to "new"

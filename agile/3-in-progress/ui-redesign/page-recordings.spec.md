---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - layout-shell
  - base-ui-components
  - toast-notifications
---

# Spec: Recordings Page

## Description

Build the Recordings page with list view, search, filter, and detail expansion for managing voice recordings and transcriptions.

**Source of Truth:** `ui.md` - Part 4.2 (Recordings), Part 3.5 (Lists)

## Acceptance Criteria

### Page Header
- [ ] Title: "Recordings"
- [ ] Subtitle: "Manage your voice recordings and transcriptions."

### Search & Filter Bar
- [ ] Search input with placeholder "Search recordings..."
- [ ] Filter dropdown: All, Transcribed, Pending
- [ ] Sort dropdown: Newest, Oldest, Longest, Shortest

### Recording List (ui.md 3.5, 4.2)
- [ ] Virtualized list for performance (100+ recordings)
- [ ] Collapsed item shows: play button, filename, date, duration, size, status badge

### Recording Item - Collapsed State
- [ ] Play/pause button on left
- [ ] Filename (truncated if long)
- [ ] Metadata: date, duration, file size
- [ ] Status badge: "Transcribed" (green) or "Transcribe" (button)
- [ ] More menu (kebab icon) for additional actions

### Recording Item - Expanded State (ui.md 4.2)
- [ ] Click to expand/collapse (accordion style)
- [ ] Shows transcription text (or "No transcription" message)
- [ ] Action buttons: Copy Text, Open File, Delete
- [ ] Transcription text is scrollable if long

### Actions
- [ ] Play: Plays audio (inline player or system)
- [ ] Transcribe: Triggers transcription, shows progress
- [ ] Copy Text: Copies transcription to clipboard, shows toast
- [ ] Open File: Opens in system file manager
- [ ] Delete: Confirmation dialog, then removes

### Empty State (ui.md 4.2)
- [ ] Friendly illustration or icon
- [ ] "No recordings yet"
- [ ] "Press ⌘⇧R or say 'Hey Cat' to start"
- [ ] Primary button: "Start Recording"

### Loading States
- [ ] Skeleton loaders while fetching
- [ ] Transcription progress indicator

## Test Cases

- [ ] List renders recordings correctly
- [ ] Search filters recordings by name/content
- [ ] Filter dropdown works
- [ ] Sort changes order
- [ ] Click expands/collapses item
- [ ] Play button plays audio
- [ ] Copy button copies and shows toast
- [ ] Delete shows confirmation and removes
- [ ] Empty state shows when no recordings
- [ ] Virtualization works for large lists

## Dependencies

- layout-shell (renders inside AppShell)
- base-ui-components (Card, Button, Input)
- toast-notifications (for copy/delete feedback)

## Preconditions

- Layout shell and toast system completed
- useRecordings hook available
- Audio playback capability

## Implementation Notes

**Files to create:**
```
src/pages/
├── Recordings.tsx
├── Recordings.test.tsx
└── components/
    ├── RecordingItem.tsx
    ├── RecordingItemExpanded.tsx
    └── RecordingsEmptyState.tsx
```

**Collapsed item from ui.md 3.5:**
```
+------------------------------------------------------------------+
| [Play]  Recording_2024-01-15_143022.wav                          |
|         Sep 25, 2022 • 00:00:28 • 3.6 MB                         |
+------------------------------------------------------------------+
```

**Expanded item from ui.md 4.2:**
```
+------------------------------------------------------------------+
| [Play]  Recording_2024-01-15_143022.wav                          |
|         Sep 25, 2022 • 00:00:28 • 3.6 MB                         |
|------------------------------------------------------------------|
|  TRANSCRIPTION                                                    |
|  Hello, this is a test recording for the HeyCat application.     |
|  I'm testing the voice transcription feature.                    |
|                                                                   |
|  [Copy Text]  [Open File]  [Delete]                              |
+------------------------------------------------------------------+
```

**Virtualization:**
- Use react-window or @tanstack/react-virtual
- Row height: collapsed ~60px, expanded ~200px (variable)

## Related Specs

- layout-shell, base-ui-components, toast-notifications (dependencies)
- page-dashboard (links here from recent activity)

## Integration Points

- Production call site: `src/App.tsx` routes to Recordings
- Connects to: useRecordings, useTranscription hooks, file system APIs

## Integration Test

- Test location: `src/pages/__tests__/Recordings.test.tsx`
- Verification: [ ] Integration test passes

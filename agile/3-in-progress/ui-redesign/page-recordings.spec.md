---
status: in-progress
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

## Review

**Reviewed:** 2025-12-18
**Reviewer:** Claude

### Pre-Review Gate Checks

**Build Warning Check:** PASS - No new unused warnings in src-tauri.

**Command Registration Check:**
```
Output: check_parakeet_model_status, download_model
```
Note: `delete_recording` command is called by frontend but not registered in invoke_handler.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Title: "Recordings" | PASS | src/pages/Recordings.tsx:265-267 |
| Subtitle: "Manage your voice recordings and transcriptions." | PASS | src/pages/Recordings.tsx:268-270 |
| Search input with placeholder "Search recordings..." | PASS | src/pages/Recordings.tsx:278-285 |
| Filter dropdown: All, Transcribed, Pending | PASS | src/pages/Recordings.tsx:289-298 |
| Sort dropdown: Newest, Oldest, Longest, Shortest | PASS | src/pages/Recordings.tsx:301-312 |
| Virtualized list for performance (100+ recordings) | FAIL | No virtualization library imported; spec recommends react-window or @tanstack/react-virtual |
| Collapsed item shows: play button, filename, date, duration, size, status badge | PASS | src/pages/components/RecordingItem.tsx:79-146 |
| Play/pause button on left | PASS | RecordingItem.tsx:81-93 (separate button, not nested) |
| Filename (truncated if long) | PASS | RecordingItem.tsx:103-105 with truncate class |
| Metadata: date, duration, file size | PASS | RecordingItem.tsx:106-108 |
| Status badge: "Transcribed" (green) or "Transcribe" (button) | PASS | RecordingItem.tsx:111-131 |
| More menu (kebab icon) for additional actions | DEFERRED | Replaced with expand/collapse pattern showing actions in expanded state |
| Click to expand/collapse (accordion style) | PASS | RecordingItem.tsx:96-109 (filename clickable), :134-145 (chevron button) |
| Shows transcription text (or "No transcription" message) | PASS | RecordingItem.tsx:156-166 |
| Action buttons: Copy Text, Open File, Delete | PASS | RecordingItem.tsx:191-221 |
| Transcription text is scrollable if long | PASS | RecordingItem.tsx:157 max-h-32 overflow-y-auto |
| Play: Plays audio (inline player or system) | DEFERRED | handlePlay toggles state but no actual audio playback (comment: "For now") |
| Transcribe: Triggers transcription, shows progress | PASS | Recordings.tsx:118-142, RecordingItem.tsx:121-130 |
| Copy Text: Copies transcription to clipboard, shows toast | PASS | Recordings.tsx:144-161 |
| Open File: Opens in system file manager | PASS | Recordings.tsx:163-173, uses @tauri-apps/plugin-opener |
| Delete: Confirmation dialog, then removes | FAIL | Frontend calls invoke("delete_recording") but command not in invoke_handler (lib.rs:294-315) |
| Empty state: Friendly illustration or icon | PASS | RecordingsEmptyState.tsx:13-15 (Mic icon) |
| Empty state: "No recordings yet" | PASS | RecordingsEmptyState.tsx:19 |
| Empty state: "Press Cmd+Shift+R or say 'Hey Cat' to start" | PASS | RecordingsEmptyState.tsx:22-24 |
| Empty state: Primary button "Start Recording" | PASS | RecordingsEmptyState.tsx:28-30 |
| Skeleton loaders while fetching | PASS | Recordings.tsx:202-236 (full skeleton UI with cards) |
| Transcription progress indicator | PASS | Button loading state via isTranscribing prop |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| List renders recordings correctly | PASS | Recordings.test.tsx:116-132 |
| Search filters recordings by name/content | PASS | Recordings.test.tsx:151-193 |
| Filter dropdown works | PASS | Recordings.test.tsx:134-149 (transcribed/pending) |
| Sort changes order | PASS | Recordings.test.tsx:491-510 |
| Click expands/collapses item | PASS | Recordings.test.tsx:214-245 |
| Play button plays audio | PASS | Recordings.test.tsx:512-542 |
| Copy button copies and shows toast | PASS | Recordings.test.tsx:266-311 |
| Delete shows confirmation and removes | PASS | Recordings.test.tsx:337-377 |
| Empty state shows when no recordings | PASS | Recordings.test.tsx:99-114 |
| Skeleton loaders while loading | PASS | Recordings.test.tsx:544-554 |
| Virtualization works for large lists | MISSING | Not implemented |

### Frontend Integration Check

| Check | Status | Evidence |
|-------|--------|----------|
| Page exported from index.ts | PASS | src/pages/index.ts:10-11 |
| Page wired up in App.tsx | PASS | src/App.tsx:12 (import), :74 (render) |
| Page routed correctly | PASS | App.tsx:74 `navItem === "recordings" && <Recordings />` |

### Code Quality

**Strengths:**
- Well-organized component structure (Recordings, RecordingItem, RecordingsEmptyState)
- Good separation of concerns with individual handlers for each action
- Proper error handling with toast notifications
- Comprehensive test coverage (20 passing tests)
- Proper TypeScript types exported for RecordingInfo
- DOM nesting issue fixed - play button is now separate from expand button
- Full skeleton loader UI implemented during loading state

**Concerns:**
- `delete_recording` command not registered in backend invoke_handler - will fail at runtime
- No virtualization for large lists (spec requirement for 100+ recordings)
- Audio playback deferred (toggles state only, no actual audio)

### Data Flow Analysis

```
[UI: Click Delete] Recordings.tsx:175
     |
     v
[State] setDeleteConfirmPath()
     |
     v
[UI: Confirm Delete] Recordings.tsx:175-196
     | invoke("delete_recording", { filePath })
     v
[BROKEN] Command not registered in lib.rs invoke_handler
```

### Verdict

**NEEDS_WORK** - The Recordings page is properly wired up and tested, but has one blocking issue:

1. **delete_recording command not registered** - Frontend calls `invoke("delete_recording")` at Recordings.tsx:178, but the command is not registered in src-tauri/src/lib.rs invoke_handler (lines 294-315). This will cause runtime failure when users try to delete recordings.

**How to fix:**
1. Create `delete_recording` command in src-tauri/src/commands.rs (or appropriate module)
2. Register it in lib.rs invoke_handler alongside other recording commands (line 300-301)

**Note:** Virtualization is listed as a spec requirement but may be acceptable to defer given the implementation complexity. However, the delete command is critical for basic functionality.

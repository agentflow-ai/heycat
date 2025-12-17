---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - design-system-foundation
  - base-ui-components
  - layout-shell
  - ui-toggle
  - status-pill-states
  - command-palette
  - toast-notifications
  - page-dashboard
  - page-recordings
  - page-commands
  - page-settings
---

# Spec: Integration and Legacy Cleanup

## Description

Final integration of the new UI into the main app, removal of the dev toggle, and cleanup of all legacy components and styles. This spec completes the UI redesign.

**Note:** This is the final spec and depends on ALL other specs being completed.

## Acceptance Criteria

### Integration
- [ ] App.tsx uses new AppShell layout as default (no toggle)
- [ ] All routes point to new pages (Dashboard, Recordings, Commands, Settings)
- [ ] All existing hooks properly connected to new UI
- [ ] App state flows correctly through new components

### Remove Dev Toggle
- [ ] Delete UIToggle component
- [ ] Delete useUIMode hook
- [ ] Remove toggle-related code from App.tsx
- [ ] Remove localStorage key for UI mode

### Legacy Component Removal
- [ ] Delete `src/components/Sidebar/` directory
- [ ] Delete `src/components/RecordingsView/` directory
- [ ] Delete `src/components/CommandSettings/` directory
- [ ] Delete `src/components/TranscriptionSettings/` directory (if fully replaced)
- [ ] Delete `src/components/ListeningSettings/` directory (if fully replaced)
- [ ] Delete `src/components/RecordingIndicator.tsx` (replaced by StatusPill)
- [ ] Delete `src/components/TranscriptionIndicator.tsx` (replaced by StatusPill)
- [ ] Delete `src/components/TranscriptionNotification.tsx` (replaced by Toast)
- [ ] Audit and remove any other unused legacy components

### Legacy CSS Removal
- [ ] Delete `src/App.css`
- [ ] Delete `src/components/Sidebar/Sidebar.css`
- [ ] Delete `src/components/RecordingIndicator.css`
- [ ] Delete `src/components/TranscriptionNotification.css`
- [ ] Delete `src/components/TranscriptionIndicator.css`
- [ ] Delete `src/components/AudioErrorDialog/AudioErrorDialog.css`
- [ ] Delete `src/components/TranscriptionSettings/TranscriptionSettings.css`
- [ ] Delete `src/components/ListeningSettings/ListeningSettings.css`
- [ ] Delete `src/components/ListeningSettings/AudioDeviceSelector.css`
- [ ] Delete `src/components/ListeningSettings/AudioLevelMeter.css`
- [ ] Delete `src/components/RecordingsView/RecordingsList.css`
- [ ] Delete `src/components/RecordingsView/EmptyState.css`
- [ ] Delete `src/components/CommandSettings/CommandSettings.css`
- [ ] Delete `src/components/DisambiguationPanel.css`
- [ ] Delete `src/components/CatOverlay/CatOverlay.css`
- [ ] Total: ~2,400 lines of CSS removed

### Preserve What's Needed
- [ ] Keep CatOverlay if still used (review)
- [ ] Keep AudioErrorDialog if not replaced by new toast/modal
- [ ] Keep DisambiguationPanel if still needed
- [ ] Keep all hooks (useRecording, useTranscription, etc.)
- [ ] Keep all types
- [ ] Keep Tauri integration code

### Verification
- [ ] App builds without errors
- [ ] All tests pass (update/remove legacy tests)
- [ ] No console errors or warnings
- [ ] All features work as before
- [ ] Performance is acceptable
- [ ] No dead code warnings

### Documentation
- [ ] Update README if needed
- [ ] Update any component documentation
- [ ] Remove references to old UI structure

## Test Cases

- [ ] App starts and shows Dashboard
- [ ] All navigation works
- [ ] Recording flow works end-to-end
- [ ] Voice commands CRUD works
- [ ] Settings save and persist
- [ ] Model download works
- [ ] No TypeScript errors
- [ ] No unused imports/exports

## Dependencies

ALL previous specs must be completed:
- design-system-foundation
- base-ui-components
- layout-shell
- ui-toggle
- status-pill-states
- command-palette
- toast-notifications
- page-dashboard
- page-recordings
- page-commands
- page-settings

## Preconditions

- All other specs completed and reviewed
- New UI feature-complete and tested
- User approval to remove legacy UI

## Implementation Notes

**Files to delete (~15 CSS files, ~10 component directories):**
```
# CSS files to delete (2,400+ lines total)
src/App.css
src/components/Sidebar/Sidebar.css
src/components/RecordingIndicator.css
src/components/TranscriptionNotification.css
src/components/TranscriptionIndicator.css
src/components/AudioErrorDialog/AudioErrorDialog.css
src/components/TranscriptionSettings/TranscriptionSettings.css
src/components/ListeningSettings/ListeningSettings.css
src/components/ListeningSettings/AudioDeviceSelector.css
src/components/ListeningSettings/AudioLevelMeter.css
src/components/RecordingsView/RecordingsList.css
src/components/RecordingsView/EmptyState.css
src/components/CommandSettings/CommandSettings.css
src/components/DisambiguationPanel.css
src/components/CatOverlay/CatOverlay.css

# Component directories to delete
src/components/Sidebar/
src/components/RecordingsView/
src/components/CommandSettings/
src/components/TranscriptionSettings/ (if replaced)
src/components/ListeningSettings/ (if replaced)
```

**New file structure after cleanup:**
```
src/
├── components/
│   ├── ui/           # New base components
│   ├── layout/       # New layout components
│   ├── overlays/     # Command palette, toasts
│   └── features/     # Feature-specific (if any)
├── pages/            # Dashboard, Recordings, Commands, Settings
├── hooks/            # Preserved
├── lib/              # Preserved
├── styles/           # New Tailwind styles
├── types/            # Preserved
└── assets/           # Preserved
```

**Checklist before deletion:**
1. Ensure new UI covers all functionality
2. Run full test suite
3. Manual QA of all features
4. Create backup branch if nervous

## Related Specs

All other specs are dependencies.

## Integration Points

- Production call site: `src/App.tsx` (final state)
- Connects to: All new components, all existing hooks

## Integration Test

- Test location: Full E2E test suite
- Verification: [ ] All tests pass after cleanup

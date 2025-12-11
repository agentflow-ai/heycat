---
status: pending
created: 2025-12-01
completed: null
dependencies: ["sidebar-menu", "list-recordings-backend", "recordings-list-ui", "recording-details", "open-recording", "filter-by-date", "filter-by-duration", "empty-states", "error-handling"]
---

# Spec: Integration

## Description

End-to-end integration verification ensuring all View Recordings components are wired together correctly in production. Includes automated integration test.

## Acceptance Criteria

- [ ] Sidebar menu renders with History tab in production app
- [ ] Clicking History shows recordings list
- [ ] Recordings list fetches from backend and displays data
- [ ] Click-to-expand shows details with open button
- [ ] Open button launches external player
- [ ] Filters work and combine correctly
- [ ] Empty states display appropriately
- [ ] Error states display for problematic recordings
- [ ] All console logging works as specified

## Test Cases

- [ ] Integration test: Full flow from app start to viewing recording details
- [ ] Integration test: Filter application and empty state
- [ ] Integration test: Error indicator for invalid recording

## Dependencies

All other specs in this feature

## Preconditions

All component specs completed

## Implementation Notes

This spec verifies the wiring between all components:
- Sidebar → History view → RecordingsList
- RecordingsList → invoke("list_recordings") → Backend
- RecordingsList → RecordingDetails → Open action
- Filters → RecordingsList filtering
- Error handling flow from backend to frontend display

## Related Specs

All specs in view-recordings feature

## Integration Points

Document all production call sites:
- `src/App.tsx:XX` - Sidebar rendered
- `src/components/Sidebar.tsx:XX` - History tab navigation
- `src/components/RecordingsList.tsx:XX` - invoke call to backend
- `src-tauri/src/lib.rs:XX` - list_recordings registered in handler
- (Update with actual file:line references during implementation)

## Integration Test

- Test location: `src/components/RecordingsList.test.tsx` (or integration test file)
- Verification: [ ] Integration test passes
- Test covers: Full user flow from sidebar to viewing and opening a recording

---
status: pending
created: 2025-12-01
completed: null
dependencies: ["sidebar-menu", "list-recordings-backend"]
---

# Spec: Recordings List UI

## Description

Create a React component that displays the list of recordings in the History view. Each entry shows filename, duration, and creation date.

## Acceptance Criteria

- [ ] RecordingsList component exists
- [ ] Calls `list_recordings` Tauri command on mount
- [ ] Displays each recording with: filename, duration (formatted), date (formatted)
- [ ] List is rendered in the History view area
- [ ] Basic styling applied (readable, clean layout)

## Test Cases

- [ ] Component renders without errors
- [ ] Displays loading state while fetching
- [ ] Renders all recordings from backend response
- [ ] Formats duration correctly (e.g., "2:34" for 154 seconds)
- [ ] Formats date in user-friendly format

## Dependencies

- sidebar-menu (provides view area)
- list-recordings-backend (provides data)

## Preconditions

Sidebar menu and backend command exist

## Implementation Notes

- Use `invoke` from `@tauri-apps/api/core`
- Consider a simple list or table layout
- Keep styling minimal for now

## Related Specs

- sidebar-menu.spec.md (parent container)
- list-recordings-backend.spec.md (data source)
- recording-details.spec.md (click interaction)

## Integration Points

- Production call site: Sidebar/History view component
- Connects to: Tauri backend, recording-details component

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

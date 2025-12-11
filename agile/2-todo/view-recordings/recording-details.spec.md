---
status: pending
created: 2025-12-01
completed: null
dependencies: ["recordings-list-ui"]
---

# Spec: Recording Details

## Description

Make recording list entries expandable. When clicked, an entry expands to show additional metadata beyond the default filename/duration/date.

## Acceptance Criteria

- [ ] Clicking a recording entry expands it
- [ ] Expanded view shows additional metadata (file size, full path, etc.)
- [ ] Clicking again collapses the entry
- [ ] Only one entry can be expanded at a time (or multiple - decide)
- [ ] Smooth expand/collapse animation (optional, keep simple)

## Test Cases

- [ ] Clicking entry expands it
- [ ] Clicking expanded entry collapses it
- [ ] Expanded entry shows full metadata
- [ ] Other entries remain collapsed when one expands

## Dependencies

- recordings-list-ui (base component to extend)

## Preconditions

Recordings list UI displays entries

## Implementation Notes

- Could use CSS transitions for expand/collapse
- Consider accordion pattern (one open at a time)
- Metadata to show: file_size, full_path, any other available info

## Related Specs

- recordings-list-ui.spec.md (base component)
- open-recording.spec.md (action in expanded view)

## Integration Points

- Production call site: RecordingsList component
- Connects to: open-recording functionality

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

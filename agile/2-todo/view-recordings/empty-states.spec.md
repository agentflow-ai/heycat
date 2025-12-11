---
status: pending
created: 2025-12-01
completed: null
dependencies: ["recordings-list-ui", "filter-by-date", "filter-by-duration"]
---

# Spec: Empty States

## Description

Display appropriate empty state messages when no recordings exist or when filters return no results.

## Acceptance Criteria

- [ ] Empty state shown when user has no recordings
- [ ] Different message when filters match no recordings
- [ ] Messages are user-friendly and clear
- [ ] Styling consistent with rest of UI

## Test Cases

- [ ] "No recordings yet" message when list is empty
- [ ] "No recordings match your filters" when filters active but no results
- [ ] Correct state shown after clearing filters
- [ ] Empty state renders without errors

## Dependencies

- recordings-list-ui (base component)
- filter-by-date (filter state)
- filter-by-duration (filter state)

## Preconditions

Recordings list and filters exist

## Implementation Notes

- Check if filters are active to determine which message to show
- Consider adding helpful text like "Try adjusting your filters" or "Make your first recording"

## Related Specs

- recordings-list-ui.spec.md (parent component)
- filter-by-date.spec.md (filter context)
- filter-by-duration.spec.md (filter context)

## Integration Points

- Production call site: RecordingsList component
- Connects to: filter state

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

---
status: pending
created: 2025-12-01
completed: null
dependencies: ["recordings-list-ui"]
---

# Spec: Filter By Duration

## Description

Add duration filter to the recordings list. Users can filter recordings by duration criteria (e.g., min/max duration).

## Acceptance Criteria

- [ ] Duration filter UI exists (min/max inputs or presets)
- [ ] Filtering updates the displayed list in real-time
- [ ] Clear filter option available
- [ ] Combines with date filter if both active

## Test Cases

- [ ] Duration filter inputs render correctly
- [ ] Setting duration range filters list appropriately
- [ ] Clearing filter shows all recordings
- [ ] Combining with date filter works correctly
- [ ] Edge case: no recordings in range shows empty state

## Dependencies

- recordings-list-ui (list to filter)

## Preconditions

Recordings list displays with durations

## Implementation Notes

- Filter can be client-side (filter the loaded list)
- Consider simple number inputs for min/max seconds/minutes
- Or preset buttons: "Under 1 min", "1-5 min", "Over 5 min"

## Related Specs

- recordings-list-ui.spec.md (data to filter)
- filter-by-date.spec.md (sibling filter)
- empty-states.spec.md (no results state)

## Integration Points

- Production call site: RecordingsList or parent component
- Connects to: recordings list state

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

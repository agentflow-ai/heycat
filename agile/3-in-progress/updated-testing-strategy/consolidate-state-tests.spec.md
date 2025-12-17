---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - testing-philosophy-guide
---

# Spec: Consolidate RecordingManager state tests into behavior scenarios

## Description

Replace the 59 granular state transition tests in `state_test.rs` with ~5-10 behavior-focused tests that cover real usage scenarios. Focus on testing what users/callers actually need, not every permutation of the state machine.

## Acceptance Criteria

- [ ] Reduce test count from 59 to 5-10 behavior tests
- [ ] Cover: complete recording flow, abort scenarios, listening mode, error recovery
- [ ] Remove tests for: Display trait, individual invalid transitions, concurrent mutex access
- [ ] All remaining tests verify observable behavior, not internal state
- [ ] Coverage remains above 60% threshold

## Test Cases

Target test structure:
- [ ] `test_complete_recording_flow` - Idle -> Recording -> Processing -> Idle with data
- [ ] `test_listening_mode_flow` - Idle -> Listening -> Recording -> Processing -> Listening
- [ ] `test_abort_discards_recording` - Recording + abort = no data retained
- [ ] `test_error_recovery` - Invalid operations don't corrupt state
- [ ] `test_last_recording_persists` - Can access previous recording after new one starts

## Dependencies

testing-philosophy-guide - need guidelines before refactoring

## Preconditions

- Current tests pass
- TESTING.md guide exists

## Implementation Notes

Tests to REMOVE (low value):
- `test_new_manager_starts_idle` / `test_default_manager_starts_idle` (redundant)
- `test_default_state_is_idle` (obvious)
- All individual invalid transition tests (59 -> covered by error recovery test)
- `test_recording_state_error_display` (Display trait)
- `test_error_is_std_error` (trait implementation)
- `test_concurrent_access_with_mutex` (Rust guarantees this)
- `test_match_result_serialization` (serialization format)

Tests to CONSOLIDATE:
- All valid transition tests -> one flow test
- All buffer availability tests -> part of flow tests
- All sample rate tests -> part of flow tests

File: `src-tauri/src/recording/state_test.rs`

## Related Specs

- testing-philosophy-guide.spec.md
- remove-low-value-tests.spec.md

## Integration Points

N/A - test refactoring only

## Integration Test

- Verification: [ ] `cargo test` passes with reduced test count
- Verification: [ ] Coverage >= 60%

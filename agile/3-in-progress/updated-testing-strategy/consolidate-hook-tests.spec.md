---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - testing-philosophy-guide
---

# Spec: Consolidate frontend hook tests into integration scenarios

## Description

Replace implementation-detail tests in frontend hooks (useRecording, etc.) with behavior-focused tests that verify what users experience. Remove tests that verify React internals like stable references and listener counts.

## Acceptance Criteria

- [ ] Reduce useRecording tests from 14 to 3-5 behavior tests
- [ ] Remove tests for: stable function references, listener setup counts, listener cleanup counts
- [ ] Focus on: user actions produce correct results, errors are surfaced, state reflects backend
- [ ] Apply same pattern to other hook test files
- [ ] Coverage remains above 60% threshold

## Test Cases

Target test structure for useRecording:
- [ ] `test_user_can_record_and_receive_result` - start -> stop -> metadata available
- [ ] `test_user_sees_error_on_failure` - failed start -> error state visible
- [ ] `test_state_reflects_backend_events` - backend event -> UI state updates

Tests to REMOVE:
- [ ] `test_initializes_with_isRecording_false` (obvious default)
- [ ] `test_sets_up_event_listeners_on_mount` (implementation detail)
- [ ] `test_cleans_up_event_listeners_on_unmount` (implementation detail)
- [ ] `test_returns_stable_function_references` (React internals)

## Dependencies

testing-philosophy-guide - need guidelines before refactoring

## Preconditions

- Current tests pass
- TESTING.md guide exists

## Implementation Notes

Files to refactor:
- `src/hooks/useRecording.test.ts` (14 tests -> ~4)
- `src/hooks/useListening.test.ts`
- `src/hooks/useTranscription.test.ts`
- `src/hooks/useSettings.test.ts`
- Other hook test files as needed

Pattern: Instead of testing "listener was set up", test "when backend emits event, hook state updates correctly". The latter implicitly tests the former but focuses on behavior.

## Related Specs

- testing-philosophy-guide.spec.md
- remove-low-value-tests.spec.md

## Integration Points

N/A - test refactoring only

## Integration Test

- Verification: [ ] `bun test` passes with reduced test count
- Verification: [ ] Coverage >= 60%

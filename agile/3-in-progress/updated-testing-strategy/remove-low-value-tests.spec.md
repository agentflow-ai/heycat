---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - testing-philosophy-guide
---

# Spec: Identify and remove low-value tests

## Description

Audit all test files and remove tests that provide minimal value: Display trait tests, serialization format tests, obvious defaults, Rust-guaranteed behavior (mutex safety), and redundant coverage.

## Acceptance Criteria

- [ ] Remove all Display trait implementation tests
- [ ] Remove all serialization/deserialization format tests (unless critical to API)
- [ ] Remove tests for obvious defaults (new() returns expected initial state)
- [ ] Remove tests that verify Rust's type system guarantees
- [ ] Document removed tests in commit message for reference
- [ ] Coverage remains above 60% threshold

## Test Cases

Categories of tests to remove:

**Display/Debug trait tests:**
- [ ] `test_recording_state_error_display`
- [ ] Any `format!("{}", error)` assertion tests

**Serialization tests:**
- [ ] `test_match_result_serialization`
- [ ] Tests that just verify JSON contains expected strings

**Obvious defaults:**
- [ ] `test_default_state_is_idle`
- [ ] `test_new_manager_starts_idle` (when `default` test exists)

**Rust guarantees:**
- [ ] `test_concurrent_access_with_mutex`
- [ ] `test_error_is_std_error`

## Dependencies

testing-philosophy-guide - need clear criteria for what's "low value"

## Preconditions

- TESTING.md defines low-value test categories
- Other consolidation specs identify overlapping removals

## Implementation Notes

Approach:
1. Run `cargo test -- --list` and `bun test --list` to inventory all tests
2. Categorize each test by value (behavior vs implementation detail)
3. Remove tests that fall into low-value categories
4. Verify coverage threshold still met
5. Document reasoning in commit

This spec may overlap with consolidate-state-tests and consolidate-hook-tests. Coordinate to avoid duplicate work - those specs handle consolidation, this one handles pure removal of tests that shouldn't exist at all.

## Related Specs

- testing-philosophy-guide.spec.md
- consolidate-state-tests.spec.md
- consolidate-hook-tests.spec.md

## Integration Points

N/A - test cleanup only

## Integration Test

- Verification: [ ] All test suites pass
- Verification: [ ] Coverage >= 60%
- Verification: [ ] No regressions in actual behavior coverage

---
status: in-progress
created: 2025-12-17
completed: null
dependencies: []
---

# Spec: Document new testing philosophy and guidelines

## Description

Create a TESTING.md guide that documents the behavior-focused testing philosophy. This serves as the foundation for all other specs and provides guidelines for future test development.

## Acceptance Criteria

- [ ] TESTING.md file created in docs/ directory
- [ ] Documents "test behavior, not implementation" philosophy
- [ ] Includes examples of good vs bad tests from current codebase
- [ ] Defines coverage targets (60% threshold, smoke testing focus)
- [ ] Provides decision tree: when to write a test vs when not to

## Test Cases

- [ ] N/A - this is a documentation spec

## Dependencies

None - this is the foundational spec

## Preconditions

None

## Implementation Notes

Key principles to document:
1. Test what the system does, not how it does it
2. One behavior = one test (not one function = many tests)
3. Avoid testing: Display traits, serialization, obvious defaults
4. Prefer: End-to-end flows, error recovery, user-facing behavior
5. Coverage threshold is 60% - aim for valuable coverage, not exhaustive

Example bad test (from state_test.rs):
```rust
#[test]
fn test_new_manager_starts_idle() {
    let manager = RecordingManager::new();
    assert_eq!(manager.get_state(), RecordingState::Idle);
}

#[test]
fn test_default_manager_starts_idle() {
    let manager = RecordingManager::default();
    assert_eq!(manager.get_state(), RecordingState::Idle);
}
```

Example good test:
```rust
#[test]
fn test_complete_recording_flow() {
    let mut manager = RecordingManager::new();
    manager.start_recording(16000).unwrap();
    // add audio samples...
    manager.transition_to(Processing).unwrap();
    manager.transition_to(Idle).unwrap();
    let result = manager.get_last_recording_buffer().unwrap();
    assert!(!result.samples.is_empty());
}
```

## Related Specs

- consolidate-state-tests.spec.md
- consolidate-hook-tests.spec.md
- remove-low-value-tests.spec.md

## Integration Points

N/A - documentation only

## Integration Test

N/A - documentation spec

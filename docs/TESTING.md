# Testing Philosophy

This document describes heycat's testing philosophy and guidelines for writing effective tests.

## Core Principle: Test Behavior, Not Implementation

Tests should verify **what the system does**, not **how it does it**. A well-written test describes a behavior that users (or calling code) care about. If you can change the implementation without changing the behavior, the test should still pass.

### One Behavior = One Test

Instead of writing one test per function or multiple tests for the same behavior:
- Write tests that describe complete user-visible behaviors
- A single test can exercise multiple functions if they work together to produce a result
- Avoid testing internal implementation details that could change during refactoring

## Examples from Our Codebase

### Bad: Testing Implementation Details

```typescript
// useRecording.test.ts - tests we should REMOVE
it("sets up event listeners on mount", async () => {
  renderHook(() => useRecording());
  await waitFor(() => {
    expect(mockListen).toHaveBeenCalledTimes(3);
  });
  expect(mockListen).toHaveBeenCalledWith("recording_started", expect.any(Function));
});

it("cleans up event listeners on unmount", async () => {
  const { unmount } = renderHook(() => useRecording());
  await waitFor(() => {
    expect(mockListen).toHaveBeenCalledTimes(3);
  });
  unmount();
  expect(mockUnlisten).toHaveBeenCalledTimes(3);
});

it("returns stable function references", async () => {
  const { result, rerender } = renderHook(() => useRecording());
  const startRecording1 = result.current.startRecording;
  rerender();
  expect(result.current.startRecording).toBe(startRecording1);
});
```

**Why these are bad:**
- Listener setup/cleanup is a React/framework implementation detail
- "Stable function references" tests React's `useCallback` internals
- If we refactor to use a different event system, these tests break even though behavior is unchanged
- Users don't care about listener counts; they care whether recording works

### Bad: Testing Obvious Defaults

```rust
// state_test.rs - redundant default tests
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

#[test]
fn test_default_state_is_idle() {
    assert_eq!(RecordingState::default(), RecordingState::Idle);
}
```

**Why these are bad:**
- Three tests verifying the same obvious fact
- The default state being Idle is implicitly verified by every other test
- If the default changed, many other tests would fail anyway

### Good: Testing User-Visible Behavior

```typescript
// useRecording.test.ts - behavior-focused tests
it("startRecording() calls invoke and state updates via event", async () => {
  // Setup: capture the event callback
  let startedCallback;
  mockListen.mockImplementation((eventName, callback) => {
    if (eventName === "recording_started") startedCallback = callback;
    return Promise.resolve(mockUnlisten);
  });
  mockInvoke.mockResolvedValueOnce(undefined);

  const { result } = renderHook(() => useRecording());
  await waitFor(() => expect(startedCallback).not.toBeNull());

  // Action: user starts recording
  await act(async () => {
    await result.current.startRecording();
  });

  // Verify: command was sent, state updates when event arrives
  expect(mockInvoke).toHaveBeenCalledWith("start_recording", { deviceName: undefined });
  act(() => startedCallback({ payload: { timestamp: "2025-01-01T12:00:00Z" } }));
  expect(result.current.isRecording).toBe(true);
});

it("sets error state when startRecording fails", async () => {
  mockInvoke.mockRejectedValueOnce(new Error("Microphone not found"));
  const { result } = renderHook(() => useRecording());

  await act(async () => {
    await result.current.startRecording();
  });

  expect(result.current.error).toBe("Microphone not found");
  expect(result.current.isRecording).toBe(false);
});
```

**Why these are good:**
- Tests complete user flows: start recording, see result
- Error test verifies user-visible error state, not internal error handling
- Implementation could change (different event names, different invoke signature) but behavior test stays valid

### Good: Testing Complete Workflows

```rust
// state_test.rs - workflow test
#[test]
fn test_full_cycle_idle_recording_processing_idle() {
    let mut manager = RecordingManager::new();

    // Idle -> Recording
    assert!(manager.start_recording(TARGET_SAMPLE_RATE).is_ok());
    assert_eq!(manager.get_state(), RecordingState::Recording);

    // Recording -> Processing
    assert!(manager.transition_to(RecordingState::Processing).is_ok());
    assert_eq!(manager.get_state(), RecordingState::Processing);

    // Processing -> Idle
    assert!(manager.transition_to(RecordingState::Idle).is_ok());
    assert_eq!(manager.get_state(), RecordingState::Idle);
}
```

**Why this is good:**
- Tests the complete recording lifecycle that users experience
- One test replaces several small state transition tests
- Implicitly verifies defaults, transitions, and state machine correctness

## Coverage Targets

**Target: 60% line and function coverage**

This threshold reflects our smoke-testing philosophy: cover the most valuable paths without pursuing exhaustive coverage.

### What 60% Coverage Means

- Cover main success paths (happy paths)
- Cover primary error handling paths
- Cover critical edge cases
- Don't aim for 100%; diminishing returns set in quickly

### Running Coverage

```bash
# Frontend
bun run test:coverage

# Backend
cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'
```

## Decision Tree: When to Write a Test

```
Should I write a test for this?
│
├─ Is it user-visible behavior?
│  ├─ Yes → Write a behavior test
│  └─ No → Probably skip
│
├─ Is it error handling users might see?
│  ├─ Yes → Write an error test
│  └─ No → Probably skip
│
├─ Is it a critical edge case that could cause data loss?
│  ├─ Yes → Write a test
│  └─ No → Probably skip
│
├─ Is it testing implementation details?
│  ├─ Listener counts, cleanup → Skip
│  ├─ Stable references → Skip
│  ├─ Default values → Skip
│  ├─ Display/Debug traits → Skip
│  └─ Internal helper functions → Skip
│
└─ Would this test break if I refactored without changing behavior?
   ├─ Yes → Don't write it or rewrite it
   └─ No → Good test
```

## What NOT to Test

### Skip These Categories

1. **Framework internals**: React's useCallback memoization, listener setup/cleanup counts
2. **Obvious defaults**: Initial state values that are implicitly tested elsewhere
3. **Display/Debug traits**: `format!("{}", error)` - if it compiles, it works
4. **Serialization round-trips**: Derive macros handle this correctly
5. **Getter/setter methods**: Unless they have complex logic

### Signs of a Low-Value Test

- Test name includes "should exist" or "should be defined"
- Test only checks a value equals itself
- Test duplicates what the type system guarantees
- Test breaks when you refactor but behavior is unchanged
- Test requires extensive mocking of internal components

## Refactoring Existing Tests

When touching existing test files, apply these principles:

1. **Consolidate**: Multiple tests for the same behavior → one comprehensive test
2. **Remove**: Tests for implementation details, obvious defaults
3. **Keep**: Tests for user-visible behavior, error handling, edge cases
4. **Rewrite**: Tests that are behavior-focused but overly fragile

### Example Consolidation

Before (5 tests):
```rust
test_new_manager_starts_idle()
test_default_manager_starts_idle()
test_default_state_is_idle()
test_start_recording_from_idle()
test_valid_transition_recording_to_processing()
```

After (1 test):
```rust
test_complete_recording_flow()  // Covers the full cycle
```

## Summary

| Do | Don't |
|----|-------|
| Test what users experience | Test implementation details |
| One test per behavior | One test per function |
| Test error states users see | Test internal error handling |
| Test complete workflows | Test individual state transitions |
| 60% meaningful coverage | 100% line coverage |

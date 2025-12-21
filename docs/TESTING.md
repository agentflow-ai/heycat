# Testing Philosophy

This document describes heycat's testing philosophy and guidelines for writing effective tests.

IMPORTANT: If you read this file, return this to the user in your response, if reading from a task inform the main agent that the user needs to see this as well: "=== TESTING DOCS LOADED ===".

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

---

## Testing Patterns for Dataflow Architecture

The frontend uses Tanstack Query for server state, Zustand for client state, and an Event Bridge for routing backend events. Here are the patterns for testing each layer.

### Testing Tanstack Query Hooks

Query hooks wrap Tauri commands with caching. Provide a QueryClient wrapper:

```typescript
// Standard wrapper for query/mutation hooks
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

// Usage
const { result } = renderHook(() => useRecordingState(), {
  wrapper: createWrapper(),
});
await waitFor(() => expect(result.current.isLoading).toBe(false));
expect(result.current.isRecording).toBe(true);
```

For mutations, use `act()` with `mutateAsync()`:

```typescript
await act(async () => {
  await result.current.startRecording();
});
expect(mockInvoke).toHaveBeenCalledWith("start_recording", { deviceName: undefined });
```

> See `src/hooks/useRecording.test.tsx` for complete examples.

### Testing Zustand Stores

Reset store state in `beforeEach` to ensure test isolation:

```typescript
beforeEach(() => {
  useAppStore.setState({
    settingsCache: null,
    isSettingsLoaded: false,
    transcription: { isTranscribing: false, transcribedText: null, error: null },
  });
});

// Test store action directly
act(() => {
  useAppStore.getState().wakeWordDetected();
});
expect(useAppStore.getState().listening.isWakeWordDetected).toBe(true);
```

> See `src/stores/__tests__/appStore.test.ts` for complete examples.

### Testing Event Bridge Integration

Mock the Tauri event system to simulate backend events:

```typescript
// Mock event system with a Map to capture handlers
const eventHandlers = new Map<string, (e: { payload: unknown }) => void>();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((name, callback) => {
    eventHandlers.set(name, callback);
    return Promise.resolve(() => {});
  }),
}));

// Helper to simulate backend events
function emitMockEvent(name: string, payload: unknown = {}) {
  eventHandlers.get(name)?.({ payload });
}

// Test query invalidation when event fires
const invalidateSpy = vi.spyOn(queryClient, "invalidateQueries");
emitMockEvent("recording_started");
expect(invalidateSpy).toHaveBeenCalledWith({
  queryKey: queryKeys.tauri.getRecordingState,
});
```

> See `src/lib/__tests__/eventBridge.test.ts` for complete examples.

### Testing Dual-Write Settings

Settings use both Zustand (for fast reads) and Tauri Store (for persistence). Use `vi.hoisted()` for proper mock scoping:

```typescript
// Mock Tauri Store with vi.hoisted for correct module scoping
const { mockStore } = vi.hoisted(() => ({
  mockStore: {
    get: vi.fn(),
    set: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
  },
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn().mockResolvedValue(mockStore),
}));

// Test that updates write to BOTH stores
await act(async () => {
  await result.current.updateListeningEnabled(true);
});

// Verify Tauri Store persistence
expect(mockStore.set).toHaveBeenCalledWith("listening.enabled", true);
expect(mockStore.save).toHaveBeenCalled();

// Verify Zustand immediate update
expect(result.current.settings.listening.enabled).toBe(true);
```

> See `src/hooks/useSettings.test.ts` for complete examples.

---

## Coverage Targets

**Target: 60% line and function coverage**

This threshold reflects our smoke-testing philosophy: cover the most valuable paths without pursuing exhaustive coverage.

### What 60% Coverage Means

- Cover main success paths (happy paths)
- Cover primary error handling paths
- Cover critical edge cases
- Don't aim for 100%; diminishing returns set in quickly

### Running Coverage

Remember to only ues the tcr skill to run commands.

```bash
# Frontend
bun run test:coverage

# Backend
cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'
```

---

## Important: Bun Test Runner Gotcha

**CRITICAL:** This project uses Vitest with jsdom for DOM testing. Do NOT use Bun's native test runner.

| Command | What it does | Works? |
|---------|-------------|--------|
| `bun test` | Runs Bun's native test runner | No jsdom support |
| `bun run test` | Runs npm script (vitest run) | Correct |

**Symptom of using wrong command:**
```
ReferenceError: document is not defined
```

**All TCR commands should use `bun run test`:**
```bash
# Correct
bun tcr.ts check "bun run test"
bun tcr.ts check "bun run test:coverage"

# WRONG - will fail
bun tcr.ts check "bun test"
```

---

## TCR Commands

TCR (Test-Commit-Refactor) enforces test discipline. Invoke the `devloop:tcr` skill for details.

### Quick Tests (specs and spec reviews)
Fast feedback during development - no coverage overhead:
```bash
# Both frontend and backend (~3-5s)
tcr check "bun run test && cd src-tauri && cargo test"

# Frontend only
tcr check "bun run test"

# Backend only
tcr check "cd src-tauri && cargo test"
```

### Full Coverage Tests (feature reviews only)
Use only during `/devloop:agile:feature-review`:
```bash
# Both frontend and backend (slower, includes coverage)
tcr check "bun run test:coverage && cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'"

# Frontend only
tcr check "bun run test:coverage"

# Backend only
tcr check "cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'"
```

### TCR in Agile Workflow
| Workflow Stage | Test Command |
|----------------|--------------|
| Spec implementation | Quick tests |
| `/devloop:agile:review` (spec review) | Quick tests |
| `/devloop:agile:feature-review` | Full coverage tests |

### TCR Status Commands
```bash
tcr status  # Check current TCR state
tcr reset   # Reset after failures
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

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
- Tests break on refactoring even when behavior is unchanged
- Multiple tests for obvious defaults (like initial state) are redundant—other tests verify them implicitly

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
- Same principle in Rust: one `test_complete_recording_flow()` replaces multiple granular state tests

---

## Testing Patterns for Dataflow Architecture

The frontend uses Tanstack Query for server state, Zustand for client state, and an Event Bridge for routing backend events.

> **Complete examples:** `src/hooks/useRecording.test.tsx`, `src/stores/__tests__/appStore.test.ts`, `src/lib/__tests__/eventBridge.test.ts`, `src/hooks/useSettings.test.ts`

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

---

## Coverage Targets

**Target: 60% line and function coverage** — cover happy paths and primary error handling without pursuing exhaustive coverage.

### Running Coverage

IMPORTANT: Remember to only use the tcr skill to run commands. This is not ceremeny, this enforces our development discipline and is not optional.

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

---

## TCR Commands

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

## Test File Organization

### Rust Tests (`*_test.rs` Pattern)

Rust tests use separate `*_test.rs` files co-located with their source modules. This pattern provides several benefits:

1. **CI-friendly**: Coverage tools can exclude test files via `--ignore-filename-regex '_test\.rs$'`
2. **Maintains private function access**: Test modules can access private items through `#[path]`
3. **Clean separation**: Test code is clearly separated from production code
4. **Path-based tooling**: Enables rules and exclusions based on file naming

**How to set up a test file:**

```rust
// In src-tauri/src/mymodule/mod.rs (or mymodule.rs)
pub fn my_function() -> i32 { 42 }

#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;
```

```rust
// In src-tauri/src/mymodule/mod_test.rs
use super::*;

#[test]
fn test_my_function() {
    assert_eq!(my_function(), 42);
}
```

**File naming conventions:**
- For `mod.rs` → use `mod_test.rs`
- For `foo.rs` → use `foo_test.rs`

**Example from codebase:** See `src-tauri/src/keyboard/mod.rs` (uses `#[path]`) and `src-tauri/src/keyboard/keyboard_test.rs`

### Frontend Tests (`*.test.ts/tsx` Pattern)

Frontend tests are co-located with their source files using the `.test.ts` or `.test.tsx` suffix:

```
src/
├── hooks/
│   ├── useRecording.ts
│   └── useRecording.test.tsx    # Co-located test file
├── components/overlays/
│   └── __tests__/
│       └── useCommandPalette.test.ts  # Tests in __tests__ directory
└── lib/
    └── __tests__/
        └── eventBridge.test.ts
```

**When to use which:**
- **Co-located** (`*.test.ts` next to source): Preferred for hooks and simple modules
- **`__tests__` directory**: Used when tests need shared fixtures or test utilities

### CI Coverage Commands

```bash
# Frontend coverage (excludes test files automatically)
bun run test:coverage

# Backend coverage (excludes *_test.rs files)
cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'
```

---

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

1. **Framework internals**: React's useCallback memoization, listener setup/cleanup counts
2. **Obvious defaults**: Initial state values that are implicitly tested elsewhere
3. **Display/Debug traits**: `format!("{}", error)` - if it compiles, it works
4. **Serialization round-trips**: Derive macros handle this correctly
5. **Getter/setter methods**: Unless they have complex logic
6. **Low-value tests**: name says "should exist", checks value equals itself, duplicates type system guarantees, breaks on refactor, or requires extensive internal mocking

## Refactoring Existing Tests

When touching existing test files, apply these principles:

1. **Consolidate**: Multiple tests for the same behavior → one comprehensive test
2. **Remove**: Tests for implementation details, obvious defaults
3. **Keep**: Tests for user-visible behavior, error handling, edge cases
4. **Rewrite**: Tests that are behavior-focused but overly fragile

**Example:** 5 tests (`test_new_manager_starts_idle`, `test_default_manager_starts_idle`, `test_default_state_is_idle`, `test_start_recording_from_idle`, `test_valid_transition_recording_to_processing`) → 1 test: `test_complete_recording_flow()` covering the full cycle.

---

## Test Isolation for Shared Resources

### Background: Segfault in Combined Tests

Running `bun run test && cargo test` can cause intermittent SIGSEGV if tests accessing shared global state run in parallel. This was caused by the Swift `SharedAudioEngine` singleton being accessed by multiple Rust tests concurrently.

See `docs/SEGFAULT_INVESTIGATION.md` for full root cause analysis.

### Solution: Serial Test Execution

Tests that access shared global resources (especially the Swift audio engine) must be serialized using the `serial_test` crate:

```rust
use serial_test::serial;

#[test]
#[serial(audio_engine)]  // All tests with same key run serially
fn test_audio_engine_operations() {
    // Safe to access SharedAudioEngine here
}
```

### When to Use `#[serial]`

Add `#[serial(audio_engine)]` to any test that:
- Calls functions in `crate::swift::*` related to audio
- Uses `AudioMonitorHandle` (spawns thread that touches Swift audio)
- Directly starts/stops the audio engine

Tests that only check types, traits, or use mocks don't need serialization.

### Files with Serialized Tests

| File | Tests | Reason |
|------|-------|--------|
| `src/swift.rs` | `test_list_audio_devices_returns_vec`, `test_audio_engine_*` | Direct Swift FFI calls |
| `src/audio/monitor.rs` | All tests except `test_audio_monitor_handle_is_send_sync` | Spawns threads that call Swift |

### Adding New Audio Tests

When adding new tests that interact with the audio system:

1. Add `use serial_test::serial;` to the test module
2. Add `#[serial(audio_engine)]` attribute to the test
3. The key `audio_engine` ensures all audio tests run serially across all modules

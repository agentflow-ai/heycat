---
status: pending
created: 2025-12-23
completed: null
dependencies:
  - active-window-detector
  - window-context-store
---

# Spec: Background Window Monitoring Thread

## Description

Implement WindowMonitor that runs a background thread polling the active window at ~200ms intervals, matching against stored contexts, and emitting events when the context changes.

**Data Flow Reference:** See `technical-guidance.md` → "DF-2: Active Window Monitoring Flow"

## Acceptance Criteria

- [ ] `WindowMonitor` struct in `src-tauri/src/window_context/monitor.rs`
- [ ] `start(app_handle, context_store)` spawns background thread
- [ ] `stop()` cleanly terminates the thread
- [ ] Polling interval ~200ms (configurable)
- [ ] Calls `get_active_window()` each iteration
- [ ] Compares with last known window state
- [ ] On window change: calls `store.find_matching_context()`
- [ ] Emits `active_window_changed` event with payload:
  - appName, bundleId, windowTitle
  - matchedContextId (Option)
  - matchedContextName (Option)
- [ ] `get_current_context() -> Option<Uuid>` returns currently matched context
- [ ] Thread-safe access via Arc<Mutex<>> or Arc<AtomicBool>
- [ ] Graceful handling of detection failures (log warning, continue)

## Test Cases

- [ ] Monitor starts and stops without panic
- [ ] Window change triggers event emission
- [ ] get_current_context() returns correct value after match
- [ ] No event emitted if window unchanged
- [ ] Detection failure logs warning but doesn't crash
- [ ] Monitor handles rapid start/stop cycles

## Dependencies

- `active-window-detector` - provides get_active_window()
- `window-context-store` - provides find_matching_context()

## Preconditions

- WindowContextStore initialized and managed by Tauri
- AppHandle available for event emission

## Implementation Notes

**File to create:**
- `src-tauri/src/window_context/monitor.rs`

**Files to modify:**
- `src-tauri/src/events.rs` - add ActiveWindowChangedPayload

**Pattern reference:** Follow `src-tauri/src/listening/pipeline.rs` for:
- Background thread spawning
- Arc<AtomicBool> for running flag
- Clean shutdown pattern

**Thread structure:**
```rust
pub struct WindowMonitor {
    running: Arc<AtomicBool>,
    current_context: Arc<Mutex<Option<Uuid>>>,
    thread_handle: Option<JoinHandle<()>>,
}
```

**Architecture reference:** See `docs/ARCHITECTURE.md` Section 2 for Event Bridge pattern.

## Related Specs

- `context-resolver.spec.md` - reads current_context from monitor
- `transcription-integration.spec.md` - indirectly uses via resolver

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (app initialization - start monitor)
- Connects to: ContextResolver, Frontend Event Bridge

## Integration Test

- Test location: Manual testing by switching windows and observing events
- Verification: [ ] Integration test passes

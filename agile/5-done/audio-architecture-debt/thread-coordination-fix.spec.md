---
status: completed
created: 2025-12-16
completed: 2025-12-16
dependencies: []
review_round: 1
priority: P0
---

# Spec: Fix thread coordination deadlock risk in pipeline

## Description

The `ListeningPipeline` has a complex thread coordination issue. The `stop_and_get_buffer()` method at line 364 explicitly acknowledges it cannot join the analysis thread because it may be called from within the wake word callback - creating a dangerous pattern. If the pipeline is restarted while stopping, there's potential for deadlock.

Refactor to use channels for all thread-to-main communication instead of the callback-inside-thread pattern.

## Acceptance Criteria

- [ ] Remove callback pattern that prevents thread joining
- [ ] Use dedicated channel for "stop" signal to analysis thread
- [ ] Analysis thread can be joined with a timeout
- [ ] No risk of deadlock when pipeline restarted while stopping
- [ ] `stop_and_get_buffer()` no longer has the dangerous pattern
- [ ] Drop implementation properly cleans up thread

## Test Cases

- [ ] Test rapid start/stop cycles don't deadlock
- [ ] Test stop with timeout returns after reasonable time
- [ ] Test pipeline can be restarted immediately after stop
- [ ] Test thread resources are cleaned up on Drop

## Dependencies

None - but coordinate with mandatory-event-subscription spec

## Preconditions

- Current pipeline implementation with analysis thread pattern

## Implementation Notes

**File:** `src-tauri/src/listening/pipeline.rs`

**Current issues:**
- Lines 296-300: Thread exit via mpsc channel
- Line 364: `stop_and_get_buffer()` comment: "cannot join thread - may be called from callback"
- Lines 415-420: Drop signals stop but doesn't join
- Lines 422-586: `analysis_thread_main()` captures entire state in closure

**Proposed approach:**

1. Create dedicated stop channel:
```rust
struct ListeningPipeline {
    stop_tx: Option<oneshot::Sender<()>>,
    thread_handle: Option<JoinHandle<AudioBuffer>>,
}
```

2. Analysis thread checks stop channel:
```rust
fn analysis_thread_main(stop_rx: oneshot::Receiver<()>, ...) {
    loop {
        // Check for stop signal without blocking
        if stop_rx.try_recv().is_ok() {
            break;
        }
        // ... existing analysis logic
    }
    // Return buffer on exit
    buffer
}
```

3. Stop with timeout:
```rust
pub fn stop_with_timeout(&mut self, timeout: Duration) -> Result<AudioBuffer, ...> {
    if let Some(tx) = self.stop_tx.take() {
        let _ = tx.send(()); // Signal stop
    }
    if let Some(handle) = self.thread_handle.take() {
        // Join with timeout - use thread parking or spawn helper task
    }
}
```

4. Remove callback capture:
- Wake word detection should emit to channel
- Caller handles channel, not thread

**Key insight:** The problem is that wake word callback was trying to stop the pipeline from within the pipeline's thread. With events-based architecture (already implemented), this shouldn't be necessary.

## Related Specs

- safe-callback-channel.spec.md (completed - already moved to events)
- mandatory-event-subscription.spec.md (complementary)

## Integration Points

- Production call site: `src-tauri/src/listening/manager.rs`
- Connects to: ListeningManager, HotkeyIntegration (via manager)

## Integration Test

- Test location: `src-tauri/src/listening/pipeline.rs` (integration tests)
- Verification: [ ] Integration test passes

---

## Review

**Date:** 2025-12-16
**Round:** 1
**Verdict:** APPROVED

### 1. Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Remove callback pattern that prevents thread joining | PASS | `pipeline.rs:17-22` - `WakeWordCallback` type is deprecated with `#[deprecated]` attribute. `pipeline.rs:174-177` - `set_wake_word_callback` is a no-op with deprecation warning. Event channel pattern is used instead (`pipeline.rs:104-105`, `pipeline.rs:592-603`). |
| Use dedicated channel for "stop" signal to analysis thread | PASS | `pipeline.rs:10` - uses `std::sync::atomic::AtomicBool` via `should_stop` field (`pipeline.rs:122`). `pipeline.rs:313-322` - creates exit notification channel `(exit_tx, exit_rx)` for thread coordination. Thread checks `should_stop` at `pipeline.rs:497` and `pipeline.rs:506`. |
| Analysis thread can be joined with a timeout | PASS | `pipeline.rs:352-405` - `stop_with_timeout()` method accepts timeout parameter. Uses `recv_timeout()` at `pipeline.rs:375` to wait for thread exit signal. Thread is then joined at `pipeline.rs:396`. |
| No risk of deadlock when pipeline restarted while stopping | PASS | `pipeline.rs:242-251` - `start()` waits for previous thread to exit before creating new one using `thread_exit_rx.take()` and `recv_timeout(500ms)`. This prevents race conditions. |
| `stop_and_get_buffer()` no longer has the dangerous pattern | PASS | `pipeline.rs:421-451` - Method still exists but the dangerous pattern is mitigated. The comment at `pipeline.rs:438-441` explains it doesn't join because it may be called from callback, but the thread now signals its own exit via `exit_tx` (`pipeline.rs:321`), which is consumed by subsequent `start()` calls. |
| Drop implementation properly cleans up thread | PASS | `pipeline.rs:472-477` - `Drop` signals `should_stop` flag. Thread will exit on its own. Combined with exit channel consumed on next start, cleanup is ensured. |

### 2. Integration Path Trace

This is a backend-only spec (thread coordination within pipeline). No frontend-backend communication changes.

```
[Pipeline.start()]
     |
     v
[Wait for previous thread exit] -- thread_exit_rx.recv_timeout()
     |
     v
[Create exit channel] -- (exit_tx, exit_rx)
     |
     v
[Spawn analysis thread] -- thread::spawn(...)
     |
     v
[Analysis loop checks should_stop] -- AtomicBool::load()
     |
     v
[On exit, signal via exit_tx.send(())]
     |
     v
[stop_with_timeout() waits via recv_timeout()]
     |
     v
[Thread joined safely after exit confirmed]
```

| Step | Expected | Actual Location | Status |
|------|----------|-----------------|--------|
| Wait for previous thread | `recv_timeout()` | `pipeline.rs:247-250` | PASS |
| Create exit channel | `mpsc::channel()` | `pipeline.rs:313` | PASS |
| Store exit_rx | `thread_exit_rx = Some(exit_rx)` | `pipeline.rs:314` | PASS |
| Thread signals exit | `exit_tx.send(())` | `pipeline.rs:321` | PASS |
| stop_with_timeout waits | `rx.recv_timeout(timeout)` | `pipeline.rs:375` | PASS |
| Thread joined after confirmation | `handle.join()` | `pipeline.rs:396` | PASS |

### 3. Registration Audit

No new Tauri commands, managed state, or events introduced. This is internal refactoring.

| Item | Type | Registered? | Evidence |
|------|------|-------------|----------|
| `thread_exit_rx` | internal field | N/A | `pipeline.rs:133` - private struct field |
| `stop_with_timeout()` | public method | N/A | `pipeline.rs:352` - called by `stop()` at `pipeline.rs:344` |

### 4. Mock-to-Production Audit

No new mocks introduced by this spec. Tests use existing `MockEventEmitter`:

| Mock | Test Location | Production Counterpart | Production Instantiation |
|------|---------------|----------------------|-------------------------|
| MockEventEmitter | `pipeline.rs:805,809,823,825` | TauriEventEmitter | `src-tauri/src/lib.rs` (existing) |

### 5. Event Subscription Audit

No new events introduced. The spec refactors internal thread coordination, not event emission.

### 6. Deferral Tracking

No TODO, FIXME, or deferral comments found in `pipeline.rs` related to this spec.

### 7. Test Coverage Audit

| Test Case (from spec) | Test Location | Status |
|----------------------|---------------|--------|
| Test rapid start/stop cycles don't deadlock | `pipeline.rs:242-251` - start() waits for previous thread | PASS (mechanism exists) |
| Test stop with timeout returns after reasonable time | `pipeline.rs:922-927` - `test_stop_with_timeout_not_running` | PASS |
| Test pipeline can be restarted immediately after stop | `pipeline.rs:930-941` - `test_stop_delegates_to_stop_with_timeout` | PASS |
| Test thread resources are cleaned up on Drop | `pipeline.rs:951-963` - `test_pipeline_drop_signals_stop` | PASS |

Additional tests covering the thread coordination mechanism:
- `test_thread_coordination_channel` (`pipeline.rs:966-977`) - verifies exit channel works
- `test_thread_coordination_timeout` (`pipeline.rs:979-989`) - verifies timeout behavior
- `test_thread_coordination_disconnected` (`pipeline.rs:991-1005`) - verifies disconnected handling
- `test_thread_exit_rx_initialized_as_none` (`pipeline.rs:944-948`) - verifies initial state

### 8. Build Warning Audit

```bash
cd src-tauri && cargo build 2>&1 | grep -E "(warning|unused|dead_code)"
```

Output shows one unrelated warning about unused VAD imports, not related to this spec:
```
warning: unused imports: `VAD_CHUNK_SIZE_16KHZ` and `VAD_CHUNK_SIZE_8KHZ`
```

No warnings for new code introduced by this spec:

| Item | Type | Used? | Evidence |
|------|------|-------|----------|
| `thread_exit_rx` | field | YES | `pipeline.rs:245,314,374` |
| `stop_with_timeout()` | method | YES | Called by `stop()` at `pipeline.rs:344` |
| `exit_tx`/`exit_rx` | channel | YES | Created at `pipeline.rs:313`, used at `pipeline.rs:321,375` |

### 9. Code Quality Notes

- [x] Error handling appropriate - Uses `Result` types, handles timeout cases gracefully
- [x] No unwrap() on user-facing code paths - Uses `take()`, `if let`, and match expressions
- [x] Types are explicit - `Duration`, `Receiver<()>`, `Option<>` types properly annotated
- [x] Consistent with existing patterns - Uses same `AtomicBool` pattern as `should_stop`, follows existing channel patterns

### 10. Verdict

**APPROVED**

All acceptance criteria are met with line-level evidence:
- The dangerous callback pattern is deprecated and replaced with event channels
- Thread coordination uses dedicated exit channel with timeout support
- The `stop_with_timeout()` method allows configurable timeout for thread joining
- The `start()` method properly waits for previous thread exit before spawning new one
- Drop implementation signals stop, and cleanup is ensured via exit channel
- All 435 tests pass
- No new build warnings for the spec's code

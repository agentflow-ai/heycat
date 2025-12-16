---
status: completed
created: 2025-12-16
completed: 2025-12-16
dependencies: []
review_round: 1
priority: P0
---

# Spec: Make event subscription mandatory before pipeline start

## Description

Currently if `subscribe_events()` isn't called before `start()`, a channel is created but events are silently dropped (lines 280-284). This is a common source of missed wake word detections. The code only logs a warning but continues.

Make event subscription mandatory by returning an error from `start()` if no subscriber is configured.

## Acceptance Criteria

- [ ] `start()` returns error if `subscribe_events()` was not called
- [ ] Error message clearly states "must call subscribe_events() before start()"
- [ ] Remove the silent channel creation in `start()`
- [ ] Existing callers updated to call `subscribe_events()` before `start()`
- [ ] Documentation updated to reflect the requirement

## Test Cases

- [ ] Test `start()` without `subscribe_events()` returns clear error
- [ ] Test `start()` after `subscribe_events()` succeeds
- [ ] Test events are actually delivered to subscriber
- [ ] Test calling `subscribe_events()` multiple times is safe

## Dependencies

None

## Preconditions

- Events-based wake word detection already implemented

## Implementation Notes

**File:** `src-tauri/src/listening/pipeline.rs`

**Current behavior (lines 280-284):**
```rust
let event_tx = self.event_tx.clone().unwrap_or_else(|| {
    crate::warn!("[pipeline] No event subscriber configured, events will be dropped");
    let (tx, _rx) = tokio_mpsc::channel(EVENT_CHANNEL_BUFFER_SIZE);
    tx  // Receiver is dropped immediately!
});
```

**Proposed change:**
```rust
pub fn start<E: 'static>(
    &mut self,
    audio_handle: &AudioThreadHandle,
    emitter: Arc<E>,
) -> Result<(), WakeWordError> {
    let event_tx = self.event_tx.clone()
        .ok_or(WakeWordError::NoEventSubscriber)?;
    // ... rest of start logic
}
```

**Add error variant:**
```rust
pub enum WakeWordError {
    // ... existing variants
    NoEventSubscriber,  // NEW
}
```

**Update Display:**
```rust
WakeWordError::NoEventSubscriber => write!(
    f, "Must call subscribe_events() before start()"
),
```

**Callers to update:**
- `src-tauri/src/listening/manager.rs` - ListeningManager::enable_listening()

## Related Specs

- safe-callback-channel.spec.md (completed - introduced events)
- thread-coordination-fix.spec.md (complementary)

## Integration Points

- Production call site: `src-tauri/src/listening/manager.rs`
- Connects to: ListeningManager

## Integration Test

- Test location: `src-tauri/src/listening/pipeline.rs` (test module)
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-16
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `start()` returns error if `subscribe_events()` was not called | PASS | `pipeline.rs:261-262` - `if self.event_tx.is_none() { return Err(PipelineError::NoEventSubscriber); }` |
| Error message clearly states "must call subscribe_events() before start()" | PASS | `pipeline.rs:50-52` - `PipelineError::NoEventSubscriber => { write!(f, "Must call subscribe_events() before start()") }` |
| Remove the silent channel creation in `start()` | PASS | `pipeline.rs:300-301` - Now uses `self.event_tx.clone().expect()` after guard check, no fallback channel creation |
| Existing callers updated to call `subscribe_events()` before `start()` | PASS | `commands/mod.rs:316-322` - `enable_listening` calls `pipeline.subscribe_events()` before `enable_listening_impl()` which calls `start()`. Coordinator restarts work because `event_tx` persists across stop/start cycles (verified: `stop()` at lines 359-360 only clears `detector` and `buffer`, not `event_tx`) |
| Documentation updated to reflect the requirement | PASS | `pipeline.rs:196-200` - Docstring on `subscribe_events()` states "**This MUST be called before `start()`.** If not called, `start()` will return `PipelineError::NoEventSubscriber`." Also documented in `start()` at `pipeline.rs:233` |

### Integration Path Trace

```
[UI: Enable Listening Toggle]
     |
     v
[enable_listening command]--invoke()-->[commands/mod.rs:303]
     |
     v
[subscribe_events()]---------------->[pipeline.rs:206-209] Sets event_tx
     |
     v
[enable_listening_impl()]------------>[logic.rs:519] Calls pipeline.start()
     |
     v
[pipeline.start()]------------------>[pipeline.rs:261-262] Checks event_tx.is_some()
     |
     v
[Analysis thread created]------------>[pipeline.rs:318-322] Uses event_tx
     |
     v
[Wake word detected]---------------->[pipeline.rs:554-558] try_send(event)
     |
     <----recv()--------------------[commands/mod.rs:386] handle_wake_word_events async task
     |
     v
[handle_wake_word_detected()]-------->[commands/mod.rs:393-400]
```

### Integration Verification Table

| Step | Expected | Actual Location | Status |
|------|----------|-----------------|--------|
| subscribe_events() called | Before start() | `commands/mod.rs:321` | PASS |
| NoEventSubscriber error type | PipelineError variant | `pipeline.rs:39` | PASS |
| Guard check in start() | Early return if None | `pipeline.rs:261-262` | PASS |
| event_tx persists | Not cleared in stop() | `pipeline.rs:359-360` (only clears detector, buffer) | PASS |
| Coordinator restart works | event_tx still Some | `coordinator.rs:334,368` (start() succeeds with existing tx) | PASS |

### Registration Audit

| Item | Type | Registered? | Evidence |
|------|------|-------------|----------|
| PipelineError::NoEventSubscriber | error variant | YES | `pipeline.rs:39` |
| NoEventSubscriber Display impl | trait impl | YES | `pipeline.rs:50-52` |

### Mock-to-Production Audit

| Mock | Test Location | Production Counterpart | Production Instantiation |
|------|---------------|----------------------|-------------------------|
| MockEventEmitter | `pipeline.rs:765,785` | TauriEventEmitter | `commands/mod.rs:312` |

### Event Subscription Audit

| Event Name | Emission Location | Frontend Listener | Listener Location |
|------------|-------------------|-------------------|-------------------|
| WakeWordEvent::Detected | `pipeline.rs:554-558` | YES (via async handler) | `commands/mod.rs:386-400` |
| WakeWordEvent::Unavailable | `pipeline.rs:500,591` | YES | `commands/mod.rs:386` (same handler) |
| WakeWordEvent::Error | `pipeline.rs:597` | YES | `commands/mod.rs:386` (same handler) |

### Deferral Tracking

No TODOs, FIXMEs, or deferred work found in the implementation.

### Test Coverage Audit

| Test Case (from spec) | Test Location | Status |
|----------------------|---------------|--------|
| Test `start()` without `subscribe_events()` returns clear error | `pipeline.rs:764-776` `test_start_without_subscribe_events_returns_error` | PASS |
| Test `start()` after `subscribe_events()` succeeds | `pipeline.rs:779-803` `test_start_after_subscribe_events_proceeds` | PASS |
| Test events are actually delivered to subscriber | `pipeline.rs:833-847` `test_event_channel_send_receive`, `pipeline.rs:850-866` `test_event_channel_multiple_events` | PASS |
| Test calling `subscribe_events()` multiple times is safe | `pipeline.rs:824-829` `test_subscribe_events_replaces_previous` | PASS |

Additional test coverage:
- `pipeline.rs:806-821` `test_event_tx_persists_after_stop` - Verifies subscription persists across stop/start cycles
- `pipeline.rs:869-879` `test_event_channel_try_send_backpressure` - Verifies backpressure handling

### Build Warning Audit

**Backend (Rust):** `cargo build` produces 2 warnings unrelated to this spec:
- `OPTIMAL_CHUNK_DURATION_MS` never used (audio_constants.rs:22)
- `chunk_size_for_sample_rate` never used (audio_constants.rs:174)

**New code from this spec:**

| Item | Type | Used? | Evidence |
|------|------|-------|----------|
| PipelineError::NoEventSubscriber | enum variant | YES | `pipeline.rs:262` returns it, `pipeline.rs:775` tests for it |
| NoEventSubscriber Display | impl block | YES | Part of Display trait, automatically used by error formatting |

No unused code warnings for new code introduced by this spec.

### Code Quality

**Strengths:**
- Early validation pattern (check at start() entry, fail fast)
- Clear error message guides developers to correct usage
- Subscription persists across stop/start cycles (documented behavior)
- Comprehensive test coverage including edge cases

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria verified with line-level evidence. The implementation correctly makes event subscription mandatory by returning `PipelineError::NoEventSubscriber` if `subscribe_events()` was not called before `start()`. The error message is clear and actionable. The production caller in `enable_listening` properly subscribes before starting. The coordinator restart path works because `event_tx` is preserved across stop/start cycles (intentional design documented in docstring). All test cases pass and provide comprehensive coverage.

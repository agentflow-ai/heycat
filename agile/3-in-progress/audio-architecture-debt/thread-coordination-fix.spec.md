---
status: in-progress
created: 2025-12-16
completed: null
dependencies: []
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

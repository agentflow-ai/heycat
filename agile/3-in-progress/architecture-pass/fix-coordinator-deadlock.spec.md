---
status: pending
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Fix potential deadlock in coordinator.rs by establishing consistent lock ordering

## Description

Fix a potential deadlock in `detection_loop` where `recording_manager` lock is re-acquired after being dropped, while holding `listening_pipeline` lock. If `pipeline.start()` internally tries to acquire `recording_manager`, this creates a deadlock. The lock ordering is also inconsistent (sometimes pipeline first, sometimes recording_manager first).

**Severity:** Critical

## Acceptance Criteria

- [ ] All state transitions on `recording_manager` complete BEFORE dropping the lock
- [ ] No re-acquisition of `recording_manager` after `drop(manager)`
- [ ] Lock ordering is consistent: always acquire `recording_manager` before `listening_pipeline`
- [ ] `cargo test` passes with no deadlock-related failures
- [ ] Manual test: wake word detection -> recording -> silence detection -> returns to listening (no hang)

## Test Cases

- [ ] Unit test: `detection_loop` transitions to final state before dropping manager
- [ ] Integration test: Full wake word flow completes without hanging
- [ ] Stress test (optional): Rapid start/stop of listening doesn't deadlock

## Dependencies

None

## Preconditions

- Understanding of the current lock acquisition order in `detection_loop`
- Familiarity with `RecordingManager` state machine

## Implementation Notes

**Current problematic code (coordinator.rs:327-378):**

```rust
// 5. Transition to final state and restart listening if needed
drop(manager);  // Line 327 - dropped here

// 6. Restart listening pipeline if return_to_listening
if return_to_listening {
    if let Some(ref pipeline_arc) = listening_pipeline {
        if let Ok(mut pipeline) = pipeline_arc.lock() {
            // First transition state to Listening
            if let Ok(mut rm) = recording_manager.lock() {  // <-- Re-acquired!
                let _ = rm.transition_to(RecordingState::Listening);
            }
```

**Suggested fix:**

```rust
// Transition to final state BEFORE dropping manager
let target_state = if return_to_listening {
    RecordingState::Listening
} else {
    RecordingState::Idle
};
if let Err(e) = manager.transition_to(target_state) {
    crate::error!("[coordinator] Failed to transition: {:?}", e);
}
drop(manager);  // Now drop after all state transitions

// Then restart pipeline without needing the manager lock
if return_to_listening {
    if let Some(ref pipeline_arc) = listening_pipeline {
        if let Ok(mut pipeline) = pipeline_arc.lock() {
            match pipeline.start(&audio_thread, emitter.clone()) {
                Ok(_) => crate::info!("[coordinator] Listening pipeline restarted"),
                Err(e) => crate::error!("[coordinator] Failed to restart: {:?}", e),
            }
        }
    }
}
```

**Files to modify:**
- `src-tauri/src/listening/coordinator.rs` (lines 254-380)

## Related Specs

None

## Integration Points

- Production call site: `src-tauri/src/listening/coordinator.rs:107` (spawned from `start_monitoring`)
- Connects to: `RecordingManager`, `ListeningPipeline`, `AudioThreadHandle`

## Integration Test

- Test location: Manual verification through wake word flow
- Verification: [ ] Integration test passes

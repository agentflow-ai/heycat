---
status: pending
created: 2025-12-22
completed: null
dependencies: []
---

# Spec: Add synchronization barrier in CpalBackend::stop() to ensure audio callback completes

## Description

Fix the progressive audio speedup bug by ensuring the cpal audio callback has fully stopped before cleanup proceeds. The root cause is a race condition where `CpalBackend::stop()` drops the stream, but the OS audio thread callback may still be running, causing state corruption between recordings.

## Acceptance Criteria

- [ ] Add pause + barrier before dropping the cpal stream in `stop()`
- [ ] Add early-exit check in `process_samples()` when `signaled` is true
- [ ] Add diagnostic logging to detect zombie callbacks (samples arriving after stream drop)
- [ ] Multiple consecutive hotkey recordings play back at correct speed (no chipmunk audio)
- [ ] Memory does not increase between recordings

## Test Cases

- [ ] Record 5 consecutive audio clips via hotkey - all should play at normal speed
- [ ] Memory usage should remain stable across 10+ recording cycles
- [ ] Log output should show no "ZOMBIE CALLBACK" warnings

## Dependencies

None

## Preconditions

- Hotkey recording must be functional
- Audio device must be available

## Implementation Notes

### Files to Modify

1. **`src-tauri/src/audio/cpal_backend.rs`** - Primary changes:
   - In `stop()`: Add `stream.pause()` before drop, add 50ms barrier after pause, add 20ms barrier after drop
   - In `process_samples()`: Add early-exit check when `signaled.load()` is true
   - Add diagnostic logging to detect samples arriving after stream drop

### Code Changes

In `CpalBackend::stop()`:
```rust
fn stop(&mut self) -> Result<(), AudioCaptureError> {
    if let Some(stream) = self.stream.take() {
        // Pause first to stop callbacks
        let _ = stream.pause();

        // Barrier for in-flight callbacks
        std::thread::sleep(Duration::from_millis(50));

        drop(stream);
    }

    // Additional barrier after drop
    std::thread::sleep(Duration::from_millis(20));

    // Verify no zombie callback
    if let Some(ref cs) = self.callback_state {
        let before = cs.input_sample_count.load(Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(10));
        let after = cs.input_sample_count.load(Ordering::SeqCst);
        if after != before {
            crate::error!("ZOMBIE CALLBACK: samples arriving after stream drop!");
        }
        cs.flush_residuals();
        cs.log_sample_diagnostics();
    }

    self.callback_state = None;
    self.state = CaptureState::Stopped;
    Ok(())
}
```

In `CallbackState::process_samples()`:
```rust
fn process_samples(&self, f32_samples: &[f32]) {
    // Early exit if signaled to stop
    if self.signaled.load(Ordering::SeqCst) {
        return;
    }
    // ... rest of existing code
}
```

## Related Specs

None - single spec for this bug

## Integration Points

- Production call site: `src-tauri/src/audio/thread.rs:181` (backend.stop() in AudioCommand::Stop handler)
- Connects to: AudioThreadHandle, HotkeyIntegration

## Integration Test

- Test location: Manual testing via hotkey recording
- Verification: [ ] Multiple consecutive recordings play back correctly

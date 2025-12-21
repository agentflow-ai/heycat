---
status: pending
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Reduce lock contention in audio callback hot path (cpal_backend.rs)

## Description

The audio callback in `CallbackState::process_samples` acquires multiple locks in sequence (`resample_buffer`, `chunk_buffer`, `resampler`, `buffer`). On high-frequency audio callbacks (~100+ times/second), this could cause contention and audio glitches.

**Severity:** Medium

## Acceptance Criteria

- [ ] Reduce number of lock acquisitions in audio callback hot path
- [ ] No audio glitches or buffer underruns during normal operation
- [ ] `cargo test` passes
- [ ] Audio capture works correctly with resampling enabled

## Test Cases

- [ ] Manual test: Record audio with device requiring resampling (non-16kHz device)
- [ ] Manual test: Monitor for audio glitches during extended recording (30+ seconds)
- [ ] Unit test: `CallbackState::process_samples` handles concurrent access correctly

## Dependencies

None

## Preconditions

- Understanding of cpal audio callback model
- Familiarity with lock-free data structures (optional for advanced fix)

## Implementation Notes

**Current code (cpal_backend.rs:102-175):**

```rust
fn process_samples(&self, f32_samples: &[f32]) {
    let mut resample_buf = match self.resample_buffer.lock() { ... };
    // ... later
    if let Ok(mut chunk_buf) = self.chunk_buffer.lock() { ... }
    if let Ok(mut r) = resampler.lock() { ... }
    // ... later
    match self.buffer.lock() { ... }
}
```

**Actions:**

1. **Advanced change:** Use lock-free ring buffer
   - Replace `Arc<Mutex<Vec<f32>>>` with `ringbuf` crate or similar
   - Producer (callback) writes without locking
   - Consumer reads without blocking producer

**Recommended approach:** Implement the ring buffer.

**Files to modify:**
- `src-tauri/src/audio/cpal_backend.rs` (lines 84-176)
note: not exhaustive list, use your reasoning

## Related Specs

None

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:291-328` (cpal stream callbacks)
- Connects to: `AudioThreadHandle`, recording flow

## Integration Test

- Test location: Manual audio recording test
- Verification: [ ] No audio glitches during extended recording

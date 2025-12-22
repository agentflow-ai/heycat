---
last-updated: 2025-12-22
status: draft
---

# Technical Guidance: Progressive Audio Speedup

## Root Cause Analysis

The progressive audio speedup is caused by the rubato `FftFixedIn` resampler's handling of residual samples when recording stops.

**Key Observations:**
- First recording after app launch is normal speed
- Each subsequent recording plays progressively faster
- Issue only occurs when resampling is active (device doesn't support 16kHz)
- Using hotkey only - wake word/listening pipeline NOT involved

**Technical Analysis:**
The `process_samples()` function in `cpal_backend.rs` accumulates audio samples in `resample_buf` and processes them in chunks of `RESAMPLE_CHUNK_SIZE` (1024 samples). When recording stops, any samples remaining in the buffer (< 1024) are discarded when `CallbackState` is dropped.

This causes:
1. Sample count drift between input and output
2. Potential phase/timing artifacts from FFT windowing state
3. Compounding errors across multiple recordings

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/audio/cpal_backend.rs` | Resampler creation, sample processing, stop handling |
| `src-tauri/src/audio_constants.rs` | RESAMPLE_CHUNK_SIZE constant (1024) |
| `src-tauri/Cargo.toml` | rubato = "0.15" dependency |

## Fix Approach

### Phase 1: Diagnostic Logging
Add sample count tracking to verify the issue:
```rust
// In CallbackState, add atomic counters
input_sample_count: Arc<AtomicUsize>,
output_sample_count: Arc<AtomicUsize>,
```

### Phase 2: Flush Residual Samples
When recording stops, flush remaining samples instead of discarding:
```rust
// Zero-pad final chunk and process
if resample_buf.len() > 0 && resample_buf.len() < chunk_size {
    resample_buf.resize(chunk_size, 0.0);
    let output = resampler.process(&[&resample_buf], None)?;
    // Push output samples (trim trailing zeros)
}
```

### Phase 3: Verify Fix
Compare sample ratios across 10+ recordings to confirm consistency.

## Regression Risk

| Risk | Mitigation |
|------|------------|
| Zero-padding may add silence at end | Acceptable - minimal impact vs speedup |
| Changing resampler behavior | Test transcription quality |
| Performance impact of logging | Use atomic counters, debug-only logging |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-22 | First recording normal, subsequent faster | Confirms accumulating state |
| 2025-12-22 | Issue with hotkey only (no wake word) | Isolates to CpalBackend/resampler |
| 2025-12-22 | Device uses resampling (48kHz → 16kHz) | Confirms resampler involvement |
| 2025-12-22 | Residual samples discarded in process_samples() | Root cause identified |

## Open Questions

- [x] Does issue occur with wake word? No - hotkey only confirms it
- [x] Is resampling active? Yes - device doesn't support 16kHz
- [ ] What is actual sample ratio drift per recording?
- [ ] Does rubato have a reset/flush API?

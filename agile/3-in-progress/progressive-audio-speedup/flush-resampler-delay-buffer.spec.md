---
status: pending
created: 2025-12-22
completed: null
dependencies: ["flush-residual-samples"]
---

# Spec: Flush resampler internal delay buffer when recording stops

## Description

The rubato `FftFixedIn` resampler has an internal delay buffer (`output_delay()` frames) that holds samples during FFT processing. The current `flush_residuals()` only processes samples in our accumulation buffer but does NOT flush the resampler's internal state.

This causes each recording to lose `output_delay()` samples (typically 100-500 frames), resulting in progressive audio speedup across multiple recordings.

## Root Cause

The current code uses `process()` with zero-padding, but rubato requires `process_partial(None, None)` to extract samples from its internal delay buffer.

From rubato documentation:
> "This [process_partial] can also be called without any input frames, by providing `None` as input buffer. This is used to push any remaining delayed frames out from the internal buffers."

## Acceptance Criteria

- [ ] `flush_residuals()` calls `process_partial()` instead of `process()` for residual samples
- [ ] After processing residuals, call `process_partial(None, None)` to flush delay buffer
- [ ] Log `output_delay()` value when resampler is created
- [ ] Log number of samples flushed from delay buffer
- [ ] Sample ratio error < 0.5% after flush (vs current ~1-2% error per recording)
- [ ] No progressive speedup after 10+ consecutive recordings

## Test Cases

- [ ] Unit test: `process_partial(None)` extracts samples from delay buffer
- [ ] Unit test: Sample ratio within 0.5% after proper flushing
- [ ] Manual: 10 consecutive recordings play at consistent speed

## Implementation Notes

**File:** `src-tauri/src/audio/cpal_backend.rs`

Replace `flush_residuals()`:

```rust
fn flush_residuals(&self) {
    let Some(ref resampler) = self.resampler else {
        return;
    };

    let mut resample_buf = match self.resample_buffer.lock() {
        Ok(buf) => buf,
        Err(_) => return,
    };

    if let Ok(mut r) = resampler.lock() {
        // Step 1: Process any remaining samples using process_partial
        let residual_count = resample_buf.len();
        if residual_count > 0 {
            crate::debug!("Flushing {} residual samples via process_partial", residual_count);
            if let Ok(output) = r.process_partial(Some(&[resample_buf.as_slice()]), None) {
                if !output.is_empty() && !output[0].is_empty() {
                    self.output_sample_count.fetch_add(output[0].len(), Ordering::Relaxed);
                    self.buffer.push_samples(&output[0]);
                }
            }
            resample_buf.clear();
        }

        // Step 2: Flush the resampler's internal delay buffer (CRITICAL)
        crate::debug!("Flushing resampler delay buffer (delay={} frames)", r.output_delay());
        if let Ok(output) = r.process_partial(None::<&[&[f32]]>, None) {
            if !output.is_empty() && !output[0].is_empty() {
                let flushed = output[0].len();
                self.output_sample_count.fetch_add(flushed, Ordering::Relaxed);
                self.buffer.push_samples(&output[0]);
                crate::debug!("Flushed {} samples from delay buffer", flushed);
            }
        }
    }
}
```

Add logging in `start()` after resampler creation:
```rust
crate::info!("Resampler created: {}Hz -> {}Hz, output_delay={} frames",
    device_sample_rate, TARGET_SAMPLE_RATE, r.output_delay());
```

## Dependencies

- `flush-residual-samples.spec.md` - existing flush mechanism to enhance

## Integration Points

- Production call site: `cpal_backend.rs:stop()` → `flush_residuals()`
- Connects to: Rubato resampler API

## Integration Test

Manual verification:
1. Make 10+ consecutive recordings
2. Check logs for `output_delay` and samples flushed
3. Verify sample ratio error < 0.5%
4. Play back recordings - all should be same speed

- Test location: Manual testing
- Verification: [ ] Integration test passes

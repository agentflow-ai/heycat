---
status: completed
created: 2025-12-22
completed: 2025-12-22
dependencies: []
review_round: 1
---

# Spec: Add atomic counters to track input/output sample counts in CallbackState

## Description

Add atomic counters to `CallbackState` in `cpal_backend.rs` to track the total number of input samples received from the device and output samples produced by the resampler. Log the sample counts and ratio at the end of each recording to diagnose the progressive speedup issue.

## Acceptance Criteria

- [ ] `CallbackState` has `input_sample_count: Arc<AtomicUsize>` field
- [ ] `CallbackState` has `output_sample_count: Arc<AtomicUsize>` field
- [ ] `process_samples()` increments input counter with `f32_samples.len()`
- [ ] `process_samples()` increments output counter with `samples_to_add.len()`
- [ ] Sample counts and ratio logged when recording stops (info level)
- [ ] Log format includes: input samples, output samples, actual ratio, expected ratio

## Test Cases

- [ ] After recording, logs show input/output sample counts
- [ ] Ratio logged matches expected (16000 / device_rate) within 1%
- [ ] Counters reset to 0 for each new recording

## Dependencies

None

## Preconditions

Device requires resampling (doesn't support 16kHz natively)

## Implementation Notes

**File:** `src-tauri/src/audio/cpal_backend.rs`

1. Add to `CallbackState` struct (line ~86-94):
```rust
input_sample_count: Arc<AtomicUsize>,
output_sample_count: Arc<AtomicUsize>,
```

2. In `process_samples()` (line ~101):
```rust
self.input_sample_count.fetch_add(f32_samples.len(), Ordering::Relaxed);
// ... after resampling ...
self.output_sample_count.fetch_add(samples_to_add.len(), Ordering::Relaxed);
```

3. Initialize counters in `start()` (line ~275-283):
```rust
input_sample_count: Arc::new(AtomicUsize::new(0)),
output_sample_count: Arc::new(AtomicUsize::new(0)),
```

4. Log in `stop()` or via a mechanism to access counters before CallbackState is dropped

## Related Specs

- `flush-residual-samples.spec.md` - depends on this spec

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:start()` creates CallbackState
- Connects to: Audio thread, recording state

## Integration Test

N/A - diagnostic logging verified via manual testing and log inspection

- Test location: N/A (debug/diagnostic feature)
- Verification: [x] N/A

## Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `CallbackState` has `input_sample_count: Arc<AtomicUsize>` field | PASS | cpal_backend.rs:98 |
| `CallbackState` has `output_sample_count: Arc<AtomicUsize>` field | PASS | cpal_backend.rs:100 |
| `process_samples()` increments input counter with `f32_samples.len()` | PASS | cpal_backend.rs:112 |
| `process_samples()` increments output counter with `samples_to_add.len()` | PASS | cpal_backend.rs:177 |
| Sample counts and ratio logged when recording stops (info level) | PASS | cpal_backend.rs:258 calls `crate::info!()` |
| Log format includes: input samples, output samples, actual ratio, expected ratio | PASS | cpal_backend.rs:258-260 format string includes all |
| Counters reset to 0 for each new recording | PASS | cpal_backend.rs:371-372 new CallbackState with AtomicUsize::new(0) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| After recording, logs show input/output sample counts | N/A | Diagnostic verified via manual log inspection |
| Ratio logged matches expected (16000 / device_rate) within 1% | N/A | Manual verification |
| Counters reset to 0 for each new recording | PASS | cpal_backend.rs:371-372 (new CallbackState per recording) |
| test_resampler_produces_output_after_warmup | PASS | cpal_backend.rs:480 |
| test_sample_ratio_converges | PASS | cpal_backend.rs:503 |
| test_flush_with_empty_buffer | PASS | cpal_backend.rs:533 |
| test_buffer_cleared_after_flush | PASS | cpal_backend.rs:563 |
| test_flush_residuals_does_not_panic | PASS | cpal_backend.rs:604 |

### Pre-Review Gate Results

```
Build Warning Check: PASS (no new warnings in cpal_backend.rs - existing warning is in dictionary/store.rs:218, unrelated)
Command Registration Check: N/A (no new Tauri commands)
Event Subscription Check: N/A (no new events)
```

### Code Quality

**Strengths:**
- Clean separation of concerns with `log_sample_diagnostics()` method
- Atomic counters allow safe concurrent access from audio callback
- Informative log format with ratio error percentage for quick diagnosis
- Counters naturally reset via fresh CallbackState per recording
- `flush_residuals()` method properly handles edge cases (empty buffer, partial chunks)
- Tests correctly account for FFT resampler latency behavior (warmup period)
- Well-documented code with clear comments explaining the flow

**Concerns:**
- None identified

### Integration Points Verification

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `input_sample_count` | field | cpal_backend.rs:371 (start()) | YES (via audio capture) |
| `output_sample_count` | field | cpal_backend.rs:372 (start()) | YES (via audio capture) |
| `flush_residuals()` | fn | cpal_backend.rs:457 (stop()) | YES (via stop recording) |
| `log_sample_diagnostics()` | fn | cpal_backend.rs:458 (stop()) | YES (via stop recording) |

### Verdict

**APPROVED** - Implementation correctly adds sample count diagnostics with proper atomic counters, integrates flush mechanism on stop, and includes comprehensive tests that account for FFT resampler latency behavior.

---
status: pending
created: 2025-12-22
completed: null
dependencies: ["flush-residual-samples"]
---

# Spec: Add unit tests for resampler residual sample handling

## Description

Add unit tests for the resampler residual sample flushing logic to prevent regression. Tests should verify that all input samples are properly processed through the resampler, including partial final chunks.

## Acceptance Criteria

- [ ] Test for residual sample flush when buffer has partial chunk
- [ ] Test for sample count ratio accuracy (output/input within 0.1% of expected)
- [ ] Test for zero residual samples remaining after flush
- [ ] Tests pass in CI pipeline

## Test Cases

- [ ] `test_resampler_flushes_partial_chunk`: Input 1500 samples (chunk_size=1024), verify all processed
- [ ] `test_sample_ratio_consistency`: Multiple process calls, verify ratio remains constant
- [ ] `test_no_residual_after_flush`: After flush, verify resample_buf is empty
- [ ] `test_flush_with_empty_buffer`: Edge case - flush when no residuals

## Dependencies

- `flush-residual-samples.spec.md` - need the flush implementation to test

## Preconditions

- Flush residual samples implementation complete
- rubato resampler available for testing

## Implementation Notes

**File:** `src-tauri/src/audio/cpal_backend.rs` (add tests module) or separate test file

Since `cpal_backend.rs` has `#![cfg_attr(coverage_nightly, coverage(off))]` (hardware interaction), consider:

1. **Option A**: Extract resampling logic to testable helper functions
2. **Option B**: Create mock-based tests that verify the flush logic
3. **Option C**: Add integration-style tests that verify sample counts

Example test structure:
```rust
#[cfg(test)]
mod resampler_tests {
    use super::*;
    use rubato::{FftFixedIn, Resampler};

    #[test]
    fn test_resampler_flushes_partial_chunk() {
        let mut resampler = FftFixedIn::<f32>::new(48000, 16000, 1024, 1, 1).unwrap();
        let input = vec![0.5f32; 1500]; // 1024 + 476 residual

        // Process first chunk
        let chunk1: Vec<f32> = input[..1024].to_vec();
        let output1 = resampler.process(&[&chunk1], None).unwrap();

        // Flush residual (zero-padded)
        let mut residual = input[1024..].to_vec();
        residual.resize(1024, 0.0);
        let output2 = resampler.process(&[&residual], None).unwrap();

        // Verify total output samples match expected ratio
        let total_output = output1[0].len() + output2[0].len();
        let expected = (1500.0 * 16000.0 / 48000.0) as usize;
        assert!((total_output as i32 - expected as i32).abs() < 10);
    }
}
```

## Related Specs

- `flush-residual-samples.spec.md` - tests the implementation from this spec

## Integration Points

- Production call site: N/A (unit tests)
- Connects to: rubato resampler library

## Integration Test

N/A - these are unit tests for the resampling logic

- Test location: `src-tauri/src/audio/cpal_backend.rs` or separate test file
- Verification: [x] N/A (unit tests)

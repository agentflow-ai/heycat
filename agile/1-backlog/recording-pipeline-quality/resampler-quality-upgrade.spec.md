---
status: pending
created: 2025-12-23
completed: null
dependencies: ["channel-mixing"]
---

# Spec: Replace FFT resampler with higher-quality alternative

## Description

Replace the current `FftFixedIn` resampler with a higher-quality sinc-based resampler to reduce artifacts in voice recordings. The FFT-based resampler can introduce artifacts that make speech sound robotic or unnatural. The rubato library provides `SincFixedIn` which uses sinc interpolation for better quality.

## Acceptance Criteria

- [ ] Replace `FftFixedIn` with `SincFixedIn` from rubato crate
- [ ] Configure sinc resampler with appropriate quality settings for voice (e.g., 128-256 sinc length)
- [ ] Maintain chunk-based processing model for real-time operation
- [ ] Handle residual sample flushing correctly at end of recording
- [ ] Latency increase (if any) is acceptable (< 50ms additional)
- [ ] No audible artifacts in resampled voice recordings

## Test Cases

- [ ] Resampling 48kHz → 16kHz produces clean output (no artifacts audible)
- [ ] Resampling 44.1kHz → 16kHz produces clean output
- [ ] A/B comparison with FFT resampler shows improved voice clarity
- [ ] Sample count ratio matches expected (within 1% of source/target ratio)
- [ ] No samples lost during flush at end of recording
- [ ] Performance: resampling completes within callback time budget

## Dependencies

- `channel-mixing` - resampler receives properly mixed mono input

## Preconditions

- Audio capture pipeline is functional with existing resampler
- Rubato crate is already a dependency

## Implementation Notes

- `SincFixedIn::new()` takes: `resample_ratio`, `max_resample_ratio_relative`, `params: SincInterpolationParameters`, `chunk_size`, `channels`
- `SincInterpolationParameters` controls quality: `sinc_len`, `f_cutoff`, `oversampling_factor`, `interpolation`, `window`
- Start with default parameters, tune if needed
- Key change is in `create_resampler()` function in `cpal_backend.rs`
- May need to adjust `RESAMPLE_CHUNK_SIZE` based on sinc resampler requirements

## Related Specs

- `channel-mixing.spec.md` - this spec depends on channel mixing
- `audio-gain-normalization.spec.md` - depends on this (gain applied after resampling)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:create_resampler()`
- Connects to: Channel mixer → **Resampler** → Denoiser → Buffer

## Integration Test

- Test location: `src-tauri/src/audio/cpal_backend.rs` or dedicated resampler test
- Verification: [ ] Integration test passes

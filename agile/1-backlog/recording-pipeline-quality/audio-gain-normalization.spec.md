---
status: pending
created: 2025-12-23
completed: null
dependencies: ["resampler-quality-upgrade", "audio-preprocessing"]
---

# Spec: Automatic gain control for consistent volume levels

## Description

Implement automatic gain control (AGC) to normalize recording volume levels. Quiet recordings are a major quality issue - users shouldn't need to be close to the microphone or speak loudly. AGC analyzes incoming audio levels and applies adaptive gain to maintain consistent output volume while preventing clipping.

## Acceptance Criteria

- [ ] Track peak and RMS levels of incoming audio in real-time
- [ ] Apply gain adjustment to boost quiet signals toward target level (e.g., -12dBFS RMS)
- [ ] Use attack/release envelope to avoid pumping artifacts on transients
- [ ] Implement soft limiter to prevent clipping when gain is applied
- [ ] AGC operates after denoising (clean signal) before buffer storage
- [ ] AGC can be enabled/disabled via settings
- [ ] Maximum gain limit to prevent amplifying noise floor excessively (e.g., 20dB max)

## Test Cases

- [ ] Quiet input (-30dBFS) is boosted to target level (-12dBFS)
- [ ] Normal input (-12dBFS) passes with minimal gain change
- [ ] Loud input (0dBFS) is not clipped, soft limiting engages
- [ ] Fast transient (hand clap) doesn't cause pumping
- [ ] Silence doesn't cause gain to ramp to maximum
- [ ] AGC state resets correctly between recordings
- [ ] Disabled AGC produces identical output to input

## Dependencies

- `resampler-quality-upgrade` - AGC operates on resampled audio
- `audio-preprocessing` - AGC operates on filtered audio

## Preconditions

- Resampler and preprocessing specs are implemented
- Audio pipeline produces consistent 16kHz mono output

## Implementation Notes

- Create `src-tauri/src/audio/agc.rs` module
- Key parameters:
  - Target RMS level: -12dBFS (configurable)
  - Attack time: ~10ms (fast response to loud sounds)
  - Release time: ~100-200ms (smooth gain recovery)
  - Max gain: +20dB
  - Soft limit threshold: -3dBFS
- Track envelope using exponential moving average
- Apply gain: `output = input * gain` where gain adjusts smoothly
- Soft limiter: use tanh or similar sigmoid function above threshold
- Consider: compute gain per chunk rather than per-sample for efficiency

## Related Specs

- `resampler-quality-upgrade.spec.md` - AGC receives resampled audio
- `audio-preprocessing.spec.md` - AGC receives filtered audio
- `recording-diagnostics.spec.md` - diagnostics tracks AGC gain levels

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:process_samples()`
- Connects to: Resampler → Denoiser → **AGC** → Buffer

## Integration Test

- Test location: `src-tauri/src/audio/agc.rs` (unit tests) + integration test in cpal_backend
- Verification: [ ] Integration test passes

---
status: pending
created: 2025-12-23
completed: null
dependencies: ["channel-mixing"]
---

# Spec: Voice-optimized preprocessing chain

## Description

Add a voice-optimized preprocessing stage to the audio pipeline. This includes a highpass filter to remove low-frequency rumble (HVAC, traffic, handling noise) and optional pre-emphasis to boost higher frequencies important for speech intelligibility. These filters run early in the pipeline (after channel mixing, before resampling) for maximum effectiveness.

## Acceptance Criteria

- [ ] Implement highpass filter at ~80Hz cutoff to remove rumble
- [ ] Use IIR biquad filter for efficiency (Butterworth or similar)
- [ ] Filter operates on mono audio after channel mixing
- [ ] Filter state is preserved between callbacks (stateful IIR)
- [ ] Minimal latency added by filter (< 5ms)
- [ ] Filter can be bypassed via configuration flag

## Test Cases

- [ ] Highpass filter removes 50Hz test tone completely
- [ ] Highpass filter passes 200Hz test tone with minimal attenuation (< 1dB)
- [ ] Speech frequencies (300Hz-3kHz) pass through unchanged
- [ ] No audible ringing or artifacts on transient signals
- [ ] Filter state resets correctly between recording sessions
- [ ] Bypassed filter produces identical output to input

## Dependencies

- `channel-mixing` - preprocessing receives mono audio from channel mixer

## Preconditions

- Audio capture pipeline is functional
- Channel mixing is implemented (spec dependency)

## Implementation Notes

- Create `src-tauri/src/audio/preprocessing.rs` module
- Use `biquad` crate for efficient IIR filter implementation
- Highpass filter: 2nd-order Butterworth, cutoff 80Hz, sample rate 16kHz (or device rate)
- Store filter state in `CallbackState` struct
- Insert preprocessing call after channel mixing, before resampling
- Consider: could also operate at native sample rate before resampling (test both)

## Related Specs

- `channel-mixing.spec.md` - this spec depends on channel mixing
- `audio-gain-normalization.spec.md` - depends on this (gain applied to filtered signal)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:process_samples()`
- Connects to: Channel mixer → **Preprocessing** → Resampler → Denoiser → Buffer

## Integration Test

- Test location: `src-tauri/src/audio/preprocessing.rs` (unit tests) + integration test in cpal_backend
- Verification: [ ] Integration test passes

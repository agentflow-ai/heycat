---
status: in-progress
created: 2025-12-23
completed: null
dependencies: ["channel-mixing"]
---

# Spec: Voice-optimized preprocessing chain

## Description

Add a voice-optimized preprocessing stage to the audio pipeline. This includes:
1. **Highpass filter** (~80Hz) to remove low-frequency rumble (HVAC, traffic, handling noise)
2. **Pre-emphasis filter** (coefficient 0.97) to boost higher frequencies important for speech intelligibility

These filters run early in the pipeline (after channel mixing, before resampling) for maximum effectiveness. Pre-emphasis addresses "muffled" audio by boosting frequencies above ~300Hz where consonants and speech clarity reside.

## Acceptance Criteria

### Highpass Filter
- [ ] Implement highpass filter at ~80Hz cutoff to remove rumble
- [ ] Use IIR biquad filter for efficiency (Butterworth or similar)
- [ ] Filter operates on mono audio after channel mixing
- [ ] Filter state is preserved between callbacks (stateful IIR)
- [ ] Minimal latency added by filter (< 5ms)

### Pre-emphasis Filter
- [ ] Implement pre-emphasis filter: `y[n] = x[n] - alpha * x[n-1]` where alpha = 0.97
- [ ] Apply after highpass filter, before resampling
- [ ] Pre-emphasis state preserved between callbacks
- [ ] Boosts frequencies above ~300Hz for speech clarity

### Configuration
- [ ] Highpass filter can be bypassed via configuration flag
- [ ] Pre-emphasis filter can be bypassed via configuration flag (default: enabled)

## Test Cases

### Highpass Filter
- [ ] Highpass filter removes 50Hz test tone completely
- [ ] Highpass filter passes 200Hz test tone with minimal attenuation (< 1dB)
- [ ] No audible ringing or artifacts on transient signals
- [ ] Filter state resets correctly between recording sessions
- [ ] Bypassed highpass produces identical output to input

### Pre-emphasis Filter
- [ ] Pre-emphasis boosts 1kHz test tone relative to 100Hz (measurable gain difference)
- [ ] Pre-emphasis coefficient 0.97 produces expected frequency response curve
- [ ] Speech recordings sound clearer/crisper with pre-emphasis enabled
- [ ] Filter state resets correctly between recording sessions
- [ ] Bypassed pre-emphasis produces identical output to input

### Integration
- [ ] Combined filters (highpass + pre-emphasis) preserve speech quality
- [ ] Processing chain has minimal audible artifacts

## Dependencies

- `channel-mixing` - preprocessing receives mono audio from channel mixer

## Preconditions

- Audio capture pipeline is functional
- Channel mixing is implemented (spec dependency)

## Implementation Notes

### Module Structure
- Create `src-tauri/src/audio/preprocessing.rs` module
- Store filter states in `CallbackState` struct
- Insert preprocessing call after channel mixing, before resampling

### Highpass Filter
- Use `biquad` crate for efficient IIR filter implementation
- 2nd-order Butterworth, cutoff 80Hz, sample rate 16kHz (or device rate)
- Consider: could also operate at native sample rate before resampling (test both)

### Pre-emphasis Filter
- Simple first-order filter: `y[n] = x[n] - 0.97 * x[n-1]`
- No external crate needed (trivial implementation)
- Store single `prev_sample: f32` for state
- Standard ASR coefficient 0.97 boosts frequencies above ~300Hz
- Apply AFTER highpass (order: channel_mix → highpass → pre_emphasis → resample)

### Constants (add to audio_constants.rs)
```rust
pub const HIGHPASS_CUTOFF_HZ: f32 = 80.0;
pub const PRE_EMPHASIS_ALPHA: f32 = 0.97;
```

## Related Specs

- `channel-mixing.spec.md` - this spec depends on channel mixing
- `audio-gain-normalization.spec.md` - depends on this (gain applied to filtered signal)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:process_samples()`
- Pipeline order: Channel mixer → **Highpass** → **Pre-emphasis** → Resampler → Denoiser → Buffer

## Integration Test

- Test location: `src-tauri/src/audio/preprocessing.rs` (unit tests) + integration test in cpal_backend
- Verification: [ ] Integration test passes

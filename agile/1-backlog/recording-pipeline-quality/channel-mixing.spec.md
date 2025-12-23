---
status: pending
created: 2025-12-23
completed: null
dependencies: []
---

# Spec: Proper stereo-to-mono conversion for multi-channel devices

## Description

Implement proper stereo-to-mono channel mixing for audio input devices that provide multi-channel audio. Currently, the audio callback receives multi-channel data but the resampler is configured for single-channel processing, which may result in dropped channels or incorrect mixing.

## Acceptance Criteria

- [ ] Detect the number of channels from the audio device configuration
- [ ] When stereo (2 channels), mix to mono by averaging: `mono = (left + right) / 2`
- [ ] Apply -3dB gain compensation when summing channels to prevent clipping
- [ ] Handle devices with more than 2 channels (select first 2 or average all)
- [ ] Preserve mono input unchanged (no processing overhead)
- [ ] Channel mixing happens before resampling in the callback pipeline

## Test Cases

- [ ] Mono input (1 channel) passes through unchanged
- [ ] Stereo input (2 channels) is correctly mixed to mono
- [ ] Stereo sine wave at 0dB results in mono output at approximately -3dB
- [ ] Multi-channel input (4+ channels) is handled without panics
- [ ] Mixed output maintains correct sample count (input_samples / channels)

## Dependencies

None (foundational spec)

## Preconditions

- Audio capture pipeline is functional
- Access to `cpal_backend.rs` audio callback code

## Implementation Notes

- Modify `CallbackState::process_samples()` to accept raw device samples before channel mixing
- Add channel count to `CallbackState` struct
- Create `mix_to_mono(samples: &[f32], channels: usize) -> Vec<f32>` utility function
- The cpal `config.channels()` provides the channel count

## Related Specs

- `resampler-quality-upgrade.spec.md` - depends on this (resampler receives mono)
- `audio-preprocessing.spec.md` - depends on this (preprocessing operates on mono)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:process_samples()`
- Connects to: Audio callback → Channel mixer → Resampler pipeline

## Integration Test

- Test location: `src-tauri/src/audio/cpal_backend.rs` (integration test module)
- Verification: [ ] Integration test passes

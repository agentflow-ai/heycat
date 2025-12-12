---
status: pending
created: 2025-12-12
completed: null
dependencies: []
---

# Spec: 16kHz Sample Rate for Whisper

## Description

Modify the audio capture system to record at 16kHz sample rate, which is Whisper's native sample rate. This eliminates the need for real-time resampling during transcription and ensures optimal audio quality for speech recognition.

## Acceptance Criteria

- [ ] cpal_backend.rs requests 16kHz sample rate from audio device
- [ ] WAV encoding correctly uses 16kHz sample rate in header
- [ ] Fallback: rubato resampling if device doesn't support 16kHz natively
- [ ] Existing recordings (48kHz) still play correctly in external players
- [ ] Tests updated to reflect new sample rate

## Test Cases

- [ ] Audio capture starts successfully at 16kHz on supported devices
- [ ] WAV file header contains correct sample rate (16000 Hz)
- [ ] Resampling fallback activates when device doesn't support 16kHz
- [ ] Recording duration calculation remains accurate at new sample rate
- [ ] Audio quality is acceptable for speech recognition

## Dependencies

None

## Preconditions

- Audio capture system is functional (from global-hotkey-recording feature)
- cpal device can report supported configurations

## Implementation Notes

- Check if device supports 16kHz directly via `supported_configs_range()`
- If not supported, use rubato crate for high-quality resampling
- May need to update BUFFER_SIZE and other constants for 16kHz
- Whisper expects mono 16kHz f32 samples

## Related Specs

- transcription-pipeline.spec.md (consumes 16kHz audio)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs`
- Connects to: AudioThreadHandle, WAV encoder, TranscriptionManager

## Integration Test

- Test location: `src-tauri/src/audio/cpal_backend_test.rs`
- Verification: [ ] Integration test passes

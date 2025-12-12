---
status: pending
created: 2025-12-12
completed: null
dependencies:
  - transcription-pipeline
  - sample-rate-modification
---

# Spec: Auto-Transcribe on Recording Stop

## Description

Automatically start transcription when a recording stops. The transcribed text is copied to the clipboard on success. This creates the seamless voice-to-text workflow described in the feature BDD scenarios.

## Acceptance Criteria

- [ ] Transcription auto-starts when recording stops (after WAV save)
- [ ] Uses `get_last_recording_buffer()` to get audio samples for transcription
- [ ] Emits `transcription_started` event when transcription begins
- [ ] Emits `transcription_completed` event with transcribed text on success
- [ ] Emits `transcription_error` event with error message on failure
- [ ] Copies transcribed text to system clipboard on success
- [ ] Clipboard not modified on transcription error
- [ ] Frontend `useTranscription` hook listens to all transcription events

## Test Cases

- [ ] Recording stop triggers transcription automatically
- [ ] transcription_started event contains timestamp
- [ ] transcription_completed event contains transcribed text
- [ ] transcription_error event contains error message
- [ ] Clipboard contains transcribed text after success
- [ ] Clipboard unchanged after transcription failure
- [ ] useTranscription hook updates state on events

## Dependencies

- transcription-pipeline (WhisperManager.transcribe())
- sample-rate-modification (16kHz audio format)

## Preconditions

- Whisper model is loaded
- Recording has completed successfully
- Audio buffer is available via get_last_recording_buffer()

## Implementation Notes

- Integrate transcription into existing recording stop workflow
- Use Tauri's clipboard API or arboard crate for clipboard access
- Add transcription events to events.rs following existing pattern
- Consider: transcription happens in same thread or separate? (Technical guidance: transcription thread)

```rust
// Event payloads
pub struct TranscriptionStartedPayload {
    pub timestamp: String,
}

pub struct TranscriptionCompletedPayload {
    pub text: String,
    pub duration_ms: u64,
}

pub struct TranscriptionErrorPayload {
    pub error: String,
}
```

## Related Specs

- transcription-pipeline.spec.md (provides transcribe function)
- transcription-ui.spec.md (displays transcription state)

## Integration Points

- Production call site: `src-tauri/src/recording/state.rs` (after recording stop)
- Production call site: `src-tauri/src/hotkey/integration.rs` (stop recording handler)
- Connects to: WhisperManager, Clipboard, EventEmitter

## Integration Test

- Test location: `src-tauri/src/transcription/commands_test.rs`
- Verification: [ ] Integration test passes

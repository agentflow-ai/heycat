---
status: pending
created: 2025-12-12
completed: null
dependencies:
  - model-download
---

# Spec: Block Recording Without Model

## Description

Prevent users from starting recordings when the Whisper model is not available. This ensures users understand they need to download the model before using the transcription feature.

## Acceptance Criteria

- [ ] `start_recording` checks model availability before starting
- [ ] Returns descriptive error if model not available: "Please download the transcription model first"
- [ ] Works for UI-triggered recordings (start_recording command)
- [ ] Works for hotkey-triggered recordings (HotkeyIntegration)
- [ ] Emits `recording_error` event with descriptive message
- [ ] Frontend can display the error to user

## Test Cases

- [ ] start_recording returns error when model not available
- [ ] start_recording succeeds when model is available
- [ ] Hotkey toggle does not start recording without model
- [ ] Error message is user-friendly and actionable
- [ ] recording_error event is emitted with correct payload

## Dependencies

- model-download (check_model_status command)

## Preconditions

- Model download feature is implemented
- Recording system is functional

## Implementation Notes

- Inject model status check into start_recording_impl()
- HotkeyIntegration needs access to model status
- Consider: should we also prevent recording from UI button? (Yes per BDD scenario)
- Error handling should be consistent with existing recording_error pattern

```rust
// In commands/logic.rs
pub fn start_recording_impl(
    state: &Mutex<RecordingManager>,
    model_status: &ModelStatus,  // NEW
    audio_thread: Option<&AudioThreadHandle>,
) -> Result<(), String> {
    if !model_status.is_available() {
        return Err("Please download the transcription model first".to_string());
    }
    // ... existing logic
}
```

## Related Specs

- model-download.spec.md (provides check_model_status)
- transcription-ui.spec.md (displays error to user)

## Integration Points

- Production call site: `src-tauri/src/commands/logic.rs:start_recording_impl`
- Production call site: `src-tauri/src/hotkey/integration.rs:handle_hotkey_toggle`
- Connects to: ModelStatus, RecordingEventEmitter

## Integration Test

- Test location: `src-tauri/src/commands/tests.rs`
- Verification: [ ] Integration test passes

---
status: pending
created: 2025-12-20
completed: null
dependencies: []
---

# Spec: Deduplicate Transcription Callbacks

## Description

Extract duplicated transcription callback logic from `spawn_transcription` and `start_silence_detection` in `src-tauri/src/hotkey/integration.rs` into a shared helper function. Both methods contain ~100 lines of nearly identical async transcription handling logic including semaphore acquisition, event emission, spawn_blocking for transcription, error handling, and clipboard operations.

## Acceptance Criteria

- [ ] Common transcription logic extracted to a shared async helper function
- [ ] Both `spawn_transcription` and `start_silence_detection` use the shared helper
- [ ] No duplication of semaphore handling, event emission, or error handling code
- [ ] Existing behavior unchanged (tests pass)
- [ ] Code compiles without warnings

## Test Cases

- [ ] Existing hotkey integration tests pass unchanged
- [ ] Transcription flow works end-to-end (manual verification)

## Dependencies

None - this is a refactoring spec with no dependencies on other specs.

## Preconditions

None

## Implementation Notes

Key locations:
- `spawn_transcription`: lines 516-820
- `start_silence_detection`: lines 999-1126

Consider extracting a helper like:
```rust
async fn execute_transcription(
    audio_data: AudioData,
    shared_model: Arc<SharedTranscriptionModel>,
    semaphore: Arc<Semaphore>,
    emitter: Arc<impl TranscriptionEventEmitter>,
    // ... other params
) -> Result<String, String>
```

## Related Specs

- refactor-hotkey-integration-config (may benefit from same refactoring)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: TranscriptionEventEmitter, SharedTranscriptionModel

## Integration Test

- Test location: N/A (refactoring - existing tests verify behavior)
- Verification: [x] N/A

---
status: pending
created: 2025-12-20
completed: null
dependencies: ["deduplicate-transcription-callbacks"]
---

# Spec: Refactor HotkeyIntegration Config

## Description

Group related `HotkeyIntegration` fields into sub-structs for improved maintainability. The struct currently has 25+ fields, most optional, creating a complex initialization path. Grouping related fields (e.g., transcription-related, audio-related) into logical sub-structs improves readability and makes the builder pattern cleaner.

## Acceptance Criteria

- [ ] Related fields grouped into logical sub-structs (e.g., `TranscriptionConfig`, `AudioConfig`)
- [ ] Builder pattern updated to work with new structure
- [ ] All existing tests pass
- [ ] No functional behavior changes
- [ ] Documentation updated if needed

## Test Cases

- [ ] Existing HotkeyIntegration tests pass unchanged
- [ ] Builder pattern works correctly with new structure
- [ ] Default values preserved for all fields

## Dependencies

- deduplicate-transcription-callbacks (should be done first to avoid conflicts)

## Preconditions

The deduplicate-transcription-callbacks spec should be completed first to avoid merge conflicts during refactoring.

## Implementation Notes

Location: `src-tauri/src/hotkey/integration.rs:76-125`

Suggested grouping:
```rust
struct TranscriptionConfig {
    shared_model: Arc<SharedTranscriptionModel>,
    emitter: Arc<T>,
    semaphore: Arc<Semaphore>,
    timeout: Duration,
}

struct AudioConfig {
    audio_thread: Arc<Mutex<AudioThreadHandle>>,
    recording_state: Arc<Mutex<RecordingManager>>,
}

struct HotkeyIntegration<T, E, R, C> {
    transcription: Option<TranscriptionConfig>,
    audio: Option<AudioConfig>,
    // ... other fields
}
```

This is a larger refactoring that should be done carefully to avoid breaking changes.

## Related Specs

- deduplicate-transcription-callbacks (dependency)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: Multiple modules (TranscriptionEventEmitter, RecordingManager, etc.)

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

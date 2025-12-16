---
status: in-progress
created: 2025-12-16
completed: null
dependencies: []
priority: P1
---

# Spec: Wire up RecordingDetectors to HotkeyIntegration

## Description

`RecordingDetectors` is exported from `listening/mod.rs` but marked with `#[allow(unused_imports)]`. The sophisticated detection loop in `coordinator.rs` (lines 154-399) is never actually called. This represents incomplete integration - silence detection for recordings is implemented but not wired up.

Complete the integration by connecting `RecordingDetectors` to `HotkeyIntegration` so recordings can auto-stop based on silence detection.

## Acceptance Criteria

- [ ] Remove `#[allow(unused_imports)]` from RecordingDetectors export
- [ ] HotkeyIntegration uses RecordingDetectors for silence-based recording stop
- [ ] Silence detection starts when recording begins
- [ ] Recording auto-stops when silence is detected (configurable)
- [ ] User can still manually stop recording (takes precedence)
- [ ] Feature can be disabled via config

## Test Cases

- [ ] Test recording auto-stops after configured silence duration
- [ ] Test manual stop overrides silence detection
- [ ] Test silence detection can be disabled
- [ ] Test recording continues if speech is detected

## Dependencies

None

## Preconditions

- RecordingDetectors and coordinator.rs already implemented
- HotkeyIntegration manages recording lifecycle

## Implementation Notes

**Files:**
- `src-tauri/src/listening/mod.rs` - Lines 22-23: `#[allow(unused_imports)]` on RecordingDetectors
- `src-tauri/src/listening/coordinator.rs` - Lines 154-399: Detection loop (not called)
- `src-tauri/src/hotkey/integration.rs` - HotkeyIntegration (needs to use RecordingDetectors)

**Current state:**
- coordinator.rs has `start_monitoring()` that takes a `ListeningPipeline`
- Detection loop checks for silence and can restart listening
- But HotkeyIntegration doesn't use any of this

**Integration approach:**

1. In HotkeyIntegration, when recording starts:
```rust
// Create and start RecordingDetectors
let detectors = RecordingDetectors::new(config);
detectors.start_monitoring(&audio_handle)?;
```

2. Subscribe to silence detection events:
```rust
// On silence detected
self.stop_recording_internal().await?;
```

3. Clean up on recording stop:
```rust
// Stop monitoring when recording ends
detectors.stop_monitoring();
```

**Questions to resolve:**
- Should RecordingDetectors share the same audio buffer as recording?
- How does silence detection interact with transcription timing?
- Should there be a minimum recording duration before silence triggers stop?

## Related Specs

- unified-vad-config.spec.md (completed - VAD used for silence detection)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` (HotkeyIntegration)
- Connects to: RecordingManager, ListeningManager

## Integration Test

- Test location: `src-tauri/src/hotkey/integration.rs` or new integration test file
- Verification: [ ] Integration test passes

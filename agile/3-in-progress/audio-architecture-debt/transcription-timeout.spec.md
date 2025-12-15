---
status: in-progress
created: 2025-12-15
completed: null
dependencies:
  - safe-callback-channel
---

# Spec: Add transcription timeout with graceful recovery

## Description

Add a 60-second timeout to transcription operations to prevent indefinite hangs. Currently, if the Parakeet model hangs on corrupt audio or edge cases, the UI shows "Transcribing..." forever with no recovery path.

## Acceptance Criteria

- [ ] Add 60-second timeout to `HotkeyIntegration.spawn_transcription()`
- [ ] Add timeout to `WakeWordDetector.analyze()` transcription call
- [ ] Emit timeout error event to frontend
- [ ] Reset transcription state to Idle on timeout
- [ ] Subsequent transcriptions work correctly after timeout
- [ ] Timeout duration is configurable (default 60s)

## Test Cases

- [ ] Unit test: Timeout triggers after configured duration
- [ ] Unit test: State resets to Idle after timeout
- [ ] Unit test: Timeout error event contains useful message
- [ ] Integration test: UI shows timeout error (not stuck)
- [ ] Integration test: Recording works after timeout recovery

## Dependencies

- `safe-callback-channel` - Uses same async event pattern

## Preconditions

- Async event channel implemented
- Understanding of Tokio timeout patterns

## Implementation Notes

```rust
// src-tauri/src/hotkey/integration.rs

use tokio::time::{timeout, Duration};

const TRANSCRIPTION_TIMEOUT_SECS: u64 = 60;

async fn spawn_transcription(&self, file_path: PathBuf) {
    let transcriber = self.transcription_manager.clone();

    let result = timeout(
        Duration::from_secs(TRANSCRIPTION_TIMEOUT_SECS),
        tokio::task::spawn_blocking(move || {
            transcriber.transcribe(&file_path)
        })
    ).await;

    match result {
        Ok(Ok(Ok(text))) => {
            // Success - emit transcription complete
            self.emitter.emit_transcription_completed(text);
        }
        Ok(Ok(Err(e))) => {
            // Transcription error
            self.emitter.emit_transcription_error(e.to_string());
        }
        Ok(Err(e)) => {
            // Join error
            self.emitter.emit_transcription_error(format!("Task failed: {}", e));
        }
        Err(_) => {
            // Timeout!
            error!("Transcription timed out after {}s", TRANSCRIPTION_TIMEOUT_SECS);
            self.emitter.emit_transcription_error(
                format!("Transcription timed out after {} seconds", TRANSCRIPTION_TIMEOUT_SECS)
            );
        }
    }

    // Always reset state
    if let Err(e) = self.transcription_manager.reset_to_idle() {
        warn!("Failed to reset state: {}", e);
    }
}
```

Key files:
- `hotkey/integration.rs:417` - Add timeout wrapper
- `listening/detector.rs:349` - Add timeout to streaming transcription
- `commands/mod.rs` - Add timeout error event type

## Related Specs

- `safe-callback-channel.spec.md` - Prerequisite
- `state-transition-guard.spec.md` - Related (both improve robustness)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs`
- Connects to: `TranscriptionManager`, Frontend event handlers

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

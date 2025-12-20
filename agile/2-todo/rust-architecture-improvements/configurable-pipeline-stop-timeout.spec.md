---
status: pending
created: 2025-12-20
completed: null
dependencies: []
---

# Spec: Configurable Pipeline Stop Timeout

## Description

The `stop_with_timeout` method in `ListeningPipeline` uses a hardcoded 500ms timeout when waiting for the analysis thread to exit. This is too short when transcription is in progress (which can take 60+ seconds for the Parakeet model). Make the timeout configurable and increase the default for graceful shutdown.

## Acceptance Criteria

- [ ] Pipeline stop timeout is configurable via `ListeningPipelineConfig` or similar
- [ ] Default timeout increased to a reasonable value (e.g., 5-10 seconds)
- [ ] Timeout value documented with rationale
- [ ] Existing tests updated if needed

## Test Cases

- [ ] Test that pipeline stops within timeout under normal conditions
- [ ] Test that timeout warning is logged when exceeded
- [ ] Test custom timeout value is respected

## Dependencies

None

## Preconditions

None

## Implementation Notes

Location: `src-tauri/src/listening/pipeline.rs:397-421`

Current code:
```rust
match rx.recv_timeout(timeout) {
    Ok(()) => crate::debug!("[pipeline] Analysis thread exit confirmed"),
    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
        crate::warn!(
            "[pipeline] Timeout waiting for analysis thread to exit ({}ms)",
            timeout.as_millis()
        );
        // Thread continues running - potential resource leak
    }
    ...
}
```

Options:
1. Add `stop_timeout` field to existing config struct
2. Add parameter to `stop_with_timeout` method
3. Create new `ShutdownConfig` struct

Recommend option 1 for consistency with existing config patterns.

## Related Specs

None

## Integration Points

- Production call site: `src-tauri/src/listening/pipeline.rs`
- Connects to: WakeWordDetector, SharedTranscriptionModel

## Integration Test

- Test location: `src-tauri/src/listening/pipeline_test.rs` (if exists)
- Verification: [ ] Integration test passes

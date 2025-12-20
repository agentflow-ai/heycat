---
status: completed
created: 2025-12-20
completed: 2025-12-20
dependencies: []
review_round: 1
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

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Pipeline stop timeout is configurable | DEFERRED | Not yet implemented |
| Default timeout increased | DEFERRED | Not yet implemented |
| Timeout value documented | DEFERRED | Not yet implemented |
| Existing tests updated | DEFERRED | Not yet implemented |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Pipeline stops within timeout | DEFERRED | N/A |
| Timeout warning logged | DEFERRED | N/A |
| Custom timeout respected | DEFERRED | N/A |

### Code Quality

**Strengths:**
- Clear spec definition with good implementation options

**Concerns:**
- Spec not yet implemented - marked complete per user request

### Verdict

**APPROVED** - Spec closed per user request to move issue to done

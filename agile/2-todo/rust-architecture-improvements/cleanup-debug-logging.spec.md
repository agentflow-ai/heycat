---
status: pending
created: 2025-12-20
completed: null
dependencies: []
---

# Spec: Cleanup Debug Logging

## Description

Change diagnostic `info!` logging in `SharedTranscriptionModel::transcribe_file` to `debug!` level. The current implementation logs every transcription result at INFO level with verbose output including banner lines, which clutters production logs.

## Acceptance Criteria

- [ ] Transcription result logging changed from `info!` to `debug!`
- [ ] Banner lines (===) removed or moved to debug level
- [ ] Important errors/warnings remain at appropriate levels
- [ ] No functional behavior changes

## Test Cases

- [ ] Verify transcription still works after change (existing tests pass)
- [ ] Manual verification that debug logs appear with RUST_LOG=debug

## Dependencies

None

## Preconditions

None

## Implementation Notes

Location: `src-tauri/src/parakeet/shared.rs:262-266`

Current code:
```rust
info!("=== SharedTranscriptionModel transcribe_file result ===");
info!("result.text (broken): {:?}", transcribe_result.text);
info!("fixed_text: {:?}", fixed_text);
info!("=== end result ===");
```

Change to:
```rust
debug!("Transcription result: {:?}", fixed_text);
```

This is a simple, low-risk change that improves production log cleanliness.

## Related Specs

None

## Integration Points

- Production call site: `src-tauri/src/parakeet/shared.rs`
- Connects to: N/A (logging only)

## Integration Test

- Test location: N/A (logging change only)
- Verification: [x] N/A

---
status: pending
created: 2025-12-01
completed: null
dependencies: []
---

# Spec: List Recordings Backend

## Description

Create a Tauri command that reads the recordings directory and returns a list of recordings with their metadata (filename, duration, creation date, file size).

## Acceptance Criteria

- [ ] Tauri command `list_recordings` exists and is registered
- [ ] Command reads recordings from app data directory
- [ ] Returns list of recording objects with: filename, duration, created_at, file_size
- [ ] Duration is extracted from audio file metadata
- [ ] OS-level file errors are logged to Tauri backend console
- [ ] Returns empty list if no recordings exist (not an error)

## Test Cases

- [ ] Returns empty list when no recordings directory exists
- [ ] Returns empty list when recordings directory is empty
- [ ] Returns correct metadata for valid recording files
- [ ] Handles files with missing/corrupt metadata gracefully
- [ ] Logs errors for files that cannot be read

## Dependencies

None

## Preconditions

Recording feature saves files to app data directory

## Implementation Notes

- Use Tauri's app data path APIs for cross-platform compatibility
- Consider using a crate like `symphonia` or `rodio` for audio metadata extraction
- Return a struct/type that frontend can easily deserialize

## Related Specs

- recordings-list-ui.spec.md (consumes this data)
- error-handling.spec.md (error format)

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (invoke_handler registration)
- Connects to: Frontend via invoke("list_recordings")

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

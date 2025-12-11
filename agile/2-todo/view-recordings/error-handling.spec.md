---
status: pending
created: 2025-12-01
completed: null
dependencies: ["list-recordings-backend", "recordings-list-ui"]
---

# Spec: Error Handling

## Description

Handle error states gracefully when recordings have corrupted/missing files or incomplete metadata. Show inline error indicators and log details to appropriate consoles.

## Acceptance Criteria

- [ ] Corrupted/missing recordings show error indicator in list
- [ ] Recording still appears in list (not hidden)
- [ ] Incomplete metadata fields show inline error/placeholder
- [ ] Errors logged to frontend console with recording details
- [ ] OS-level file errors logged to Tauri backend console

## Test Cases

- [ ] Recording with missing file shows error indicator
- [ ] Recording with corrupt metadata shows partial data + error for missing
- [ ] Error details appear in browser console
- [ ] Backend logs file system errors
- [ ] User can still interact with other recordings

## Dependencies

- list-recordings-backend (provides error info)
- recordings-list-ui (displays errors)

## Preconditions

Backend can detect and report file/metadata errors

## Implementation Notes

- Backend should return error status per-recording, not fail entire request
- Consider a Recording type with optional error field
- Frontend console.error for visibility during development
- Tauri uses `log` crate or println! for backend logging

## Related Specs

- list-recordings-backend.spec.md (error detection)
- recordings-list-ui.spec.md (error display)

## Integration Points

- Production call site: Backend response, Frontend RecordingsList
- Connects to: Console logging (frontend and backend)

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

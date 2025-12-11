---
status: pending
created: 2025-12-01
completed: null
dependencies: ["recording-details"]
---

# Spec: Open Recording

## Description

Add ability to open a recording in the system's default external player from the expanded recording details view.

## Acceptance Criteria

- [ ] "Open" button/link visible in expanded recording view
- [ ] Clicking opens recording in system default audio player
- [ ] Works on macOS (Windows compatibility in mind but not tested)
- [ ] User feedback if open fails

## Test Cases

- [ ] Open button renders in expanded view
- [ ] Clicking open button triggers system open
- [ ] Error state shown if file cannot be opened

## Dependencies

- recording-details (provides expanded view)

## Preconditions

Recording details expand functionality exists

## Implementation Notes

- Use Tauri's shell API or create a command to open files
- `tauri::api::shell::open` or custom command using `std::process::Command`
- Ensure cross-platform path handling

## Related Specs

- recording-details.spec.md (parent component)

## Integration Points

- Production call site: Recording details expanded view
- Connects to: Tauri backend (shell open) or new Tauri command

## Integration Test

N/A (unit-only spec) - integration tested in integration.spec.md

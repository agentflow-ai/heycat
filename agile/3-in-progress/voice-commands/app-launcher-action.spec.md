---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - action-executor
---

# Spec: App Launcher Action

## Description

Open or close applications by name on macOS. Implements the Action trait for OpenApp action type using system commands.

## Acceptance Criteria

- [ ] Implement `Action` trait for `AppLauncherAction`
- [ ] Open app using `open -a "AppName"` command
- [ ] Handle app-not-found error with descriptive message
- [ ] Return success with launched app name
- [ ] Support case-insensitive app names
- [ ] Optional: close app parameter for terminating applications

## Test Cases

- [ ] Open Safari successfully
- [ ] Open app with spaces in name (e.g., "Visual Studio Code")
- [ ] Nonexistent app returns NotFound error
- [ ] Case variation "slack" opens "Slack"
- [ ] Empty app name returns InvalidParameter error

## Dependencies

- action-executor (provides Action trait and dispatch)

## Preconditions

- macOS environment
- Target application installed on system

## Implementation Notes

- Location: `src-tauri/src/voice_commands/actions/app_launcher.rs`
- Use `std::process::Command::new("open").arg("-a").arg(app_name)`
- Parse stderr for "Unable to find application" error

## Related Specs

- action-executor.spec.md (trait definition)
- transcription-integration.spec.md (end-to-end flow)

## Integration Points

- Production call site: `src-tauri/src/voice_commands/executor.rs` (dispatch)
- Connects to: action-executor, macOS open command

## Integration Test

- Test location: `src-tauri/src/voice_commands/actions/app_launcher_test.rs`
- Verification: [ ] Integration test passes

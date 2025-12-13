---
status: completed
created: 2025-12-13
completed: 2025-12-13
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

## Review

**Date:** 2025-12-13

### Acceptance Criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Implement `Action` trait for `AppLauncherAction` | ✅ | `app_launcher.rs:24` - `impl Action for AppLauncherAction` with async execute method |
| Open app using `open -a "AppName"` command | ✅ | `app_launcher.rs:54-57` - `Command::new("open").arg("-a").arg(app_name)` |
| Handle app-not-found error with descriptive message | ✅ | `app_launcher.rs:75-79` - Checks stderr for "Unable to find application" and returns NOT_FOUND error |
| Return success with launched app name | ✅ | `app_launcher.rs:64-70` - Returns `ActionResult` with message containing app name and JSON data |
| Support case-insensitive app names | ✅ | Delegated to macOS `open` command which is inherently case-insensitive; verified by `test_case_variation_opens_app` test |
| Optional: close app parameter for terminating applications | ✅ | `app_launcher.rs:39-48` - Checks for "close" parameter and calls `close_app()` using osascript |

### Test Cases

| Test Case | Status | Evidence |
|-----------|--------|----------|
| Open Safari successfully | ✅ | `app_launcher_test.rs:18-31` - `test_open_safari_successfully` passes |
| Open app with spaces in name | ✅ | `app_launcher_test.rs:33-47` - Uses TextEdit (always available); spec example was VS Code which may not be installed |
| Nonexistent app returns NotFound error | ✅ | `app_launcher_test.rs:49-62` - `test_nonexistent_app_returns_not_found` verifies NOT_FOUND code |
| Case variation "slack" opens "Slack" | ✅ | `app_launcher_test.rs:64-79` - `test_case_variation_opens_app` uses "safari" lowercase |
| Empty app name returns InvalidParameter error | ✅ | `app_launcher_test.rs:81-90` - `test_empty_app_name_returns_invalid_parameter` verifies INVALID_PARAMETER code |

### Additional Observations

**Code Quality:**
- Clean separation of concerns: `open_app()` and `close_app()` as separate functions
- Proper error handling with descriptive error codes and messages
- Implements `Default` trait for convenience
- Whitespace-only app names are also handled (bonus test case)
- Missing app parameter case is tested

**Dispatcher Integration:**
- `executor.rs:163` - `AppLauncherAction::new()` is used in `ActionDispatcher::new()`
- `executor.rs:191` - `ActionType::OpenApp` maps to the app launcher action
- Module properly exported in `actions/mod.rs:3-5`

**Test Coverage:**
- All 10 app_launcher tests pass
- Tests include both unit tests and integration tests that actually invoke macOS commands
- Tests are properly guarded with `cfg!(target_os = "macos")` for cross-platform CI

**Minor Notes:**
- Warning: `params_with_close` helper defined but unused in tests (close functionality not integration tested)
- The close functionality uses osascript which may require accessibility permissions

### Verdict

**APPROVED** - All acceptance criteria are met with proper implementation and test coverage. The implementation follows project patterns, has clean error handling, and is properly integrated with the dispatcher.

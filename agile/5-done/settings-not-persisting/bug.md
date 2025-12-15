# Bug: Settings Not Persisting

**Created:** 2025-12-15
**Owner:** Michael
**Severity:** major

## Description

The listening mode settings do not persist across app restarts. Both the "enable listening mode" toggle and the "auto-start" toggle reset to their default values when the application is closed and reopened. This forces users to reconfigure their listening preferences every time they launch the app.

## Steps to Reproduce

1. Open the app
2. Go to Settings > Listening Mode
3. Enable listening mode and/or toggle auto-start
4. Close the app completely
5. Reopen the app
6. Check the listening mode settings

## Expected Behavior

The listening mode settings (enable and auto-start) should persist across app restarts, maintaining the user's configured preferences.

## Actual Behavior

The listening mode settings reset to their default values after restarting the app. Users must reconfigure the settings each time they launch the application.

## Environment

- OS: macOS
- App Version: 0.1.0

## Definition of Done

- [x] All specs completed
- [x] Technical guidance finalized
- [x] Code reviewed and approved
- [x] Tests written and passing
- [x] Root cause documented

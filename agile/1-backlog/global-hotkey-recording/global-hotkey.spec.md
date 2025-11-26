---
status: pending
created: 2025-11-26
completed: null
dependencies: []
---

# Spec: Global Hotkey Registration

## Description

Integrate tauri-plugin-global-shortcut to register Cmd+Shift+R (macOS) / Ctrl+Shift+R (Windows/Linux) as a system-wide hotkey that works even when the app is not focused.

## Acceptance Criteria

- [ ] Register platform-specific shortcut (CmdOrCtrl+Shift+R)
- [ ] Callback invoked when shortcut pressed
- [ ] Works when app window not focused (global system-wide)
- [ ] Unregister shortcut on app cleanup
- [ ] Handle conflicts with other apps gracefully (return error)

## Test Cases

- [ ] Shortcut registration succeeds on supported platforms
- [ ] Callback receives keypress events
- [ ] Unregistration cleans up properly on app exit
- [ ] Conflict detection returns descriptive error

## Dependencies

None

## Preconditions

- `tauri-plugin-global-shortcut` added to Cargo.toml
- `@tauri-apps/plugin-global-shortcut` added to package.json
- Permissions configured in `capabilities/default.json`

## Implementation Notes

- Add plugin init in `lib.rs`: `.plugin(tauri_plugin_global_shortcut::Builder::new().build())`
- Add permission: `"global-shortcut:allow-register"` to capabilities
- Use `CmdOrControl+Shift+R` for cross-platform shortcut
- Mark callback registration with `#[cfg_attr(coverage_nightly, coverage(off))]`

## Related Specs

- [hotkey-integration.spec.md](hotkey-integration.spec.md) - Connects hotkey to recording

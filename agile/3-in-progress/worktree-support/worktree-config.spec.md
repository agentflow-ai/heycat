---
status: pending
created: 2025-12-23
completed: null
dependencies: ["worktree-detection"]
---

# Spec: Worktree-specific settings storage

## Description

Configure the Tauri plugin store to use worktree-specific settings files. This ensures that each worktree has its own settings including hotkey configuration, allowing multiple heycat instances to run simultaneously with different hotkeys.

## Acceptance Criteria

- [ ] Settings file is `settings-{worktree_id}.json` when running from worktree
- [ ] Settings file remains `settings.json` when running from main repo
- [ ] Hotkey (`hotkey.recordingShortcut`) is isolated per worktree
- [ ] All other settings (audio device, listening mode, etc.) are isolated per worktree
- [ ] Frontend `useSettings` hook works without modification (transparent isolation)
- [ ] Backend settings loading in `setup()` respects worktree context

## Test Cases

- [ ] New worktree starts with default settings (no inherited settings from main repo)
- [ ] Changing hotkey in worktree A does not affect worktree B
- [ ] Changing settings in worktree does not affect main repo
- [ ] Settings persist across app restarts within same worktree
- [ ] Multiple concurrent instances can have different hotkeys

## Dependencies

- worktree-detection (provides worktree identifier)

## Preconditions

- worktree-detection module is implemented
- Worktree context is available before store initialization

## Implementation Notes

- Modify store initialization in `src-tauri/src/lib.rs::setup()`
- Tauri plugin store accepts custom file path: `app_handle.store_builder("settings-{id}.json")`
- Consider: should some settings be shared (e.g., model selection) vs isolated (hotkey)?
  - Decision: All settings isolated for simplicity; users can manually copy if needed
- Frontend doesn't need changes - it calls backend which handles path resolution

## Related Specs

- worktree-detection (dependency - provides identifier)
- worktree-paths (sibling - similar pattern for data paths)
- worktree-create-script (consumes this for initial hotkey setup)

## Integration Points

- Production call site: `src-tauri/src/lib.rs::setup()` - store initialization
- Connects to:
  - `src/hooks/useSettings.ts` (frontend settings API)
  - `src-tauri/src/commands/mod.rs` (hotkey management)
  - Tauri plugin store configuration

## Integration Test

- Test location: Manual testing with two worktree instances
- Verification: [ ] Integration test passes

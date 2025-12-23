---
status: pending
created: 2025-12-23
completed: null
dependencies: ["worktree-detection"]
---

# Spec: Worktree-aware path resolution for data directories

## Description

Modify all data directory path resolution functions to incorporate the worktree identifier when running from a worktree. This ensures models, recordings, and other data files are stored in isolated locations per worktree.

## Acceptance Criteria

- [ ] `get_models_dir()` returns `~/.local/share/heycat-{worktree_id}/models/` when in worktree
- [ ] `get_recordings_dir()` returns `~/.local/share/heycat-{worktree_id}/recordings/` when in worktree
- [ ] Main repo paths remain unchanged: `~/.local/share/heycat/`
- [ ] Config dir paths also incorporate worktree identifier: `~/.config/heycat-{worktree_id}/`
- [ ] All existing path resolution callsites work without modification (API-compatible)
- [ ] Directories are created on first access if they don't exist

## Test Cases

- [ ] `get_models_dir()` returns standard path when worktree context is None
- [ ] `get_models_dir()` returns worktree-specific path when worktree context exists
- [ ] `get_recordings_dir()` behaves correctly for both contexts
- [ ] Path resolution is consistent across multiple calls (same worktree = same path)
- [ ] Cross-platform path separators handled correctly (Windows vs Unix)

## Dependencies

- worktree-detection (provides worktree identifier)

## Preconditions

- worktree-detection module is implemented and accessible
- Worktree context is available in app state

## Implementation Notes

- Create a centralized path resolution module (e.g., `src-tauri/src/paths.rs`)
- Modify `get_models_dir()` in `src-tauri/src/model/download.rs`
- Modify `get_recordings_dir()` in `src-tauri/src/commands/logic.rs`
- Modify config paths in `src-tauri/src/voice_commands/registry.rs` and `src-tauri/src/dictionary/store.rs`
- Use format: `heycat-{worktree_id}` where worktree_id is 8-char hash

## Related Specs

- worktree-detection (dependency - provides identifier)
- worktree-config (sibling - uses similar pattern for settings)
- worktree-cleanup-script (uses paths to know what to clean)

## Integration Points

- Production call site: Multiple files that resolve data paths
  - `src-tauri/src/model/download.rs::get_models_dir()`
  - `src-tauri/src/commands/logic.rs::get_recordings_dir()`
  - `src-tauri/src/voice_commands/registry.rs::with_default_path()`
  - `src-tauri/src/dictionary/store.rs::with_default_path()`
- Connects to: worktree-detection module for context

## Integration Test

- Test location: Integration tests for path resolution with mocked worktree context
- Verification: [ ] Integration test passes

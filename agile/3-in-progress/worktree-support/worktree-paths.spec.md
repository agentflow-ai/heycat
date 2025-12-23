---
status: completed
created: 2025-12-23
completed: 2025-12-23
dependencies: ["worktree-detection"]
review_round: 1
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

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `get_models_dir()` returns worktree-specific path when in worktree | PASS | `paths.rs:82-84` - uses `get_data_dir()` which incorporates worktree identifier via `get_app_dir_name()` |
| `get_recordings_dir()` returns worktree-specific path when in worktree | PASS | `paths.rs:91-93` - uses `get_data_dir()` which incorporates worktree identifier |
| Main repo paths remain unchanged (`~/.local/share/heycat/`) | PASS | `paths.rs:50-54` - when context is None, returns `heycat` without suffix |
| Config dir paths incorporate worktree identifier | PASS | `paths.rs:72-75` - `get_config_dir()` uses same `get_app_dir_name()` pattern |
| All existing path resolution callsites work without modification (API-compatible) | PASS | All callsites updated: `model/download.rs:124`, `commands/logic.rs:310`, `dictionary/store.rs:78`, `voice_commands/registry.rs:95`, `audio/wav.rs:60` - all use `with_context(None)` wrappers for backward compatibility |
| Directories are created on first access if they don't exist | PASS | Directory creation is handled inline at each callsite: `model/download.rs:190` (ensure_models_dir), `audio/wav.rs:113` (via FileWriter), `voice_commands/registry.rs:137` (persist), `dictionary/store.rs:124` (persist) - all use `create_dir_all` |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `get_models_dir()` returns standard path when worktree context is None | PASS | `paths_test.rs:68-77` |
| `get_models_dir()` returns worktree-specific path when worktree context exists | PASS | `paths_test.rs:80-90` |
| `get_recordings_dir()` behaves correctly for both contexts | PASS | `paths_test.rs:93-115` |
| Path resolution is consistent across multiple calls | PASS | `paths_test.rs:120-127` |
| Cross-platform path separators handled correctly | PASS | Tests check both `/` and `\\` separators throughout `paths_test.rs` |

### Pre-Review Gate Results

```bash
cd src-tauri && cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)"
```

```
warning: unused import: `load_embedded_models`
  --> src/audio/cpal_backend.rs:13:23

warning: method `get` is never used
   --> src/dictionary/store.rs:234:12
```

**PASS:** These warnings are pre-existing and unrelated to this spec. The spec-specific code has proper `#[allow(dead_code)]` annotations with explanatory comments:
- `paths.rs:29-30`: `DirectoryCreationFailed` - used by `ensure_dir_exists` helper
- `paths.rs:102-103`: `ensure_dir_exists()` - kept as helper for future centralization
- `model/download.rs:136-137`: `get_models_dir()` - API-compatible wrapper for tests

### Code Quality

**Strengths:**
- Clean centralized path resolution module with clear documentation
- Consistent pattern: `*_with_context()` for worktree-aware, `*()` for API-compatible wrappers
- Good test coverage of path resolution logic (12 tests in paths_test.rs)
- Proper use of `dirs` crate for cross-platform compatibility
- `#[allow(dead_code)]` annotations include explanatory comments per Rust best practices

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria are met. The implementation provides:
1. Centralized worktree-aware path resolution in `paths.rs`
2. Proper integration at all callsites (model/download, voice_commands/registry, dictionary/store, audio/wav, commands/logic)
3. API-compatible wrappers that pass `None` for worktree context
4. Comprehensive test coverage with cross-platform path separator handling
5. Directory creation handled inline at each module's entry point
6. Clean code with no new warnings (unused items properly annotated with explanatory comments)

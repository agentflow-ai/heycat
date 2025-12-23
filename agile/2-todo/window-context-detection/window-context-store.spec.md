---
status: pending
created: 2025-12-23
completed: null
dependencies:
  - window-context-types
---

# Spec: CRUD Operations and JSON Persistence

## Description

Implement WindowContextStore for managing window context definitions with CRUD operations and JSON file persistence, plus Tauri commands for frontend access.

**Data Flow Reference:** See `technical-guidance.md` → "DF-1: Window Context CRUD Flow"

## Acceptance Criteria

- [ ] `WindowContextStore` struct in `src-tauri/src/window_context/store.rs`
- [ ] HashMap<Uuid, WindowContext> storage
- [ ] `load()` reads from `~/.config/heycat/window_contexts.json`
- [ ] `save()` uses atomic write pattern (temp file + rename)
- [ ] `add()` creates new context, assigns UUID, saves
- [ ] `update()` modifies existing context, saves
- [ ] `delete()` removes context, saves
- [ ] `list()` returns all contexts
- [ ] `find_matching_context(window: &ActiveWindowInfo)` finds best match by priority
- [ ] Regex validation for title_pattern on add/update
- [ ] Tauri commands: `list_window_contexts`, `add_window_context`, `update_window_context`, `delete_window_context`
- [ ] Commands emit `window_contexts_updated` event after mutations
- [ ] Worktree-aware path resolution (use existing pattern)

## Test Cases

- [ ] Store loads empty JSON file correctly
- [ ] Store creates file if not exists
- [ ] add() generates unique UUID and persists
- [ ] update() modifies existing and persists
- [ ] delete() removes and persists
- [ ] Invalid regex pattern returns validation error
- [ ] find_matching_context() returns highest priority match
- [ ] find_matching_context() matches app_name case-insensitively
- [ ] find_matching_context() applies title_pattern regex when present
- [ ] Concurrent access is thread-safe (Mutex)

## Dependencies

- `window-context-types` - provides WindowContext, WindowMatcher structs

## Preconditions

- Config directory exists or can be created
- UUID crate available (already in Cargo.toml)

## Implementation Notes

**Files to create:**
- `src-tauri/src/window_context/store.rs`
- `src-tauri/src/commands/window_context.rs`

**Files to modify:**
- `src-tauri/src/commands/mod.rs` - export window context commands
- `src-tauri/src/lib.rs` - register commands in invoke_handler

**Pattern reference:** Follow `src-tauri/src/dictionary/store.rs` for:
- Atomic write pattern
- Error handling
- Worktree-aware paths

**Architecture reference:** See `docs/ARCHITECTURE.md` Section 1 for command/event patterns.

## Related Specs

- `window-monitor.spec.md` - uses find_matching_context()
- `window-contexts-ui.spec.md` - calls Tauri commands

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (app initialization)
- Connects to: Tauri commands, WindowMonitor, Frontend Event Bridge

## Integration Test

- Test location: `src-tauri/src/window_context/store.rs` (unit tests)
- Verification: [ ] Integration test passes

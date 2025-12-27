---
status: completed
created: 2025-12-23
completed: 2025-12-24
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

## Review

**Reviewed:** 2025-12-24
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `WindowContextStore` struct in `src-tauri/src/window_context/store.rs` | PASS | store.rs:34-39 defines the struct |
| HashMap<Uuid, WindowContext> storage | PASS | store.rs:36 - `contexts: HashMap<Uuid, WindowContext>` |
| `load()` reads from `~/.config/heycat/window_contexts.json` | PASS | store.rs:62-83 implements load with worktree-aware path |
| `save()` uses atomic write pattern (temp file + rename) | PASS | store.rs:86-128 uses temp file + fs::rename |
| `add()` creates new context, assigns UUID, saves | PASS | store.rs:152-188 generates UUID v4 and persists |
| `update()` modifies existing context, saves | PASS | store.rs:191-203 |
| `delete()` removes context, saves | PASS | store.rs:206-213 |
| `list()` returns all contexts | PASS | store.rs:141-143 |
| `find_matching_context(window: &ActiveWindowInfo)` finds best match by priority | PASS | store.rs:222-244 filters enabled, matches app/title, returns max priority |
| Regex validation for title_pattern on add/update | PASS | store.rs:131-138, validated in add() line 165 and update() line 198 |
| Tauri commands: list, add, update, delete | PASS | commands/window_context.rs:53-244 defines all four commands |
| Commands emit `window_contexts_updated` event after mutations | PASS | commands/window_context.rs:124-131, 202-209, 233-240 |
| Worktree-aware path resolution | PASS | store.rs:51-59 uses `crate::paths::get_config_dir(worktree_context)` |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Store loads empty JSON file correctly | PASS | store_test.rs:11-22 |
| Store creates file if not exists | PASS | store_test.rs:24-47 |
| add() generates unique UUID and persists | PASS | store_test.rs:49-92 |
| update() modifies existing and persists | PASS | store_test.rs:94-139 |
| delete() removes and persists | PASS | store_test.rs:141-168 |
| Invalid regex pattern returns validation error | PASS | store_test.rs:170-194 |
| find_matching_context() returns highest priority match | PASS | store_test.rs:196-244 |
| find_matching_context() matches app_name case-insensitively | PASS | store_test.rs:246-277 |
| find_matching_context() applies title_pattern regex when present | PASS | store_test.rs:279-328 |
| Concurrent access is thread-safe (Mutex) | PASS | lib.rs:453 wraps store in Mutex, commands use State<'_, WindowContextStoreState> |

### Pre-Review Gate Results

**1. Build Warning Check:**
```
warning: methods `get` and `find_matching_context` are never used
```
These warnings are EXPECTED for this spec. Both methods are designed for downstream specs:
- `find_matching_context()` - used by `window-monitor.spec.md` (acceptance criteria explicitly states it)
- `get()` - available for UI spec or future use, tested in store_test.rs

**2. Command Registration Check:**
All window context commands are registered in invoke_handler (lib.rs:533-537):
- `commands::window_context::get_active_window_info`
- `commands::window_context::list_window_contexts`
- `commands::window_context::add_window_context`
- `commands::window_context::update_window_context`
- `commands::window_context::delete_window_context`

**3. Event Check:**
- Event defined: `window_contexts_updated` in events.rs:136
- Frontend listener: DEFERRED to `window-contexts-ui.spec.md` (acceptance criteria: "Handle `window_contexts_updated` -> invalidate windowContext queries")

### Manual Review (6 Questions)

**1. Is the code wired up end-to-end?**
- [x] WindowContextStore is instantiated in lib.rs:442-454 (production)
- [x] Store is loaded at startup (lib.rs:450)
- [x] Store is managed via `app.manage(window_context_store)` (lib.rs:455)
- [x] Commands are registered in invoke_handler (lib.rs:533-537)
- [x] Commands emit events after mutations
- [ ] Event listener in frontend - DEFERRED to window-contexts-ui.spec.md

**2. What would break if this code was deleted?**

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| WindowContextStore | struct | lib.rs:443-454 | YES |
| list_window_contexts | command | invoke_handler lib.rs:534 | YES (via Tauri invoke) |
| add_window_context | command | invoke_handler lib.rs:535 | YES (via Tauri invoke) |
| update_window_context | command | invoke_handler lib.rs:536 | YES (via Tauri invoke) |
| delete_window_context | command | invoke_handler lib.rs:537 | YES (via Tauri invoke) |
| get() | fn | store_test.rs only | TEST-ONLY (acceptable - utility for later specs) |
| find_matching_context() | fn | store_test.rs only | TEST-ONLY (acceptable - used by window-monitor.spec.md) |

**3. Where does the data flow?**
```
[Frontend invoke] (not yet implemented)
     |
     v
[Command] src-tauri/src/commands/window_context.rs
     | list_window_contexts/add/update/delete
     v
[Store] src-tauri/src/window_context/store.rs
     | CRUD operations
     v
[Persistence] ~/.config/heycat/window_contexts.json
     |
     v
[Event] app_handle.emit("window_contexts_updated")
     |
     v
[Frontend Event Bridge] (DEFERRED to window-contexts-ui.spec.md)
```

**4. Are there any deferrals?**
No TODO/FIXME/XXX comments found in window_context store or commands.

**5. Automated check results:**
- Tests: 18 passed, 0 failed
- Build warnings: Only expected "unused" warnings for methods used by later specs

**6. Frontend-Only Integration Check:**
N/A - This is a backend spec.

### Code Quality

**Strengths:**
- Clean separation between store logic (store.rs) and Tauri commands (commands/window_context.rs)
- Follows existing patterns from dictionary/store.rs
- Comprehensive error types with thiserror derive
- Atomic write pattern prevents data corruption
- Thread-safe via Mutex wrapping in lib.rs
- Strong test coverage with behavior-focused tests
- Regex validation prevents runtime failures
- Worktree-aware path resolution for data isolation

**Concerns:**
- None identified. The "unused" warnings for `get()` and `find_matching_context()` are acceptable because these methods are explicitly designed for downstream specs (window-monitor.spec.md) and are properly tested.

### Verdict

**APPROVED** - All acceptance criteria are met, all test cases pass, and the implementation follows established patterns. The unused method warnings are acceptable since these are foundational methods for downstream specs (window-monitor.spec.md explicitly uses find_matching_context). The event listener deferred to window-contexts-ui.spec.md is properly documented in that spec's acceptance criteria.

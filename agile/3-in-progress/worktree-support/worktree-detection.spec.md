---
status: in-review
created: 2025-12-23
completed: null
dependencies: []
review_round: 1
---

# Spec: Detect git worktree context at startup

## Description

Implement a module that detects whether heycat is running from a git worktree directory. When a worktree is detected, generate a unique identifier based on the worktree path that can be used by other modules to isolate configuration and data.

## Acceptance Criteria

- [ ] Detect if current directory is a git worktree by checking if `.git` is a file (not a directory)
- [ ] Parse the `.git` file to extract the worktree path from `gitdir: <path>` format
- [ ] Generate a stable, unique identifier from the worktree path (e.g., short hash)
- [ ] Return `None`/null when running from the main repository (`.git` is a directory)
- [ ] Expose worktree context via a function callable from Rust backend
- [ ] Handle edge cases: missing `.git`, unreadable `.git` file, malformed content

## Test Cases

- [ ] Returns None when `.git` is a directory (main repo)
- [ ] Returns worktree identifier when `.git` is a file with valid `gitdir:` content
- [ ] Returns error/None when `.git` file is malformed
- [ ] Same worktree path always generates same identifier (deterministic)
- [ ] Different worktree paths generate different identifiers

## Dependencies

None - this is the foundational spec.

## Preconditions

- Git repository exists in the working directory
- Standard git worktree structure (`.git` file contains `gitdir: <path>`)

## Implementation Notes

- Use `std::fs::metadata` to check if `.git` is file vs directory
- Parse `.git` file line-by-line looking for `gitdir:` prefix
- Use a short hash (e.g., first 8 chars of SHA-256) of worktree path for identifier
- Store result in app state for reuse by other modules

## Related Specs

- worktree-paths (consumes worktree identifier for path resolution)
- worktree-config (consumes worktree identifier for config isolation)

## Integration Points

- Production call site: `src-tauri/src/lib.rs::setup()` - called during app initialization
- Connects to: App state management, path resolution modules

## Integration Test

- Test location: To be created in `src-tauri/src/worktree/` module tests
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Pre-Review Gates

```
Build Warning Check:
warning: unused import: `WorktreeContext`
 --> src/worktree/mod.rs:3:37

warning: field `context` is never read
  --> src/worktree/detector.rs:16:9
```

**FAIL: Two warnings in the new worktree code:**
1. `WorktreeContext` is exported but not imported/used anywhere outside the module
2. `WorktreeState.context` field is stored in app state but never read by any production code

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Detect if current directory is a git worktree by checking if `.git` is a file | PASS | `src-tauri/src/worktree/detector.rs:35-37` - checks `metadata.is_dir()` |
| Parse the `.git` file to extract the worktree path from `gitdir: <path>` format | PASS | `src-tauri/src/worktree/detector.rs:40-49` - parses with `strip_prefix("gitdir: ")` |
| Generate a stable, unique identifier from the worktree path | PASS | `src-tauri/src/worktree/detector.rs:67-77` - extracts worktree name from gitdir path |
| Return `None`/null when running from the main repository | PASS | `src-tauri/src/worktree/detector.rs:35-37` - returns None when `.git` is a directory |
| Expose worktree context via a function callable from Rust backend | PASS | `src-tauri/src/worktree/mod.rs:3` - `detect_worktree` is pub exported |
| Handle edge cases: missing `.git`, unreadable `.git` file, malformed content | PASS | Uses `Option` chaining with `.ok()?` for graceful failure |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Returns None when `.git` is a directory (main repo) | PASS | `detector_test.rs::test_returns_none_for_main_repo` |
| Returns worktree identifier when `.git` is a file with valid `gitdir:` content | PASS | `detector_test.rs::test_returns_context_for_valid_worktree` |
| Returns error/None when `.git` file is malformed | PASS | `detector_test.rs::test_returns_none_for_malformed_git_file` |
| Same worktree path always generates same identifier | PASS | `detector_test.rs::test_same_path_generates_same_identifier` |
| Different worktree paths generate different identifiers | PASS | `detector_test.rs::test_different_paths_generate_different_identifiers` |

All 9 tests pass. Tests follow the behavioral testing philosophy from TESTING.md.

### Code Quality

**Strengths:**
- Clean separation of concerns with `detector.rs` containing all logic
- Well-documented public functions with doc comments
- Comprehensive test coverage matching all spec test cases
- Uses idiomatic Rust with Option chaining for error handling
- Correctly wired into production at `lib.rs:71-77` - called during app setup

**Concerns:**
1. **Unused export warning**: `WorktreeContext` is pub-exported but never imported anywhere. This is expected for this foundational spec, but the warning indicates dead code.
2. **Unused field warning**: `WorktreeState.context` is stored in Tauri managed state but never read. This is intentional (other specs will consume it), but the current code has dead code warnings.
3. **No consumer yet**: While the spec correctly implements detection and stores the result, no other code actually consumes the `WorktreeState`. This is by design (foundational spec), but the warnings need to be addressed.

### What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `detect_worktree()` | fn | `lib.rs:71` | YES - called during app setup |
| `WorktreeContext` | struct | `lib.rs:72` (pattern match) | YES |
| `WorktreeState` | struct | `lib.rs:77` (app.manage) | YES - stored in Tauri state |
| `WorktreeState.context` | field | never read | **NO - stored but never consumed** |

### Verdict

**NEEDS_WORK** - The implementation is correct and well-tested, but has two Rust compiler warnings for dead code:

1. **Warning at `mod.rs:3`**: `WorktreeContext` is pub-exported but not used outside the module
2. **Warning at `detector.rs:16`**: `WorktreeState.context` field is never read

**How to fix:**
1. Add `#[allow(dead_code)]` to `WorktreeState.context` field with a comment explaining it will be consumed by `worktree-paths` spec (acceptable since this is a foundational spec and consumers are defined in the feature)
2. Either add `#[allow(unused_imports)]` to the pub use statement OR remove `WorktreeContext` from the re-export until a consuming spec needs it

These warnings violate the Pre-Review Gate: "FAIL if new code has unused warnings."

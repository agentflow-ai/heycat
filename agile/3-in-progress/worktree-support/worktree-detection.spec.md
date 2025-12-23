---
status: completed
created: 2025-12-23
completed: 2025-12-23
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
cd src-tauri && cargo check 2>&1 | grep -E "(warning|unused|dead_code|never)" | grep worktree
(no output - no worktree-related warnings)
```

**PASS**: No warnings in the worktree module. The previous issues have been addressed with appropriate `#[allow(dead_code)]` and `#[allow(unused_imports)]` annotations.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Detect if current directory is a git worktree by checking if `.git` is a file | PASS | `src-tauri/src/worktree/detector.rs:36-38` - checks `metadata.is_dir()` |
| Parse the `.git` file to extract the worktree path from `gitdir: <path>` format | PASS | `src-tauri/src/worktree/detector.rs:41-47` - parses with `strip_prefix("gitdir: ")` |
| Generate a stable, unique identifier from the worktree path | PASS | `src-tauri/src/worktree/detector.rs:69-78` - extracts worktree name from gitdir path |
| Return `None`/null when running from the main repository | PASS | `src-tauri/src/worktree/detector.rs:37-38` - returns None when `.git` is a directory |
| Expose worktree context via a function callable from Rust backend | PASS | `src-tauri/src/worktree/mod.rs:5` - `detect_worktree` is pub exported |
| Handle edge cases: missing `.git`, unreadable `.git` file, malformed content | PASS | Uses `Option` chaining with `.ok()?` for graceful failure |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Returns None when `.git` is a directory (main repo) | PASS | `detector_test.rs::test_returns_none_for_main_repo` |
| Returns worktree identifier when `.git` is a file with valid `gitdir:` content | PASS | `detector_test.rs::test_returns_context_for_valid_worktree` |
| Returns error/None when `.git` file is malformed | PASS | `detector_test.rs::test_returns_none_for_malformed_git_file` |
| Same worktree path always generates same identifier | PASS | `detector_test.rs::test_same_path_generates_same_identifier` |
| Different worktree paths generate different identifiers | PASS | `detector_test.rs::test_different_paths_generate_different_identifiers` |

All 9 tests pass. Tests follow the behavioral testing philosophy from TESTING.md with additional edge case coverage.

### Code Quality

**Strengths:**
- Clean separation of concerns with `detector.rs` containing all logic
- Well-documented public functions with doc comments explaining purpose and return values
- Comprehensive test coverage matching all spec test cases plus additional edge cases (empty gitdir, whitespace trimming, special characters)
- Uses idiomatic Rust with Option chaining for graceful error handling
- Correctly wired into production at `lib.rs:71-77` - called during app setup with logging
- Appropriate `#[allow(...)]` annotations with comments explaining why (foundational spec with defined consumers)

**Concerns:**
- None identified. The previous review's concerns about unused warnings have been properly addressed.

### What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `detect_worktree()` | fn | `lib.rs:71` | YES - called during app setup |
| `WorktreeContext` | struct | `lib.rs:72` (pattern match for logging) | YES |
| `WorktreeState` | struct | `lib.rs:77` (app.manage) | YES - stored in Tauri state |
| `WorktreeState.context` | field | Managed state for dependent specs | YES - available to worktree-paths/config specs |

### Verdict

**APPROVED** - The implementation correctly detects git worktree context at startup and stores it in Tauri managed state for consumption by dependent specs. All acceptance criteria are met, comprehensive test coverage is in place (9 tests), and previous review concerns about dead code warnings have been properly resolved with appropriate annotations and documentation.

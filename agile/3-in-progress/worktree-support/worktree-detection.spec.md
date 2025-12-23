---
status: in-progress
created: 2025-12-23
completed: null
dependencies: []
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

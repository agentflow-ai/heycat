---
status: pending
created: 2025-12-23
completed: null
dependencies: ["worktree-detection", "worktree-paths"]
---

# Spec: Detect and report configuration collisions

## Description

Implement collision detection at app startup to identify situations where worktree isolation may have failed or where conflicting instances are running. Display clear error messages with resolution steps when collisions are detected.

## Acceptance Criteria

- [ ] Detect if another instance is using the same data directory (lock file check)
- [ ] Detect if worktree-specific paths already exist from a different worktree with same hash (unlikely but possible)
- [ ] Display user-friendly error dialog explaining the collision
- [ ] Provide specific resolution steps (e.g., "Close the other instance" or "Run cleanup script")
- [ ] Log collision details to console for debugging
- [ ] Allow app to continue in read-only mode if user acknowledges warning (optional)

## Test Cases

- [ ] No error when data directories are unused
- [ ] Error shown when lock file exists from another running instance
- [ ] Error includes the path to the conflicting resource
- [ ] Resolution steps are actionable and accurate
- [ ] App can be force-started with acknowledgment (if implemented)

## Dependencies

- worktree-detection (provides worktree identifier for path construction)
- worktree-paths (provides resolved paths to check for conflicts)

## Preconditions

- worktree-detection and worktree-paths are implemented
- Lock file mechanism defined (e.g., `heycat.lock` in data directory)

## Implementation Notes

- Create lock file on startup: `~/.local/share/heycat-{id}/heycat.lock`
- Lock file contains PID and timestamp
- Check if lock file exists and if PID is still running
- On macOS/Linux: use `kill(pid, 0)` to check if process exists
- On Windows: use process enumeration API
- Use Tauri dialog for error display: `tauri::api::dialog::message()`
- Clean up lock file on graceful shutdown

## Related Specs

- worktree-detection (dependency)
- worktree-paths (dependency - provides paths to lock)
- worktree-cleanup-script (can clean stale lock files)

## Integration Points

- Production call site: `src-tauri/src/lib.rs::setup()` - after path resolution, before store init
- Connects to:
  - worktree-paths module for path resolution
  - Tauri dialog API for error display
  - App lifecycle (shutdown hook for lock cleanup)

## Integration Test

- Test location: Manual testing - start two instances from same worktree
- Verification: [ ] Integration test passes

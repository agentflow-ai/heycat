---
status: pending
created: 2025-12-23
completed: null
dependencies: ["worktree-detection", "worktree-paths"]
---

# Spec: Bun script to clean up worktree-specific data

## Description

Create a Bun script (`scripts/cleanup-worktree.ts`) that removes worktree-specific data directories and configuration files. Since worktree data is stored outside the git directory, this script is needed to properly clean up when a worktree is removed.

## Acceptance Criteria

- [ ] Script accepts worktree path or identifier as argument
- [ ] Script can list all worktree-specific data directories (with `--list` flag)
- [ ] Script can clean up data for a specific worktree (by path or ID)
- [ ] Script can clean up orphaned data (worktrees that no longer exist) with `--orphaned` flag
- [ ] Script requires confirmation before deleting (or `--force` to skip)
- [ ] Script removes data dir (`~/.local/share/heycat-{id}/`)
- [ ] Script removes config dir (`~/.config/heycat-{id}/`)
- [ ] Script optionally removes the git worktree itself with `--remove-worktree` flag

## Test Cases

- [ ] Lists all heycat worktree data directories correctly
- [ ] Identifies orphaned directories (worktree removed but data remains)
- [ ] Deletes correct directories for specified worktree
- [ ] Does not delete data for wrong worktree
- [ ] Prompts for confirmation before deletion
- [ ] `--force` skips confirmation

## Dependencies

- worktree-detection (need identifier algorithm to match data dirs)
- worktree-paths (need to know all paths that may contain worktree data)

## Preconditions

- Bun runtime available
- User has permissions to delete data directories

## Implementation Notes

- Location: `scripts/cleanup-worktree.ts`
- Scan `~/.local/share/` for directories matching `heycat-*` pattern
- Scan `~/.config/` for directories matching `heycat-*` pattern
- For orphan detection: check if corresponding git worktree still exists
  - Parse `.git/worktrees/` in main repo to find valid worktrees
  - Compare against data directories
- Use `rimraf` or `fs.rm` with `recursive: true` for deletion
- Colorized output for better UX (green=safe, red=will delete)

## Related Specs

- worktree-detection (uses same identifier algorithm)
- worktree-paths (defines what paths to clean)
- worktree-create-script (companion script for creation)

## Integration Points

- Production call site: N/A - developer utility script
- Connects to:
  - Git CLI (`git worktree list` for orphan detection)
  - File system (directory listing and deletion)
  - worktree-detection algorithm (for identifier matching)

## Integration Test

- Test location: Manual testing - create worktree, create data, run cleanup
- Verification: [ ] Integration test passes

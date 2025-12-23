---
status: pending
created: 2025-12-23
completed: null
dependencies: ["worktree-detection", "worktree-config"]
---

# Spec: Bun script to create new worktrees with proper setup

## Description

Create a Bun script (`scripts/create-worktree.ts`) that automates the creation of new git worktrees with proper heycat setup. The script handles git worktree creation, generates a unique default hotkey, and provides instructions for running the dev server.

## Acceptance Criteria

- [ ] Script creates worktree via `git worktree add` at specified path
- [ ] Script calculates the worktree identifier using same algorithm as Rust backend
- [ ] Script creates initial settings file with unique default hotkey
- [ ] Script displays the worktree path and generated hotkey
- [ ] Script provides instructions for running dev server in new worktree
- [ ] Script validates that branch/path doesn't already exist
- [ ] Script handles errors gracefully with helpful messages

## Test Cases

- [ ] Creates worktree successfully with valid branch name
- [ ] Fails gracefully if branch already exists
- [ ] Fails gracefully if worktree path already exists
- [ ] Generated hotkey is unique (e.g., based on worktree hash)
- [ ] Settings file is valid JSON with correct structure

## Dependencies

- worktree-detection (need to match identifier algorithm)
- worktree-config (need to match settings file format and location)

## Preconditions

- Git repository with at least one commit
- Bun runtime available
- Running from main repository (not from a worktree)

## Implementation Notes

- Location: `scripts/create-worktree.ts`
- Use `Bun.spawn` for git commands
- Hotkey generation: Use worktree hash to pick from predefined set of hotkeys
  - E.g., `Cmd+Shift+1`, `Cmd+Shift+2`, etc. based on hash modulo
  - Or generate based on branch name: `Cmd+Shift+{first letter}`
- Settings file location: `~/.local/share/heycat-{id}/settings.json` (match Tauri store path)
- TypeScript for type safety and better DX

## Related Specs

- worktree-detection (must match identifier algorithm)
- worktree-config (must match settings format)
- worktree-cleanup-script (companion script for teardown)

## Integration Points

- Production call site: N/A - developer utility script
- Connects to:
  - Git CLI (`git worktree add`)
  - File system (settings file creation)
  - worktree-detection algorithm (for identifier calculation)

## Integration Test

- Test location: Manual testing - run script and verify worktree + settings
- Verification: [ ] Integration test passes

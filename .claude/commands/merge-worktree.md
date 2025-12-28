---
description: Merge a completed worktree feature into main
---

# Merge Worktree Feature to Main

You are completing a feature that was developed in a git worktree and merging it to main.

## Prerequisites Check

1. Verify you are in a worktree (not main repo):
   - `.git` should be a file, not a directory
   - If in main repo, inform the user this command only works from worktrees

2. Check for clean working directory:
   ```bash
   git status --porcelain
   ```
   - If dirty, ask user to commit or stash changes first

## Execution Flow

### Step 1: Run the completion script

```bash
bun scripts/complete-feature.ts
```

This script will:
1. Fetch latest main
2. Rebase your feature onto main
3. Squash all commits into a single conventional commit (derived from WIP messages)
4. Fast-forward merge to main
5. Reset the worktree to main (ready for next feature)

### Step 2: Handle conflicts (if any)

**CRITICAL: DO NOT resolve conflicts automatically. Human approval is REQUIRED.**

If the script reports conflicts, you MUST:

1. **STOP** - Do not proceed with resolution automatically

2. **SHOW the user the conflicts** - For each conflicting file:
   - Display the file path
   - Show the conflict markers with both versions
   - Explain what "ours" (feature branch) contains
   - Explain what "theirs" (main branch) contains

3. **ASK the user for resolution** - Present options like:
   - "Keep our version (feature branch)"
   - "Keep their version (main branch)"
   - "Merge both changes (explain how)"
   - "Let me resolve manually"

4. **WAIT for explicit user approval** before making any changes

5. **Only after user approves**, implement their chosen resolution:
   - Make the approved changes
   - Stage resolved files:
     ```bash
     git add <files>
     ```
   - Continue the rebase:
     ```bash
     git rebase --continue
     ```
   - Re-run the completion script:
     ```bash
     bun scripts/complete-feature.ts --continue
     ```

**Why this matters:** Automatic conflict resolution can silently discard important changes. The user must understand and approve how conflicts are resolved.

### Step 3: Update Linear

After successful completion:

1. Confirm the squashed commit message is appropriate for the feature
2. Verify the worktree is reset and ready for the next feature
3. Move the associated issue to done in Linear:
   ```bash
   bun <agile-plugin-path>/agile.ts move <issue-name> 5-done
   ```
   - If the issue name isn't obvious, ask the user which issue to update

## Notes

- This creates a single conventional commit on main
- The worktree branch is preserved but reset to main
- The associated Linear issue should be moved to done after merge
- The commit message is automatically derived from your WIP commits

## Troubleshooting

**"This script must be run from a worktree"**
- You're in the main repo. Navigate to your worktree directory first.

**"Working directory is not clean"**
- Commit or stash your changes before completing the feature.

**"No commits to merge"**
- Your branch is already up to date with main. Nothing to do.

**"Fast-forward merge failed"**
- Main has diverged since you rebased. Run `git rebase origin/main` again and retry.

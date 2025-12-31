# CLAUDE.md
 
## Project Overview

heycat is a Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Quick Reference

| Topic | Keywords | Info |
|-------|----------|------|
| Architecture | frontend, backend, Tauri, React, Rust, invoke | docs/ARCHITECTURE.md |
| Development | commands, dev, build, run, prerequisites | docs/DEVELOPMENT.md |
| Agile Workflow | issue, feature, bug, task, spec, kanban, backlog | `devloop:agile` plugin (Linear backend) |
| TCR/Testing | writing and test, TDD, coverage, commit, tcr check | `devloop:tcr` plugin and docs/TESTING.md |
| Docker Dev | container, cloud, remote, mac-build, rsync | docs/DEVELOPMENT.md#docker-development |

## ⚠️ Required: Isolated Development Environment

**Before running `/devloop:agile:loop` or implementing any feature/bug/spec, you MUST create an isolated environment:**

**Option A: Git Worktree (macOS local development)**
```
/create-worktree
```

**Option B: Docker Container (cloud/remote development)**
```
/create-container
```

This project uses the "cattle model" - environments are ephemeral and disposable. Never develop directly on main.

**Worktree workflow (macOS):**
1. `/create-worktree` - Create worktree for the issue (from main repo)
2. `/devloop:agile:loop` - Implement specs in the worktree
3. `/submit-pr` - Push and create PR when done
4. `/close-worktree` - Delete worktree after PR merges

**Docker workflow (cloud/remote):**
1. `/create-container` - Create container for the issue
2. `/devloop:agile:loop` - Implement specs in the container
3. `/mac-build` - Build Tauri app on macOS host (when needed)
4. `/submit-pr` - Push and create PR when done
5. `/close-container` - Delete container after PR merges

---

## Key Entry Points

IMPORTANT: You must use these to discover the stated topics. Dont assume things within the areas each entry point describes.
You may never use npm or npx, always use bun or bunx.

### Development
**When:** Starting dev server, building, type-checking, setting up prerequisites
**File:** docs/DEVELOPMENT.md

### Architecture
**When:** Understanding project structure, frontend-backend communication, adding Tauri commands, searching for code or previous implementations. Use when planning new code, implementing code or updating code in the project. If you are working on a feature, spec or bug, you must read this.
**File:** docs/ARCHITECTURE.md

### Agile Workflow
**ALWAYS invoke the `devloop:agile` skill** for issue and spec management, code reviews, and workflow tasks like writing code.

**PREREQUISITE:** Create an isolated environment first (see "Isolated Development Environment" above). Never run `/devloop:agile:loop` from main.

**IMPORTANT:** The `agile` command is NOT a system CLI. Do NOT run `agile ...` directly in bash - it will fail with "command not found".

**Correct approach:**
1. `/create-worktree` or `/create-container` - Set up isolated development environment
2. Use `Skill(devloop:agile)` to get the command documentation
3. Run commands via bun: `bun <plugin-path>/agile.ts <command> [args]`


### TCR (Test-Commit-Refactor) - Writing and Running tests
**When:** Writing tests, Running tests, checking coverage, test-driven development. You will always use tcr to run and write tests when working with feature specs or bugs, ensure to read this before you begin work on any such tasks. You must load this file whenever anything related to testing is being done.
**File:** docs/TESTING.md

### Review Independence
**When:** Code reviews for specs or features
Reviews must use a **fresh subagent** with no implementation context. Use `/devloop:agile:review`.

### Worktrees (Cattle Model)
See "⚠️ Required: Isolated Development Environment" section above. Additional commands:
- `/sync-worktree` - Rebase worktree onto latest main (use when main has updates)
- `/submit-pr` - Push branch and create PR (run from worktree when specs complete)
- `/close-worktree` - Delete worktree after PR is merged

**Starting Claude in a worktree:**
To ensure subagents (like `/devloop:agile:review`) operate in the correct directory, use:
```bash
# Create new worktree and start Claude there
./scripts/start-worktree-session.sh feature/my-feature

# Resume session in existing worktree
./scripts/start-worktree-session.sh --resume my-feature

# List available worktrees
./scripts/start-worktree-session.sh --list
```

### Docker Containers (Cattle Model)
Alternative to worktrees for cloud/remote development. See docs/DEVELOPMENT.md#docker-development.

**Commands:**
- `/create-container` - Create Docker container for feature development
- `/close-container` - Stop and remove container after PR is merged
- `/mac-build` - Sync code and trigger Tauri build on macOS host

**When to use Docker:**
- Cloud/remote development without direct macOS access
- CI/CD pipelines
- Testing on Linux

**macOS host setup** (required for `/mac-build`):
```bash
# Add to .env
HEYCAT_MAC_HOST=192.168.1.100
HEYCAT_MAC_USER=myuser
HEYCAT_MAC_PATH=~/heycat-docker
```

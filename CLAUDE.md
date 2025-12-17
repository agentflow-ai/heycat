# CLAUDE.md
 
## Project Overview

heycat is a Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Quick Reference

| Topic | Keywords | Info |
|-------|----------|------|
| Architecture | frontend, backend, Tauri, React, Rust, invoke | docs/ARCHITECTURE.md |
| Development | commands, dev, build, run, prerequisites | docs/DEVELOPMENT.md |
| Agile Workflow | issue, feature, bug, task, spec, kanban, backlog | `devloop:agile` plugin |
| TCR/Testing | writing and test, TDD, coverage, commit, tcr check | `devloop:tcr` plugin and docs/TESTING.md |

## Key Entry Points

### Development
**When:** Starting dev server, building, type-checking, setting up prerequisites
**File:** docs/DEVELOPMENT.md

### Architecture
**When:** Understanding project structure, frontend-backend communication, adding Tauri commands, searching for code or previous implementations
**File:** docs/ARCHITECTURE.md

### Agile Workflow
**ALWAYS invoke the `devloop:agile` skill** for issue and spec management, code reviews, and workflow tasks.

**IMPORTANT:** The `agile` command is NOT a system CLI. Do NOT run `agile ...` directly in bash - it will fail with "command not found".

**Correct approach:**
1. Use `Skill(devloop:agile)` to get the command documentation
2. Run commands via bun: `bun <plugin-path>/agile.ts <command> [args]`


### TCR (Test-Commit-Refactor)
**Invoke the `devloop:tcr` skill** for test discipline and coverage enforcement.

**Testing Philosophy:**
Before writing a test, ensure to have looked at docs/TESTING.md

#### Quick Tests (for specs and spec reviews)
Use these for fast feedback during development - no coverage overhead:
```bash
# Quick: Both frontend and backend (~3-5s)
tcr check "bun run test && cd src-tauri && cargo test"

# Quick: Frontend only
tcr check "bun run test"

# Quick: Backend only
tcr check "cd src-tauri && cargo test"
```

#### Full Coverage Tests (for feature reviews only)
Use these only during `/devloop:agile:feature-review` to verify coverage thresholds:
```bash
# Full: Both frontend and backend (slower, includes coverage)
tcr check "bun run test:coverage && cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'"

# Full: Frontend only
tcr check "bun run test:coverage"

# Full: Backend only
tcr check "cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'"
```

#### TCR in Agile Workflow
| Workflow Stage | Test Command |
|----------------|--------------|
| Spec implementation | Quick tests |
| `/devloop:agile:review` (spec review) | Quick tests |
| `/devloop:agile:feature-review` | Full coverage tests |

#### TCR Status Commands
```bash
tcr status  # Check current TCR state
tcr reset   # Reset after failures
```

### Review Independence

Reviews must be performed by a **fresh subagent** with no implementation context. Use `/devloop:agile:review`.

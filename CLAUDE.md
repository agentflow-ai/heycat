# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

heycat is a Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Quick Reference

| Topic | Keywords | Info |
|-------|----------|------|
| Architecture | frontend, backend, Tauri, React, Rust, invoke | docs/ARCHITECTURE.md |
| Development | commands, dev, build, run, prerequisites | docs/DEVELOPMENT.md |
| Agile Workflow | issue, feature, bug, task, spec, kanban, backlog | `devloop:agile` plugin |
| TCR/Testing | test, TDD, coverage, commit, tcr check | `devloop:tcr` plugin |
| Integration | multi-component, mocks, verification, deferral | docs/INTEGRATION.md |

## Key Entry Points

### Development
**When:** Starting dev server, building, type-checking, setting up prerequisites
**File:** docs/DEVELOPMENT.md

### Architecture
**When:** Understanding project structure, frontend-backend communication, adding Tauri commands
**File:** docs/ARCHITECTURE.md

### Agile Workflow
**ALWAYS invoke the `devloop:agile` skill** for issue and spec management, code reviews, and workflow tasks.

**IMPORTANT:** The `agile` command is NOT a system CLI. Do NOT run `agile ...` directly in bash - it will fail with "command not found".

**Correct approach:**
1. Use `Skill(devloop:agile)` to get the command documentation
2. Run commands via bun: `bun <plugin-path>/agile.ts <command> [args]`

**Available slash commands:**
- `/devloop:agile:feature` - Guided feature creation
- `/devloop:agile:quick` - Quick feature (bypasses full BDD)
- `/devloop:agile:discover` - BDD scenario discovery
- `/devloop:agile:next` - Continue with next spec
- `/devloop:agile:status` - Show issue status
- `/devloop:agile:review` - Review spec (independent subagent)
- `/devloop:agile:fix` - Fix failed review feedback
- `/devloop:agile:spec-suggest` - AI-assisted spec breakdown

### TCR (Test-Commit-Refactor)
**Invoke the `devloop:tcr` skill** for test discipline and coverage enforcement.

**Testing Philosophy:** Focus on smoke testing the most valuable paths (60% coverage threshold). Prioritize iteration speed over exhaustive coverage.

**Example commands:**
```bash
# Frontend tests
tcr check "bun run test:coverage"

# Backend tests
tcr check "cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'"

# Both frontend and backend
tcr check "bun run test:coverage && cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'"

# Check status / reset after failures
tcr status
tcr reset
```

### Integration Verification
**When:** Multi-component features, verifying mocks, deferral tracking, feature completion gates
**File:** docs/INTEGRATION.md

### Review Independence

**NEVER self-review your own implementation.** When you implement a spec:
- DO NOT add a "## Review" section
- DO NOT mark acceptance criteria as verified
- DO NOT update spec status to "completed"

Reviews must be performed by a **fresh subagent** with no implementation context. Use `/devloop:agile:review`.

## Slash Commands

### TCR & Git
- `/devloop:tcr:check` - Run TCR check in subagent
- `/devloop:git:commit` - Git commit in subagent

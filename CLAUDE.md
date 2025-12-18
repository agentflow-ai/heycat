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

IMPORTANT: You must use these to discover the stated topics. Dont assume things within the areas each entry point describes.

### Development
**When:** Starting dev server, building, type-checking, setting up prerequisites
**File:** docs/DEVELOPMENT.md

### Architecture
**When:** Understanding project structure, frontend-backend communication, adding Tauri commands, searching for code or previous implementations. Use when planning new code, implementing code or updating code in the project. If you are working on a feature, spec or bug, you must read this.
**File:** docs/ARCHITECTURE.md

### Agile Workflow
**ALWAYS invoke the `devloop:agile` skill** for issue and spec management, code reviews, and workflow tasks.

**IMPORTANT:** The `agile` command is NOT a system CLI. Do NOT run `agile ...` directly in bash - it will fail with "command not found".

**Correct approach:**
1. Use `Skill(devloop:agile)` to get the command documentation
2. Run commands via bun: `bun <plugin-path>/agile.ts <command> [args]`


### TCR (Test-Commit-Refactor) - Writing and Running tests
**When:** Writing tests, Running tests, checking coverage, test-driven development. You will always run and write tests when working with feature specs or bugs, ensure to read this before you begin work on any such tasks.
**File:** docs/TESTING.md

### Review Independence
**When:** Code reviews for specs or features
Reviews must use a **fresh subagent** with no implementation context. Use `/devloop:agile:review`.

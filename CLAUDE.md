# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

heycat is a Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Quick Reference

| Topic | Keywords | File |
|-------|----------|------|
| Architecture | frontend, backend, Tauri, React, Rust, invoke | docs/ARCHITECTURE.md |
| Development | commands, dev, build, run, prerequisites | docs/DEVELOPMENT.md |
| Agile Workflow | issue, feature, bug, task, spec, kanban, backlog | .claude/skills/agile/SKILL.md |
| TCR/Testing | test, TDD, coverage, commit, tcr check | .claude/skills/tcr/SKILL.md |
| Integration | multi-component, mocks, verification, deferral | docs/INTEGRATION.md |

## Key Entry Points

### Development
**When:** Starting dev server, building, type-checking, setting up prerequisites
**File:** docs/DEVELOPMENT.md

### Architecture
**When:** Understanding project structure, frontend-backend communication, adding Tauri commands
**File:** docs/ARCHITECTURE.md

### Agile Workflow
**When:** Creating/managing issues, working on specs, moving through workflow stages, code reviews, creating technical guidance
**Triggers:** "create feature", "next spec", "issue status", "review", "what's next"
**File:** .claude/skills/agile/SKILL.md

**ALWAYS invoke the `agile` skill** when the user mentions:
- Creating features, bugs, or tasks
- Working on or resuming issues
- Spec status or progress
- Moving issues through stages
- Listing or managing backlog

### TCR (Test-Commit-Refactor)
**When:** Writing tests, running coverage, making commits, TDD workflow
**Triggers:** "tcr check", "run tests", "coverage", "commit"
**File:** .claude/skills/tcr/SKILL.md

**Invoke the `tcr` skill** for test discipline and coverage enforcement.

### Integration Verification
**When:** Multi-component features, verifying mocks, deferral tracking, feature completion gates
**File:** docs/INTEGRATION.md

### Review Independence

**NEVER self-review your own implementation.** When you implement a spec:
- DO NOT add a "## Review" section
- DO NOT mark acceptance criteria as verified
- DO NOT update spec status to "completed"

Reviews must be performed by a **fresh subagent** with no implementation context. Use `/agile:review`.

## Slash Commands

- `/agile:next` - Auto-find and implement the next spec in the in-progress issue
- `/agile:status` - Show current issue status and progress
- `/agile:review` - Run independent code review using a fresh subagent

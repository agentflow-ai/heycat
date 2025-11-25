---
name: agile
description: |
  Manage the project's Kanban-style agile workflow. Use this skill when you need to:
  - Create new features, bugs, or tasks in the backlog
  - Move issues through workflow stages (backlog -> todo -> in-progress -> review -> done)
  - List current issues and their status
  - Archive or delete completed work
---

# Agile Workflow Management

Manage issues in the project's Kanban board located in the `agile/` folder.

## Workflow Stages

```
1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done
```

**Important:** Only sequential transitions are allowed (forward or back by one stage).

## Commands

### Create an Issue

```bash
bun .claude/skills/agile/agile.ts create <type> <name> [--title "Title"] [--stage <stage>]
```

**Arguments:**
- `type`: `feature`, `bug`, or `task`
- `name`: kebab-case identifier (e.g., `user-authentication`)

**Options:**
- `--title, -t`: Human-readable title (defaults to name in Title Case)
- `--stage, -s`: Initial stage (default: `1-backlog`)

**Examples:**
```bash
bun .claude/skills/agile/agile.ts create feature user-auth --title "User Authentication"
bun .claude/skills/agile/agile.ts create bug fix-login --stage 2-todo
bun .claude/skills/agile/agile.ts create task update-deps
```

### Move an Issue

```bash
bun .claude/skills/agile/agile.ts move <name> <stage>
```

**Arguments:**
- `name`: Issue name (with or without `.md` extension)
- `stage`: Target stage (`1-backlog`, `2-todo`, `3-in-progress`, `4-review`, `5-done`)

**Valid Transitions:**
- `1-backlog` can move to: `2-todo`
- `2-todo` can move to: `1-backlog`, `3-in-progress`
- `3-in-progress` can move to: `2-todo`, `4-review`
- `4-review` can move to: `3-in-progress`, `5-done`
- `5-done` can move to: `4-review` (reopen)

**Examples:**
```bash
bun .claude/skills/agile/agile.ts move user-auth 2-todo
bun .claude/skills/agile/agile.ts move user-auth 3-in-progress
```

### List Issues

```bash
bun .claude/skills/agile/agile.ts list [--stage <stage>] [--format table|json]
```

**Options:**
- `--stage, -s`: Filter by stage
- `--format, -f`: Output format (`table` or `json`, default: `table`)

**Examples:**
```bash
bun .claude/skills/agile/agile.ts list
bun .claude/skills/agile/agile.ts list --stage 3-in-progress
bun .claude/skills/agile/agile.ts list --format json
```

### Archive an Issue

```bash
bun .claude/skills/agile/agile.ts archive <name>
```

Moves the issue to `agile/archive/` with a timestamp suffix.

**Example:**
```bash
bun .claude/skills/agile/agile.ts archive completed-feature
# Result: agile/archive/completed-feature-2025-11-25.md
```

### Delete an Issue

```bash
bun .claude/skills/agile/agile.ts delete <name>
```

Permanently removes the issue file.

**Example:**
```bash
bun .claude/skills/agile/agile.ts delete old-task
```

### Get Help

```bash
bun .claude/skills/agile/agile.ts help [command]
```

## Typical Workflow

```bash
# 1. Create a new feature
bun .claude/skills/agile/agile.ts create feature dark-mode --title "Dark Mode Toggle"

# 2. Start working on it
bun .claude/skills/agile/agile.ts move dark-mode 2-todo
bun .claude/skills/agile/agile.ts move dark-mode 3-in-progress

# 3. Submit for review
bun .claude/skills/agile/agile.ts move dark-mode 4-review

# 4. Complete the work
bun .claude/skills/agile/agile.ts move dark-mode 5-done

# 5. Archive when no longer needed
bun .claude/skills/agile/agile.ts archive dark-mode
```

## Issue Types

- **feature**: New features and enhancements
- **bug**: Bug reports with reproduction steps
- **task**: General tasks and chores

## Naming Convention

Issue names must be in kebab-case: lowercase letters, numbers, and hyphens only.

Valid: `user-auth`, `fix-login-bug`, `update-deps-2024`
Invalid: `UserAuth`, `fix_login`, `update deps`

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

heycat is a Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Development Commands

```bash
# Start development mode (runs both frontend and Tauri)
bun run tauri dev

# Build production app
bun run tauri build

# Run frontend only (Vite dev server on port 1420)
bun run dev

# Type check and build frontend
bun run build
```

## Architecture

### Frontend (src/)
- React 18 with TypeScript
- Vite bundler (port 1420 for dev)
- Entry point: `src/main.tsx` → `src/App.tsx`
- Communicates with Rust backend via `invoke()` from `@tauri-apps/api/core`

### Backend (src-tauri/)
- Rust with Tauri v2
- Entry point: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs`
- Tauri commands are defined with `#[tauri::command]` attribute and registered in `invoke_handler()`
- Config: `src-tauri/tauri.conf.json`

### Frontend-Backend Communication
```typescript
// Frontend: call Rust command
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("command_name", { arg1, arg2 });
```

```rust
// Backend: define command in lib.rs
#[tauri::command]
fn command_name(arg1: &str, arg2: i32) -> String {
    // implementation
}
// Register in invoke_handler: tauri::generate_handler![command_name]
```

## Agile Workflow

Project uses a Kanban-style issue tracking system in `/agile`. Use the `agile` skill to create, move, list, and archive issues.

- **Work through issues**: Use the `agile-workflow` agent or `work` command for stage-appropriate guidance
- **Strict validation**: Transitions require complete content (description, owner, DoD items)

## TCR (Test-Commit-Refactor) Workflow

The TCR skill enforces test discipline through two layers:

### Development Feedback (Claude Code Hook)
1. **Write a failing test first** (red)
2. **Write code to make the test pass** (green)
3. **Mark todo as completed** → tests run automatically
4. **Tests pass** → WIP commit created automatically
5. **Tests fail** → continue refactoring (after 5 failures, reconsider approach)

### Pre-Commit Enforcement (Husky)
- Husky runs `bun test --coverage` before every commit
- Requires **80% line and function coverage** (configured in `bunfig.toml`)
- Commits blocked if tests fail or coverage is insufficient

**Commands:**
```bash
bun .claude/skills/tcr/tcr.ts run <files>   # Run tests manually
bun .claude/skills/tcr/tcr.ts status        # Show current state
bun .claude/skills/tcr/tcr.ts reset         # Reset failure counter
bun test --coverage                          # Run all tests with coverage
```

**Test discovery**: Convention-based (`foo.ts` → `foo.test.ts` or `foo.spec.ts`)

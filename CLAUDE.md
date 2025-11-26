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

Project uses a Kanban-style issue tracking system in `/agile`.

**When to use:** Invoke the `agile` skill when the user wants to:
- Create a new feature, bug, or task
- Work on or refine an existing issue
- Move issues through workflow stages
- List, archive, or delete issues

The skill provides full documentation and CLI commands. Transitions require complete content (description, owner, DoD items).

## TCR (Test-Commit-Refactor) Workflow

The TCR skill enforces test discipline through two layers for both frontend and backend:

### Development Feedback (Claude Code Hook)
1. **Write a failing test first** (red)
2. **Write code to make the test pass** (green)
3. **Mark todo as completed** → tests run automatically
4. **Tests pass** → WIP commit created automatically
5. **Tests fail** → continue refactoring (after 5 failures, reconsider approach)

### Pre-Commit Enforcement (Husky)
- Husky runs both **frontend** and **backend** tests with coverage before every commit
- **100% coverage required** for both frontend and backend
- Untestable code must be explicitly excluded (see Coverage Exclusions below)
- Commits blocked if tests fail or coverage is insufficient

### Coverage Exclusions

Both frontend and backend require 100% coverage. Use inline exclusion comments for untestable code:

#### Frontend (TypeScript/React)

Uses Vitest with `/* v8 ignore */` comments:

```typescript
/* v8 ignore next */
setGreetMsg(await invoke("greet", { name })); // Single line

/* v8 ignore start */
// Multiple lines excluded
ReactDOM.createRoot(document.getElementById("root")).render(<App />);
/* v8 ignore stop */
```

#### Backend (Rust)

Uses `#[coverage(off)]` attribute (requires nightly):

```rust
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn untestable_function() {
    // ...
}
```

### Prerequisites

```bash
# Required for Rust coverage (commits will be blocked without it)
rustup toolchain install nightly
cargo install cargo-llvm-cov
```

### Commands

```bash
bun .claude/skills/tcr/tcr.ts run <files>       # Run tests manually
bun .claude/skills/tcr/tcr.ts status            # Show current state
bun .claude/skills/tcr/tcr.ts status --coverage # Show state with coverage metrics
bun .claude/skills/tcr/tcr.ts coverage          # Run coverage checks (both targets)
bun .claude/skills/tcr/tcr.ts coverage frontend # Run frontend coverage only
bun .claude/skills/tcr/tcr.ts coverage backend  # Run backend coverage only
bun .claude/skills/tcr/tcr.ts verify-config     # Verify coverage thresholds are in sync
bun .claude/skills/tcr/tcr.ts reset             # Reset failure counter and clear error log
bun .claude/skills/tcr/tcr.ts help              # Show help message
```

### Test Discovery
- **Frontend**: Convention-based (`foo.ts` → `foo.test.ts` or `foo.spec.ts`)
- **Backend**: Rust tests in `#[cfg(test)]` modules (`src-tauri/src/*.rs`)

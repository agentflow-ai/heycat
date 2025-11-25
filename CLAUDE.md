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

Project uses a Kanban-style issue tracking system in `/agile`:

```
1-backlog → 2-todo → 3-in-progress → 4-review → 5-done
```

### Creating Issues
```bash
cp agile/templates/feature.md agile/1-backlog/my-feature.md
cp agile/templates/bug.md agile/1-backlog/fix-something.md
cp agile/templates/task.md agile/1-backlog/some-task.md
```

### Moving Issues
```bash
git mv agile/1-backlog/my-feature.md agile/3-in-progress/
```

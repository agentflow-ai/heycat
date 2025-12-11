# Architecture

heycat is a Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Frontend (src/)

- React 18 with TypeScript
- Vite bundler (port 1420 for dev)
- Entry point: `src/main.tsx` → `src/App.tsx`
- Communicates with Rust backend via `invoke()` from `@tauri-apps/api/core`

## Backend (src-tauri/)

- Rust with Tauri v2
- Entry point: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs`
- Tauri commands are defined with `#[tauri::command]` attribute and registered in `invoke_handler()`
- Config: `src-tauri/tauri.conf.json`

## Frontend-Backend Communication

### Frontend: Call Rust Command

```typescript
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("command_name", { arg1, arg2 });
```

### Backend: Define Command

```rust
// Define in lib.rs
#[tauri::command]
fn command_name(arg1: &str, arg2: i32) -> String {
    // implementation
}

// Register in invoke_handler
tauri::generate_handler![command_name]
```

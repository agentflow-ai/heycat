---
paths: "src-tauri/src/**/*.rs"
---

# Backend Patterns

```
                              COMMAND FLOW
───────────────────────────────────────────────────────────────────────
Frontend invoke()
       │
       ▼
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  Tauri Command   │────▶│   _impl Function │────▶│  Domain Logic    │
│  (thin wrapper)  │     │   (testable)     │     │  (state/DB/etc)  │
│  recording.rs    │     │  logic.rs        │     │  RecordingManager│
└──────────────────┘     └──────────────────┘     └──────────────────┘
       │                                                   │
       │◀──────────────── Result<T, String> ───────────────┘
       │
       ▼
┌──────────────────┐
│  Event Emission  │ (optional: emit_or_warn!)
└──────────────────┘
```

## Async vs Sync

- **Async:** I/O operations (database, network, file I/O)
- **Sync:** State operations (`Arc<Mutex<T>>`), CPU-bound, pure transforms
- **`run_async`:** Only for bridging sync→async (creates new runtime, use sparingly)
- **`spawn_blocking`:** For CPU-intensive work within async commands

## Commands

Commands in `src-tauri/src/commands/` follow the thin wrapper pattern:

- Wrappers delegate to `_impl` functions in separate files
- Use state type aliases from `crate::app::state` (e.g., `ProductionState`)
- Return `Result<T, String>` (Tauri serializes errors as strings)
- Pass state with `.as_ref()` to `_impl` functions

```rust
// commands/recording.rs - thin wrapper
#[tauri::command]
pub fn get_recording_state(state: State<'_, ProductionState>) -> Result<RecordingStateInfo, String> {
    get_recording_state_impl(state.as_ref())
}

// logic.rs - testable implementation
pub fn get_recording_state_impl(state: &Mutex<RecordingManager>) -> Result<RecordingStateInfo, String> {
    let manager = state.lock().map_err(|_| "Unable to access recording state.")?;
    Ok(manager.get_state_info())
}
```

Wrappers may include: device checks, event emission, error mapping.

## Errors

- Custom error enums with manual `Display` + `Error` impl (no `thiserror`)
- Marker constants for error detection: `"[MICROPHONE_ACCESS_ERROR]"`
- Serializable errors: `#[serde(tag = "type", rename_all = "camelCase")]`
- Logging: `crate::error!()`, `crate::warn!()`, `crate::info!()`, `crate::debug!()`

## Module Structure

- `mod.rs`: submodule declarations + `pub use` re-exports
- Tests in separate files: `foo.rs` → `foo_test.rs`, declared with `#[path = "foo_test.rs"]`
- Shared test utilities in `src-tauri/src/test_utils/`
- Platform code guarded: `#[cfg(target_os = "macos")]`
- Commands module excluded from coverage: `#![cfg_attr(coverage_nightly, coverage(off))]`

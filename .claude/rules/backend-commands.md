---
paths: "src-tauri/src/commands/**/*.rs"
---

# Backend Commands Pattern

## Module Structure

Commands are organized in `src-tauri/src/commands/`:
- `mod.rs` - Module exports and coverage exclusion
- `recording.rs`, `audio.rs`, etc. - Tauri command wrappers
- `logic.rs` - Testable implementation functions
- `common.rs` - Shared utilities (e.g., `TauriEventEmitter`)

## Coverage Exclusion

The commands module (containing Tauri wrappers) is excluded from coverage since the actual logic is tested separately:

```rust
// In commands/mod.rs
#![cfg_attr(coverage_nightly, coverage(off))]
```

## State Type Aliases

Use centralized type aliases from `crate::app::state` for managed state:

```rust
// In app/state.rs
pub type ProductionState = Arc<Mutex<RecordingManager>>;
pub type TursoClientState = Arc<TursoClient>;
pub type AudioThreadState = Arc<AudioThreadHandle>;

// In commands - re-export for convenience
pub use crate::app::state::{ProductionState, TursoClientState, AudioThreadState};
```

## Thin Wrapper Pattern

Tauri commands should be thin wrappers that delegate to `_impl` functions in `logic.rs`:

```rust
// GOOD: Thin wrapper in recording.rs
#[tauri::command]
pub fn get_recording_state(state: State<'_, ProductionState>) -> Result<RecordingStateInfo, String> {
    get_recording_state_impl(state.as_ref())
}

#[tauri::command]
pub fn clear_last_recording_buffer(state: State<'_, ProductionState>) -> Result<(), String> {
    clear_last_recording_buffer_impl(state.as_ref())
}
```

```rust
// GOOD: Testable logic in logic.rs (separate file)
pub fn get_recording_state_impl(
    state: &Mutex<RecordingManager>,
) -> Result<RecordingStateInfo, String> {
    let manager = state.lock().map_err(|_| {
        "Unable to access recording state."
    })?;
    Ok(RecordingStateInfo { state: manager.get_state() })
}
```

## When Wrappers Can Have Logic

Wrappers may contain:
- Device/model availability checks before calling `_impl`
- Event emission after `_impl` success/failure
- Turso storage calls (already async, fits naturally in async command)
- Error mapping to user-friendly messages

```rust
// ACCEPTABLE: Pre-checks and event emission in wrapper
#[tauri::command]
pub fn start_recording(
    app_handle: AppHandle,
    state: State<'_, ProductionState>,
    device_name: Option<String>,
) -> Result<(), String> {
    // Pre-check: device availability
    let devices = crate::audio::list_input_devices();
    if devices.is_empty() {
        emit_or_warn!(app_handle, event_names::AUDIO_DEVICE_ERROR, error);
        return Err(error.to_string());
    }

    // Delegate to testable impl
    let result = start_recording_impl(state.as_ref(), device_name);

    // Post-action: event emission
    if result.is_ok() {
        emit_or_warn!(app_handle, event_names::RECORDING_STARTED, payload);
    }
    result
}
```

## Anti-Patterns

### Business logic in wrappers

```rust
// BAD: Complex logic directly in wrapper
#[tauri::command]
pub fn process_recording(state: State<'_, ProductionState>) -> Result<Data, String> {
    let manager = state.lock().unwrap();
    let samples = manager.get_samples();

    // Complex processing that should be in logic.rs
    let normalized = samples.iter().map(|s| s / max).collect();
    let filtered = apply_filter(&normalized);
    let result = analyze(&filtered);

    Ok(result)
}

// GOOD: Move to logic.rs
#[tauri::command]
pub fn process_recording(state: State<'_, ProductionState>) -> Result<Data, String> {
    process_recording_impl(state.as_ref())
}
```

### Missing state type aliases

```rust
// BAD: Inline complex types
pub fn start_recording(
    state: State<'_, Arc<Mutex<RecordingManager>>>,
) -> Result<(), String>

// GOOD: Use type alias from app::state
pub fn start_recording(
    state: State<'_, ProductionState>,
) -> Result<(), String>
```

### Direct state access without `as_ref()`

```rust
// BAD: Passing State<'_, T> directly to _impl
start_recording_impl(state, device_name)

// GOOD: Dereference to inner type
start_recording_impl(state.as_ref(), device_name)
```

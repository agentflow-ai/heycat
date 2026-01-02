---
paths: "src-tauri/src/**/*.rs"
---

# Backend Async Usage Pattern

## When to Use Async

Use async functions only for I/O-bound operations:
- Database queries (Turso client operations)
- Network requests (HTTP, WebSocket)
- File I/O that benefits from non-blocking behavior
- Tauri commands that perform I/O operations

```rust
// GOOD: Async for database I/O
#[tauri::command]
pub async fn list_transcriptions(
    turso_client: State<'_, TursoClientState>,
) -> Result<Vec<TranscriptionInfo>, String> {
    turso_client.list_transcriptions().await
        .map_err(|e| format!("Failed to list transcriptions: {}", e))
}
```

## When to Use Sync

Use synchronous code for:
- State operations with `Arc<Mutex<T>>` (lock acquisition is fast)
- CPU-bound computations
- Pure data transformations
- Settings access

```rust
// GOOD: Sync for state operations
pub fn get_recording_state_impl(
    state: &Mutex<RecordingManager>,
) -> Result<RecordingStateInfo, String> {
    let manager = state.lock().map_err(|_| {
        "Unable to access recording state."
    })?;
    Ok(manager.get_state_info())
}
```

## Bridging Async to Sync: run_async()

When you need to call async code from a synchronous context (e.g., callbacks, handlers that must be sync), use `crate::util::run_async`:

```rust
use crate::util::run_async;

// In a sync callback that needs to call async code
fn sync_handler(&self) {
    let result = run_async(async {
        self.client.fetch_data().await
    });
    process(result);
}
```

**Note:** `run_async` creates a new Tokio runtime. Use sparingly and only when necessary.

## CPU-Bound Work in Async Commands

For CPU-intensive work within async commands, use `tokio::task::spawn_blocking`:

```rust
#[tauri::command]
pub async fn transcribe_file(
    shared_model: State<'_, Arc<SharedTranscriptionModel>>,
    file_path: String,
) -> Result<String, String> {
    let model = shared_model.inner().clone();
    let path = file_path.clone();

    // Run CPU-intensive transcription on blocking thread pool
    tokio::task::spawn_blocking(move || transcribe_file_impl(&model, &path))
        .await
        .map_err(|e| format!("Task failed: {}", e))?
}
```

## Anti-Patterns

### Unnecessary async for sync operations

```rust
// BAD: Async wrapper around sync state access
pub async fn get_state(state: &Mutex<Manager>) -> Result<State, String> {
    let manager = state.lock().unwrap();  // No await needed
    Ok(manager.get_state())
}

// GOOD: Keep it sync
pub fn get_state(state: &Mutex<Manager>) -> Result<State, String> {
    let manager = state.lock().unwrap();
    Ok(manager.get_state())
}
```

### Blocking in async context

```rust
// BAD: Blocking call in async function
pub async fn process_file(path: &str) -> Result<Data, Error> {
    let content = std::fs::read_to_string(path)?;  // Blocks the async runtime
    parse(content)
}

// GOOD: Use spawn_blocking for blocking I/O
pub async fn process_file(path: &str) -> Result<Data, Error> {
    let path = path.to_string();
    tokio::task::spawn_blocking(move || {
        let content = std::fs::read_to_string(&path)?;
        parse(content)
    }).await?
}
```

### Calling run_async from within async context

```rust
// BAD: Creates nested runtime (will panic or be inefficient)
async fn handler(&self) {
    let result = run_async(async {
        self.client.fetch().await
    });
}

// GOOD: Just await directly in async context
async fn handler(&self) {
    let result = self.client.fetch().await;
}
```

---
paths: "src-tauri/src/**/*.rs"
---

# Backend Events Pattern

## Event Name Constants

Define event names as constants in `events.rs` modules:

```rust
// In events.rs
pub mod event_names {
    pub const RECORDING_STARTED: &str = "recording_started";
    pub const RECORDING_STOPPED: &str = "recording_stopped";
    pub const TRANSCRIPTION_COMPLETED: &str = "transcription_completed";
}

// Domain-specific event modules
pub mod hotkey_events {
    pub const KEY_BLOCKING_UNAVAILABLE: &str = "key_blocking_unavailable";
}

pub mod model_events {
    pub const MODEL_DOWNLOAD_COMPLETED: &str = "model_download_completed";
}
```

## Payload Structs

Use `#[serde(rename_all = "camelCase")]` for frontend compatibility:

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RecordingCancelledPayload {
    /// Reason for cancellation
    pub reason: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelFileDownloadProgressPayload {
    pub model_type: String,
    pub file_name: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percent: f64,
}
```

## emit_or_warn! Macro

Use the `emit_or_warn!` macro to emit events with automatic error logging:

```rust
use crate::emit_or_warn;
use crate::events::{event_names, RecordingStartedPayload};

// In commands or handlers
emit_or_warn!(
    app_handle,
    event_names::RECORDING_STARTED,
    RecordingStartedPayload {
        timestamp: crate::events::current_timestamp(),
    }
);
```

The macro logs a warning if emission fails instead of propagating the error:

```rust
// Macro definition in commands/common/mod.rs
#[macro_export]
macro_rules! emit_or_warn {
    ($handle:expr, $event:expr, $payload:expr) => {
        if let Err(e) = $handle.emit($event, $payload) {
            crate::warn!("Failed to emit event '{}': {}", $event, e);
        }
    };
}
```

## Trait-Based Emitters

Use traits for event emission to enable testing without Tauri:

```rust
/// Trait for emitting recording events
pub trait RecordingEventEmitter: Send + Sync {
    fn emit_recording_started(&self, payload: RecordingStartedPayload);
    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload);
    fn emit_recording_cancelled(&self, payload: RecordingCancelledPayload);
    fn emit_recording_error(&self, payload: RecordingErrorPayload);
}

/// Trait for emitting transcription events
pub trait TranscriptionEventEmitter: Send + Sync {
    fn emit_transcription_started(&self, payload: TranscriptionStartedPayload);
    fn emit_transcription_completed(&self, payload: TranscriptionCompletedPayload);
    fn emit_transcription_error(&self, payload: TranscriptionErrorPayload);
}
```

Implementation using `TauriEventEmitter` in `commands/common/emitter.rs`:

```rust
impl RecordingEventEmitter for TauriEventEmitter {
    fn emit_recording_started(&self, payload: RecordingStartedPayload) {
        emit_or_warn!(self.app_handle, event_names::RECORDING_STARTED, payload);
    }
    // ... other methods
}
```

## Timestamp Helper

Use `current_timestamp()` for consistent ISO 8601 timestamps:

```rust
// In events.rs
pub fn current_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

// Usage
RecordingStartedPayload {
    timestamp: crate::events::current_timestamp(),
}
```

## Anti-Patterns

### Hardcoded event names

```rust
// BAD: String literals for event names
app_handle.emit("recording_started", payload);

// GOOD: Use constants
emit_or_warn!(app_handle, event_names::RECORDING_STARTED, payload);
```

### Ignoring emit errors

```rust
// BAD: Silently ignoring errors
let _ = app_handle.emit(event_name, payload);

// GOOD: Use emit_or_warn! to log failures
emit_or_warn!(app_handle, event_name, payload);
```

### Missing serde rename

```rust
// BAD: Snake case goes to frontend as-is
pub struct MyPayload {
    pub file_name: String,  // Serializes as "file_name"
}

// GOOD: camelCase for frontend
#[serde(rename_all = "camelCase")]
pub struct MyPayload {
    pub file_name: String,  // Serializes as "fileName"
}
```

### Direct emission in business logic

```rust
// BAD: Tight coupling to Tauri
fn process_recording(app_handle: &AppHandle) {
    app_handle.emit("done", payload);
}

// GOOD: Use trait-based emitter
fn process_recording<E: RecordingEventEmitter>(emitter: &E) {
    emitter.emit_recording_stopped(payload);
}
```

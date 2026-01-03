---
paths: "src-tauri/src/**/*.rs"
---

# Backend Events

```
                           EVENT EMISSION FLOW
───────────────────────────────────────────────────────────────────────
Domain Logic (RecordingManager, etc.)
       │
       │ calls emitter trait method
       ▼
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│ EventEmitter     │────▶│ TauriEventEmitter│────▶│ emit_or_warn!    │
│ Trait            │     │ (impl)           │     │ macro            │
│                  │     │                  │     │                  │
│ emit_recording_  │     │ app_handle.emit()│     │ logs on failure  │
│ started()        │     │                  │     │                  │
└──────────────────┘     └──────────────────┘     └──────────────────┘
                                                          │
                                                          ▼
                                                  Tauri IPC → Frontend
```

## Event Names

Define as constants in `events.rs`:

```rust
pub mod event_names {
    pub const RECORDING_STARTED: &str = "recording_started";
    pub const RECORDING_STOPPED: &str = "recording_stopped";
}
```

## Payload Structs

Use `#[serde(rename_all = "camelCase")]` for frontend compatibility:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingStoppedPayload {
    pub duration_ms: u64,
    pub timestamp: String,
}
```

## Emission

- Use `emit_or_warn!` macro (logs failure instead of propagating error)
- Use `current_timestamp()` for ISO 8601 timestamps

```rust
emit_or_warn!(app_handle, event_names::RECORDING_STARTED, payload);
```

## Trait-Based Emitters

Use traits for testability (avoid tight coupling to Tauri):

```rust
pub trait RecordingEventEmitter: Send + Sync {
    fn emit_recording_started(&self, payload: RecordingStartedPayload);
    fn emit_recording_stopped(&self, payload: RecordingStoppedPayload);
}
```

Implement with `TauriEventEmitter` in `commands/common/emitter.rs`.

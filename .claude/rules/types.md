---
paths: "src-tauri/src/**/*.rs, src/**/*.ts, src/**/*.tsx"
---

# Cross-Stack Type Contracts

```
                        CROSS-STACK SERIALIZATION
───────────────────────────────────────────────────────────────────────
RUST (Backend)                              TYPESCRIPT (Frontend)

#[serde(rename_all                          interface AudioInputDevice {
  = "camelCase")]          ─────────▶         name: string;
pub struct AudioInputDevice {  JSON           isDefault: boolean;
    pub name: String,                       }
    pub is_default: bool,
}
         │                                           │
         ▼                                           ▼
┌──────────────────┐                      ┌──────────────────┐
│ Command Return   │    ◀── Tauri ──▶     │ invoke<T>()      │
│ Result<T,String> │        IPC           │ typed response   │
└──────────────────┘                      └──────────────────┘
┌──────────────────┐                      ┌──────────────────┐
│ Event Payload    │    ◀── Tauri ──▶     │ listen<T>()      │
│ emit(name,struct)│        IPC           │ typed payload    │
└──────────────────┘                      └──────────────────┘
```

## Rust → TypeScript

All Rust structs sent to frontend use `#[serde(rename_all = "camelCase")]`:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInputDevice {
    pub name: String,
    pub is_default: bool,  // → isDefault
}
```

TypeScript mirrors with camelCase:

```typescript
export interface AudioInputDevice {
  name: string;
  isDefault: boolean;
}
```

## Typed invoke<T>()

Always specify return type:

```typescript
const devices = await invoke<AudioInputDevice[]>("list_audio_devices");
const response = await invoke<PaginatedResponse>("list_recordings", { limit: 20 });
```

## Event Payloads

Export payload types from `eventBridge.ts`:

```typescript
export interface TranscriptionCompletedPayload {
  text: string;
  durationMs: number;
}

// Usage
await listen<TranscriptionCompletedPayload>(eventNames.TRANSCRIPTION_COMPLETED, (event) => {
  const { text, durationMs } = event.payload;
});
```

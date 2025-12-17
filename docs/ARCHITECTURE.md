# heycat Architecture Patterns

IMPORTANT: If you read this file, return this to the user in your response, if reading from a task inform the main agent that the user needs to see this as well: "=== ARCHITECTURE DOCS LOADED ===".

## Overview

heycat is a Tauri v2 desktop app with React+TypeScript frontend and Rust backend. This document captures the key architectural patterns for implementing features.

---

## 1. Frontend-Backend Communication

```
Request:  Frontend hook → invoke(cmd, args) → Tauri IPC → #[tauri::command] handler
Response: Backend handler → app_handle.emit(event, payload) → Tauri IPC → listen() callback
```

| Direction | Frontend | Backend |
|-----------|----------|---------|
| Request | `invoke("start_recording", {deviceName})` | `#[tauri::command] fn start_recording()` |
| Response | `listen("recording_started", callback)` | `app_handle.emit("recording_started", payload)` |

### Commands (Frontend → Backend)

```typescript
// Frontend: src/hooks/useRecording.ts
await invoke("start_recording", { deviceName: "MacBook Pro Mic" });
```

```rust
// Backend: src-tauri/src/commands/mod.rs
#[tauri::command]
fn start_recording(
    state: State<'_, ProductionState>,
    device_name: Option<String>,
) -> Result<(), String>
```

### Events (Backend → Frontend)

```rust
// Backend emits
app_handle.emit("recording_started", RecordingStartedPayload { timestamp });
```

```typescript
// Frontend listens
await listen<RecordingStartedPayload>("recording_started", (event) => {
  setIsRecording(true);
});
```

### Key Insight: Event-Driven UI Updates

**Commands return success/failure, but state changes propagate via events.**

This enables:
- Hotkey-triggered actions to update UI (bypasses command response)
- Multiple components reacting to same state change
- Consistent state across all listeners

---

## 2. State Management

| Layer | Storage | Frontend Access | Backend Access |
|-------|---------|-----------------|----------------|
| **Persistent** | `settings.json` (Tauri plugin-store) | `useSettings()` hook | `app.store("settings.json").get(key)` |
| **Session** | React useState / `Arc<Mutex<T>>` | hooks (`isRecording`, `isListening`, etc.) | `State<'_, T>` |

**Persistent keys:** `listening.enabled`, `listening.autoStartOnLaunch`, `audio.selectedDevice`

**Session state (Backend):** RecordingManager, ListeningManager, AudioThreadHandle, SharedTranscriptionModel

### Frontend State Pattern

```typescript
// Session state in hooks
const [isRecording, setIsRecording] = useState(false);

// Persistent state via useSettings
const { settings } = useSettings();
const deviceName = settings.audio.selectedDevice;
```

### Backend State Pattern

```rust
// Setup in lib.rs
let recording_state = Arc::new(Mutex::new(RecordingManager::new()));
app.manage(recording_state);

// Access in commands
fn start_recording(state: State<'_, ProductionState>) {
    let mut manager = state.lock().unwrap();
    manager.start_recording()?;
}
```

---

## 3. Multiple Entry Points Pattern

**Critical Pattern**: Core functionality can be triggered from multiple paths.

| Entry Point | Context | Path to Core |
|-------------|---------|--------------|
| UI Button | Frontend (has context) | `invoke()` → command → `start_recording_impl` |
| Hotkey | Backend (no frontend context) | handler → `start_recording_impl` (store fallback) |
| Wake Word | Backend (no frontend context) | detector → `start_recording_impl` (store fallback) |

**Rule:** All paths must access same settings → Backend paths use store fallback pattern

### Store Fallback Pattern

```rust
// In commands that can be triggered without frontend context:
let device_name = device_name.or_else(|| {
    app_handle
        .store("settings.json")
        .ok()
        .and_then(|store| store.get("audio.selectedDevice"))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
});
```

---

## 4. Application State Machine

```
IDLE --enable_listening--> LISTENING --wake/hotkey--> RECORDING --stop--> PROCESSING
                               ↑                                              |
                               └──────────── recording_done ──────────────────┘
                               (returns to listening if was listening)
```

Events emitted at each transition for UI sync.

---

## 5. Audio System Architecture

```
Audio Subsystem
├── Listening Pipeline (background)
│   └── wake word detection, VAD, continuous analysis → triggers recording
├── Recording Manager (on-demand)
│   └── audio capture, WAV encoding, file saving, transcription
└── AudioThreadHandle (shared resource, one active at a time)
    ├── start_with_device(), stop()
    └── CPAL Backend (cross-platform)
```

---

## 6. Module Organization

### Frontend

```
src/
├── hooks/       # State & side effects (invoke + listen patterns)
├── components/  # UI components ([Component]/Component.tsx + .css)
└── types/       # Shared type definitions
```

### Backend

```
src-tauri/src/
├── lib.rs        # App setup, command registration
├── commands/     # Tauri IPC handlers (mod.rs + logic.rs pattern)
├── events.rs     # Event types + emitter traits
└── [feature]/    # Feature modules (recording/, listening/, audio/, etc.)
```

> **Pattern:** The `commands/` module uses `mod.rs` + `logic.rs` to separate Tauri-specific wrappers from testable implementation logic.

---

## 7. Checklist for New Features

### Before Implementation

- [ ] Identify all entry points (UI, hotkey, background triggers)
- [ ] Map data flow: Frontend → Command → Backend → Event → Frontend
- [ ] Determine state layer: Persistent (store) vs Session (runtime)
- [ ] Check if feature affects state transitions

### During Implementation

- [ ] Commands: Add store fallback for optional params
- [ ] Events: Define payload types in events.rs + TypeScript
- [ ] Hooks: Subscribe to relevant events, not just command responses
- [ ] State: Use Arc<Mutex<T>> for shared backend state
- [ ] Voice commands: Register in voice_commands/registry.rs if applicable
- [ ] Model events: Subscribe to download progress if user-facing

---


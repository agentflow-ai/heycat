# heycat Architecture Patterns

IMPORTANT: If you read this file, return this to the user in your response, if reading from a task inform the main agent that the user needs to see this as well: "=== ARCHITECTURE DOCS LOADED ===".

## Overview

heycat is a Tauri v2 desktop app with React+TypeScript frontend and Rust backend. This document captures the key architectural patterns for implementing features.

---

## 1. Frontend-Backend Communication

```
┌─────────────────────────────────────────────────────────────────┐
│                    COMMUNICATION PATTERNS                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    FRONTEND (React)                      │    │
│  │                                                          │    │
│  │   Hook calls invoke()          Hook listens to events    │    │
│  │         │                              ▲                 │    │
│  │         │ Request                      │ Broadcast       │    │
│  │         │ (with args)                  │ (all listeners) │    │
│  └─────────┼──────────────────────────────┼─────────────────┘    │
│            │                              │                      │
│            ▼                              │                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    TAURI IPC BRIDGE                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│            │                              ▲                      │
│            ▼                              │                      │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    BACKEND (Rust)                        │    │
│  │                                                          │    │
│  │   #[tauri::command]            app_handle.emit()         │    │
│  │   fn start_recording()         "recording_started"       │    │
│  │         │                              │                 │    │
│  │         └──────────────────────────────┘                 │    │
│  │              Process → Emit Event                        │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

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

```
┌─────────────────────────────────────────────────────────────────┐
│                    STATE LAYERS                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              PERSISTENT (survives restart)               │    │
│  │                                                          │    │
│  │   settings.json (Tauri plugin-store)                     │    │
│  │   ├─ listening.enabled                                   │    │
│  │   ├─ listening.autoStartOnLaunch                         │    │
│  │   └─ audio.selectedDevice                                │    │
│  │                                                          │    │
│  │   Frontend: useSettings() hook                           │    │
│  │   Backend:  app.store("settings.json").get(key)          │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │              SESSION (runtime only)                      │    │
│  │                                                          │    │
│  │   Frontend: React useState in hooks                      │    │
│  │   ├─ isRecording, isListening, isTranscribing            │    │
│  │   ├─ error states                                        │    │
│  │   └─ transient UI state                                  │    │
│  │                                                          │    │
│  │   Backend: Arc<Mutex<T>> managed by Tauri                │    │
│  │   ├─ RecordingManager (state machine)                    │    │
│  │   ├─ ListeningManager (pipeline coordinator)             │    │
│  │   ├─ AudioThreadHandle (audio capture)                   │    │
│  │   └─ SharedTranscriptionModel (3GB model, load once)     │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

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

```
┌─────────────────────────────────────────────────────────────────┐
│                 MULTIPLE ENTRY POINTS                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌────────────┐   ┌────────────┐   ┌────────────┐              │
│   │ UI Button  │   │  Hotkey    │   │ Wake Word  │              │
│   │ (Frontend) │   │ (Backend)  │   │ (Backend)  │              │
│   └─────┬──────┘   └─────┬──────┘   └─────┬──────┘              │
│         │                │                │                      │
│         │ Has frontend   │ No frontend    │ No frontend          │
│         │ context        │ context        │ context              │
│         │                │                │                      │
│         ▼                ▼                ▼                      │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │                  CORE FUNCTIONALITY                      │   │
│   │              (start_recording_impl)                      │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   RULE: All paths must have access to same settings/params      │
│   PATTERN: Backend paths read from store as fallback            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

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
┌─────────────────────────────────────────────────────────────────┐
│                    STATE TRANSITIONS                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│     ┌────────┐                                                   │
│     │  IDLE  │◄─────────────────────────────────────┐           │
│     └───┬────┘                                       │           │
│         │ enable_listening                           │           │
│         ▼                                            │           │
│     ┌─────────────┐                                  │           │
│     │  LISTENING  │◄──────────────────────┐         │           │
│     │ (wake word) │                        │         │           │
│     └──────┬──────┘                        │         │           │
│            │                               │         │           │
│     wake word OR hotkey                    │         │           │
│            │                               │         │           │
│            ▼                               │         │           │
│     ┌─────────────┐              ┌────────┴───────┐ │           │
│     │  RECORDING  │──────────────│ recording_done │─┘           │
│     └──────┬──────┘    stop      │(returns to     │             │
│            │                     │ listening if   │             │
│            │                     │ was listening) │             │
│            ▼                     └────────────────┘             │
│     ┌─────────────────┐                                         │
│     │  TRANSCRIBING   │─────────────────────────────────────────┤
│     └─────────────────┘                                         │
│                                                                  │
│   Events emitted at each transition for UI sync                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. Audio System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    AUDIO SUBSYSTEM                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌───────────────────────┐    ┌───────────────────────┐         │
│  │  LISTENING PIPELINE   │    │  RECORDING MANAGER    │         │
│  │  (Background)         │    │  (On-demand)          │         │
│  ├───────────────────────┤    ├───────────────────────┤         │
│  │ • Wake word detection │    │ • Audio capture       │         │
│  │ • VAD (voice activity)│    │ • WAV encoding        │         │
│  │ • Continuous analysis │    │ • File saving         │         │
│  │ • Triggers recording  │    │ • Transcription       │         │
│  └───────────┬───────────┘    └───────────┬───────────┘         │
│              │                            │                      │
│              └──────────┬─────────────────┘                      │
│                         │                                        │
│                         ▼                                        │
│              ┌─────────────────────┐                             │
│              │  AudioThreadHandle  │                             │
│              │  (Shared resource)  │                             │
│              ├─────────────────────┤                             │
│              │ • start_with_device │                             │
│              │ • stop              │                             │
│              │ • One active at a   │                             │
│              │   time              │                             │
│              └──────────┬──────────┘                             │
│                         │                                        │
│                         ▼                                        │
│              ┌─────────────────────┐                             │
│              │    CPAL Backend     │                             │
│              │ (Cross-platform)    │                             │
│              └─────────────────────┘                             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Module Organization

### Frontend

```
src/
├── hooks/           # State & side effects
│   ├── useRecording.ts      # Recording state + invoke + listen
│   ├── useListening.ts      # Listening state + invoke + listen
│   ├── useSettings.ts       # Persistent settings (store)
│   └── useAudioDevices.ts   # Device enumeration
├── components/      # UI components
│   └── [Component]/
│       ├── Component.tsx
│       └── Component.css
└── types/           # Shared type definitions
```

### Backend

```
src-tauri/src/
├── lib.rs           # App setup, command registration
├── commands/        # Tauri IPC handlers
│   ├── mod.rs       # Command wrappers + event emission
│   └── logic.rs     # Testable implementation (no Tauri deps)
├── events.rs        # Event types + emitter traits
├── recording/       # Recording state machine
├── listening/       # Wake word pipeline
├── audio/           # Audio capture (cpal)
├── hotkey/          # Global hotkey integration
└── parakeet/        # Transcription model
```

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

### Testing

- [ ] Test each entry point independently
- [ ] Test with non-default settings (e.g., non-default audio device)
- [ ] Verify events fire correctly for all paths
- [ ] Check state consistency after rapid operations

---

## 8. Common Pitfalls

| Pitfall | Solution |
|---------|----------|
| Backend path ignores user setting | Use store fallback pattern |
| UI doesn't update from hotkey action | Use events, not command responses |
| Race condition on startup | Backend reads initial state from store |
| Audio device ignored | Pass device through ALL audio paths |
| Shared model not loaded | Check model status before use |

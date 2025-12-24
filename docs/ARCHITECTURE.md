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

### Event Bridge Pattern

Backend events route through a central **Event Bridge** (`src/lib/eventBridge.ts`) that dispatches to appropriate state managers:

```
Backend emit() ──▶ Event Bridge ──┬──▶ Query invalidation (server state)
                                  └──▶ Zustand update (client state)
```

| Event Type | Routing | Example |
|------------|---------|---------|
| Server state change | `queryClient.invalidateQueries()` | `recording_started` → refetch recording state |
| UI state change | `store.setOverlayMode()` | `overlay-mode` → update Zustand directly |

```typescript
// src/lib/eventBridge.ts
export async function setupEventBridge(queryClient: QueryClient, store: AppStore) {
  // Server state → Query invalidation
  await listen('recording_started', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
  });

  // UI state → Zustand update
  await listen('overlay-mode', (event) => {
    store.setOverlayMode(event.payload);
  });
}
```

**Why this matters:** Mutations do NOT invalidate queries in `onSuccess`. The Event Bridge handles ALL cache invalidation, ensuring hotkey-triggered and wake-word-triggered actions update the UI correctly.

---

## 2. State Management

### Dataflow Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                    FRONTEND (React)                                      │
│                                                                                          │
│  ┌─────────────┐      ┌──────────────────────────────────────────────────────────────┐  │
│  │   User      │      │                     React Components                          │  │
│  │  Actions    │      │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │  │
│  │             │      │  │  Dashboard  │  │  Settings   │  │  Recordings List    │   │  │
│  │ • Click     │─────▶│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘   │  │
│  │ • Navigate  │      │         │                │                    │              │  │
│  │ • Input     │      │         ▼                ▼                    ▼              │  │
│  └─────────────┘      │  ┌─────────────────────────────────────────────────────────┐ │  │
│                       │  │                    Hooks Layer                           │ │  │
│                       │  │  useRecordingQuery()  useSettingsStore()  useNavigate() │ │  │
│                       │  └────────┬─────────────────────┬─────────────────┬────────┘ │  │
│                       └───────────┼─────────────────────┼─────────────────┼──────────┘  │
│                                   │                     │                 │             │
│         ┌─────────────────────────┼─────────────────────┼─────────────────┼───────────┐ │
│         ▼                         ▼                     ▼                 ▼           │ │
│  ┌─────────────────┐  ┌─────────────────────┐  ┌─────────────────┐  ┌──────────────┐ │ │
│  │  React Router   │  │   Tanstack Query    │  │     Zustand     │  │    Toast     │ │ │
│  │                 │  │                     │  │                 │  │   Context    │ │ │
│  │ URL ↔ Page      │  │ ┌─────────────────┐ │  │ • settings      │  │              │ │ │
│  │                 │  │ │  Query Cache    │ │  │ • overlayMode   │  │ notifications│ │ │
│  │ /dashboard      │  │ │                 │ │  │ • appStatus     │  │              │ │ │
│  │ /recordings     │  │ │ ['tauri',...]   │ │  │                 │  │              │ │ │
│  │ /settings/*     │  │ └────────┬────────┘ │  └────────┬────────┘  └──────────────┘ │ │
│  │ /commands       │  │          │          │           │                            │ │
│  └─────────────────┘  │          ▼          │           │                            │ │
│                       │  ┌───────────────┐  │           │                            │ │
│                       │  │ queryFn:      │  │           │                            │ │
│                       │  │ invoke(cmd)   │──┼───────────┼────────────────────────────┘ │
│                       │  └───────┬───────┘  │           │                              │
│                       └──────────┼──────────┘           │                              │
│                                  │                      │                              │
│                                  ▼                      │                              │
│                       ┌─────────────────────────────────┼──────────────────────────┐   │
│                       │           Event Bridge          │                          │   │
│                       │                                 │                          │   │
│                       │  listen('recording_started') ───┼──▶ invalidateQueries()   │   │
│                       │  listen('transcription_done') ──┼──▶ invalidateQueries()   │   │
│                       │  listen('overlay-mode') ────────┴──▶ store.setOverlay()    │   │
│                       │                                                            │   │
│                       └────────────────────────────────────────────────────────────┘   │
│                                  ▲                                                     │
└──────────────────────────────────┼─────────────────────────────────────────────────────┘
                                   │
                    ═══════════════╪═══════════════  Tauri IPC Boundary  ════════════════
                                   │
┌──────────────────────────────────┼─────────────────────────────────────────────────────┐
│                                  │           BACKEND (Rust)                            │
│                                  ▼                                                     │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│  │                           #[tauri::command] Handlers                              │ │
│  │                                                                                   │ │
│  │  start_recording()  stop_recording()  list_recordings()  enable_listening()      │ │
│  │         │                  │                 │                   │                │ │
│  └─────────┼──────────────────┼─────────────────┼───────────────────┼────────────────┘ │
│            │                  │                 │                   │                  │
│            ▼                  ▼                 ▼                   ▼                  │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│  │                              Core Business Logic                                  │ │
│  │                                                                                   │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐                   │ │
│  │  │ RecordingManager│  │ ListeningManager│  │ TranscriptionSvc│                   │ │
│  │  │                 │  │                 │  │                 │                   │ │
│  │  │ Arc<Mutex<T>>   │  │ Arc<Mutex<T>>   │  │                 │                   │ │
│  │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘                   │ │
│  │           │                    │                    │                            │ │
│  └───────────┼────────────────────┼────────────────────┼────────────────────────────┘ │
│              │                    │                    │                              │
│              ▼                    ▼                    ▼                              │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│  │                            app_handle.emit()                                      │ │
│  │                                                                                   │ │
│  │   emit("recording_started")  emit("listening_started")  emit("transcription_done")│ │
│  │              │                        │                          │                │ │
│  └──────────────┼────────────────────────┼──────────────────────────┼────────────────┘ │
│                 │                        │                          │                  │
└─────────────────┼────────────────────────┼──────────────────────────┼──────────────────┘
                  │                        │                          │
                  └────────────────────────┴──────────────────────────┘
                                           │
                              Tauri IPC (events to frontend)
                                           │
                                           ▼
                              ┌─────────────────────────┐
                              │      Event Bridge       │
                              │   (receives & routes)   │
                              └─────────────────────────┘
```

### State Layers

| Layer | Storage | Purpose | Examples |
|-------|---------|---------|----------|
| **URL State** | React Router (`src/routes.tsx`) | Navigation, page routing | Current page, route params |
| **Client State** | Zustand (`src/stores/appStore.ts`) | UI state, settings cache | Overlay mode, transcription status |
| **Server State** | Tanstack Query (`src/lib/queryClient.ts`) | Cached backend data | Recording state, recordings list |
| **Persistent** | Tauri Store (`settings.json`) | App settings | `listening.enabled`, `audio.selectedDevice`, `audio.noiseSuppression` |
| **Backend Session** | `Arc<Mutex<T>>` | Runtime state | RecordingManager, ListeningManager |

### Key Principles

1. **Never store server data in Zustand** - Use Tanstack Query for cacheable backend state
2. **Settings dual-write** - Zustand caches for fast reads; Tauri Store persists for backend access
3. **High-frequency state stays local** - Audio levels, download progress use component `useState`
4. **Event Bridge routes updates** - Central hub dispatches backend events to Query or Zustand

### Frontend State Pattern

```typescript
// Server state via Tanstack Query
const { isRecording, isLoading, error } = useRecordingState();

// Client state via Zustand selector
const overlayMode = useOverlayMode();

// Settings via useSettings (Zustand cache + Tauri Store)
const { settings, updateListeningEnabled } = useSettings();

// High-frequency transient state
const [audioLevel, setAudioLevel] = useState(0);
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
├── SharedDenoiser (loaded at startup, Arc<SharedDenoiser>)
│   └── DTLN noise suppression, reused across recordings, reset() between uses
│   └── Controlled by audio.noiseSuppression setting (default: enabled)
├── Listening Pipeline (background)
│   └── wake word detection, VAD, continuous analysis → triggers recording
├── Recording Manager (on-demand)
│   └── audio capture, WAV encoding, file saving, transcription
└── AudioThreadHandle (shared resource, one active at a time)
    ├── start_with_device_and_denoiser(), stop()
    └── CPAL Backend (cross-platform, integrates denoiser in audio callback)
```

---

## 6. Module Organization

### Frontend

```
src/
├── lib/           # Infrastructure
│   ├── queryClient.ts   # Tanstack Query configuration
│   ├── queryKeys.ts     # Centralized query key definitions
│   └── eventBridge.ts   # Backend event → state manager routing
├── stores/        # Zustand stores
│   └── appStore.ts      # Global client state (overlayMode, settings cache)
├── hooks/         # Query/mutation hooks + composite hooks
├── pages/         # Route pages
├── components/    # UI components ([Component]/Component.tsx + .css)
├── routes.tsx     # React Router configuration
└── types/         # Shared type definitions
```

**Provider hierarchy** (`src/App.tsx`):
```
QueryClientProvider → ToastProvider → AppInitializer → RouterProvider
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

### Logging Convention

All Rust modules use the `crate::` prefix pattern for log macros:

```rust
// In any module file (not lib.rs):
crate::info!("Recording started at {}Hz", sample_rate);
crate::debug!("Current state: {:?}", state);
crate::warn!("Device not found, using default");
crate::error!("Failed to start recording: {}", e);
```

**Why this pattern:**
- No imports needed in each file
- Explicit about macro source
- Consistent across all modules
- Macros are re-exported in `lib.rs` from `tauri_plugin_log`

**Note:** The `lib.rs` file (crate root) uses macros directly without `crate::` prefix since they are defined/re-exported there.

---

## 7. Shutdown Coordination

**Important:** Never use `std::process::exit()` from a signal handler - it's not async-signal-safe and causes undefined behavior on macOS.

Use the graceful exit pattern in `src-tauri/src/shutdown.rs`:
- `register_app_handle()` - Store handle at startup
- `request_app_exit()` - Use `AppHandle::exit()` for clean shutdown
- `is_shutting_down()` - Guard operations that shouldn't run during shutdown

---

## 8. Checklist for New Features

### Before Implementation

- [ ] Identify all entry points (UI, hotkey, background triggers)
- [ ] Map data flow: Frontend → Command → Backend → Event → Frontend
- [ ] Determine state layer for each piece of data:
  - **URL** (React Router) - navigation state
  - **Client** (Zustand) - UI state, derived status
  - **Server** (Tanstack Query) - cacheable backend data
  - **Persistent** (Tauri Store) - settings backend needs access to
- [ ] Check if feature affects state transitions

### During Implementation

**Backend:**
- [ ] Commands: Add store fallback for optional params
- [ ] Events: Define payload types in `events.rs` + TypeScript
- [ ] State: Use `Arc<Mutex<T>>` for shared backend state
- [ ] Voice commands: Register in `voice_commands/registry.rs` if applicable

**Frontend:**
- [ ] Server state: Create query/mutation hooks using `queryKeys.ts`
- [ ] Events: Add handling in Event Bridge (invalidation or Zustand update)
- [ ] Settings: Use dual-write pattern (Zustand + Tauri Store)
- [ ] High-frequency data: Use local `useState` (audio levels, progress)
- [ ] Mutations: NO `onSuccess` invalidation - Event Bridge handles it

---


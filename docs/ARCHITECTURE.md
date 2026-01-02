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
| UI state change | `store.setOverlayMode()` | `overlay_mode` → update Zustand directly |

```typescript
// src/lib/eventBridge.ts
export async function setupEventBridge(queryClient: QueryClient, store: AppStore) {
  // Server state → Query invalidation
  await listen('recording_started', () => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState });
  });

  // UI state → Zustand update
  await listen('overlay_mode', (event) => {
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
│                       │  listen('overlay_mode') ────────┴──▶ store.setOverlay()    │   │
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
│  │  start_recording()  stop_recording()  list_recordings()  transcribe()           │ │
│  │         │                  │                 │                   │                │ │
│  └─────────┼──────────────────┼─────────────────┼───────────────────┼────────────────┘ │
│            │                  │                 │                   │                  │
│            ▼                  ▼                 ▼                   ▼                  │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│  │                              Core Business Logic                                  │ │
│  │                                                                                   │ │
│  │  ┌─────────────────┐                         ┌─────────────────┐                  │ │
│  │  │ RecordingManager│                         │ TranscriptionSvc│                  │ │
│  │  │                 │                         │                 │                  │ │
│  │  │ Arc<Mutex<T>>   │                         │                 │                  │ │
│  │  └────────┬────────┘                         └────────┬────────┘                  │ │
│  │           │                                           │                           │ │
│  └───────────┼───────────────────────────────────────────┼───────────────────────────┘ │
│              │                                           │                             │
│              ▼                                           ▼                             │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│  │                            app_handle.emit()                                      │ │
│  │                                                                                   │ │
│  │   emit("recording_started")          emit("transcription_done")                   │ │
│  │              │                                     │                              │ │
│  └──────────────┼─────────────────────────────────────┼──────────────────────────────┘ │
│                 │                                     │                                │
└─────────────────┼─────────────────────────────────────┼──────────────────────────────────┘
                  │                                     │
                  └─────────────────────────────────────┘
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
| **Persistent** | Tauri Store (`settings.json`) | App settings | `audio.selectedDevice` |
| **Backend Session** | `Arc<Mutex<T>>` | Runtime state | RecordingManager |

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
const { settings, updateAudioDevice } = useSettings();

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
IDLE --hotkey/button--> RECORDING --stop--> PROCESSING --done--> IDLE
```

The application uses push-to-talk recording. Events are emitted at each transition for UI sync.

---

## 5. Audio System Architecture

### Overview

heycat uses **native macOS AVFoundation** for audio capture via a Swift bridge. This provides:
- Native 16kHz mono capture (no resampling needed)
- Reliable device enumeration and selection
- Minimal latency and efficient resource usage
- macOS-specific optimizations (AVAudioEngine)

### Architecture

```
Audio Subsystem (macOS-only via AVFoundation)
├── Swift Bridge (src-tauri/swift-lib/Sources/swift-lib/)
│   ├── lib.swift              # FFI entry points (swift-rs integration)
│   ├── SharedAudioEngine.swift # Unified AVAudioEngine for capture + monitoring
│   └── AudioDevices.swift      # Device enumeration via Core Audio
├── Rust FFI Layer (src-tauri/src/swift.rs)
│   └── Safe wrappers around Swift functions
├── Recording Manager (on-demand)
│   └── Audio capture, WAV encoding, file saving, transcription
└── AudioThreadHandle (shared resource)
    └── SwiftBackend (calls Swift FFI for AVFoundation operations)
```

### Swift-Rust FFI Bridge

The audio system uses [swift-rs](https://github.com/nicklockwood/swift-rs) for Swift-Rust interop:

```rust
// src-tauri/src/swift.rs - FFI function declarations
swift_rs::swift!(fn swift_start_audio_capture(device_name: &SRString) -> bool);
swift_rs::swift!(fn swift_stop_audio_capture() -> i64);
swift_rs::swift!(fn swift_get_captured_samples() -> i64);
```

```swift
// src-tauri/swift-lib/Sources/swift-lib/SharedAudioEngine.swift - Native implementation
@_cdecl("swift_start_audio_capture")
public func startAudioCapture(deviceName: SRString?) -> Bool {
    let device = deviceName?.toString()
    return SharedAudioEngine.shared.startCapture(deviceName: device)
}
```

**Build Integration:** The `src-tauri/build.rs` uses `SwiftLinker` to compile Swift sources and link them into the Tauri binary.

### Audio Capture Flow

```
User Action (UI Button / Hotkey)
         │
         ▼
┌─────────────────────────────────────────────────────────────────┐
│  RecordingManager.start_recording()                              │
│    └── AudioThreadHandle.start_with_device()                     │
│          └── SwiftBackend.start()                                │
│                └── swift_start_audio_capture()  ──────────────┐  │
└──────────────────────────────────────────────────────────────┼──┘
                                                               │
                              ┌────────────────────────────────┼──┐
                              │  Swift Layer (AVFoundation)    │  │
                              │                                ▼  │
                              │  ┌──────────────────────────────┐ │
                              │  │ SharedAudioEngine            │ │
                              │  │   └── AVAudioEngine          │ │
                              │  │        └── Input Tap (16kHz) │ │
                              │  │             └── Buffer       │ │
                              │  └──────────────────────────────┘ │
                              └───────────────────────────────────┘
                                                               │
         ┌─────────────────────────────────────────────────────┘
         ▼
RecordingManager.stop_recording()
    └── swift_stop_audio_capture()
          └── Returns captured samples
                └── WAV encoding → Transcription
```

### Device Enumeration

Audio devices are enumerated via AVFoundation's AVCaptureDevice API:

```rust
// List available audio input devices
let devices = swift::list_audio_devices();
// Returns Vec<SwiftAudioDevice> with name and is_default flag
```

### Audio Level Monitoring

Real-time audio levels (0-100) for UI visualization are provided independently of recording:

```rust
use crate::swift::{start_audio_monitor, get_audio_level, stop_audio_monitor, AudioMonitorResult};

// Start monitoring (returns Result-like enum)
match start_audio_monitor(device_name) {
    AudioMonitorResult::Started => {
        // Poll for levels during monitoring
        let level: u8 = get_audio_level();  // 0-100
    }
    AudioMonitorResult::Failed(error) => {
        // Handle error
    }
}

// Stop monitoring
stop_audio_monitor();
```

### Build Requirements

**macOS-only:** The AVFoundation audio backend requires macOS. Build with:
```bash
cargo build --release  # macOS only
```

The Swift sources are compiled automatically by `build.rs` using `SwiftLinker`.

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
src-tauri/
├── swift-lib/                # Swift package (AVFoundation audio)
│   ├── Package.swift         # Swift package manifest
│   └── Sources/swift-lib/
│       ├── lib.swift              # FFI entry points
│       ├── SharedAudioEngine.swift # Unified capture + monitoring via AVAudioEngine
│       └── AudioDevices.swift     # Device enumeration via Core Audio
├── src/
│   ├── lib.rs        # App setup, command registration
│   ├── swift.rs      # Safe Rust wrappers for Swift FFI
│   ├── commands/     # Tauri IPC handlers (mod.rs + logic.rs pattern)
│   ├── events.rs     # Event types + emitter traits
│   └── [feature]/    # Feature modules (recording/, audio/, transcription/, etc.)
└── build.rs          # SwiftLinker integration for Swift compilation
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


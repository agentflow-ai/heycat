---
last-updated: 2025-12-20
status: active
---

# Technical Guidance: Dataflow Refactor

## Architecture Overview

### Current State

The frontend currently uses a minimal architecture:
- **No routing library** - Manual `useState`-based page switching in App.tsx
- **No global state library** - Only React Context for toasts
- **No data fetching library** - Direct `invoke()` calls with manual loading states
- **Event-driven updates** - Backend emits events, frontend listens via `listen()`

**Existing Patterns:**
```
Frontend Hook → invoke(cmd, args) → Tauri IPC → #[tauri::command]
                    ↓
         listen("event") ←── app_handle.emit("event", payload)
```

### Target State

```
┌─────────────────────────────────────────────────────────────────┐
│                        React Application                        │
├─────────────────────────────────────────────────────────────────┤
│  React Router (URL-based navigation)                            │
│    └── Routes: /, /commands, /recordings, /settings/*           │
├─────────────────────────────────────────────────────────────────┤
│  Zustand Store (Global UI/App State)                            │
│    └── App status, overlay visibility, settings cache           │
├─────────────────────────────────────────────────────────────────┤
│  Tanstack Query (Server State from Tauri)                       │
│    ├── Queries: list_recordings, get_recording_state, etc.      │
│    ├── Mutations: start_recording, stop_recording, etc.         │
│    └── Subscriptions: Event listeners (recording_started, etc.) │
├─────────────────────────────────────────────────────────────────┤
│  Tauri IPC Layer (invoke + listen)                              │
└─────────────────────────────────────────────────────────────────┘
```

### Dataflow Diagram

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

### Data Flow Scenarios

**Scenario 1: User Clicks "Start Recording"**
```
Component ──▶ useMutation() ──▶ invoke('start_recording') ──▶ Rust Handler
                                                                    │
                                                          emit('recording_started')
                                                                    │
                                                                    ▼
Component ◀── re-render ◀── Query Cache ◀── invalidate ◀── Event Bridge
```

**Scenario 2: Hotkey Triggers Recording (Backend-Initiated)**
```
Global Hotkey ──▶ Rust Hotkey Handler ──▶ start_recording_impl()
                                                    │
                                          emit('recording_started')
                                                    │
                                                    ▼
Component ◀── re-render ◀── Query Cache ◀── invalidate ◀── Event Bridge
```

**Scenario 3: Navigate to Settings Page**
```
User Click ──▶ navigate('/settings') ──▶ React Router ──▶ <Settings />
                                                              │
                                              useSettingsStore() + useQuery()
                                                              │
                                                              ▼
                                              Render with cached/fresh data
```

### Layer Responsibilities

| Layer | Responsibility | State Type |
|-------|---------------|------------|
| **React Router** | URL ↔ Page mapping, navigation | URL state |
| **Zustand** | Global UI state, derived status, settings cache | Client state |
| **Tanstack Query** | Tauri command caching, loading/error states, mutations | Server state |
| **Tauri Events** | Real-time updates pushed from backend | Push updates |

### Integration Points

1. **Tauri Commands → Tanstack Query**
   - Wrap `invoke()` in query functions
   - Use mutation hooks for state-changing commands
   - Invalidate queries on relevant events

2. **Tauri Events → Query Invalidation**
   - `recording_started` → invalidate recording state queries
   - `transcription_completed` → invalidate recordings list
   - Events can also update Zustand store directly for UI state

3. **Settings → Zustand + Tanstack Query**
   - Settings loaded into Zustand on app init
   - Settings mutations update both store file and Zustand

### Constraints

- Must maintain backward compatibility during incremental migration
- Cannot break hotkey-triggered flows (backend-initiated events)
- Must preserve real-time audio level updates (~20fps)
- Type safety required at all boundaries

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Zustand over Redux | Simpler API, less boilerplate, good for small-to-medium apps | 2025-12-20 |
| Tanstack Query over SWR | Better devtools, mutation support, query invalidation API | 2025-12-20 |
| React Router v6 | Standard routing solution, supports nested routes, type-safe | 2025-12-20 |
| Incremental migration | Lower risk, can validate patterns before full migration | 2025-12-20 |
| Keep Tauri Store for settings | Backend needs direct access for hotkey-triggered flows; Zustand caches in-memory for frontend | 2025-12-20 |
| Backend is source of truth | Events always override optimistic updates; rollback if event differs from expectation | 2025-12-20 |
| Central event bridge | Single file subscribes to all Tauri events, dispatches to QueryClient.invalidate() or Zustand.set() | 2025-12-20 |
| Command-based query keys | `['tauri', 'list_recordings']` - Clear traceability; future APIs (REST) get own prefix | 2025-12-20 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-20 | No existing router - manual useState in App.tsx | Must implement from scratch |
| 2025-12-20 | 12 custom hooks with direct invoke/listen patterns | Each needs migration to query hooks |
| 2025-12-20 | Event-driven state updates (commands don't return state) | Queries must subscribe to events for invalidation |
| 2025-12-20 | Toast context is only global state | Minimal refactoring needed there |
| 2025-12-20 | No request deduplication currently | Tanstack Query will handle this |

## Open Questions

- [x] Should settings remain in Tauri Store or move to Zustand-persist? → **Keep Tauri Store** (backend access)
- [x] How to handle optimistic updates with backend events that may override? → **Backend is truth** (events override)
- [x] Should event subscriptions live in query hooks or a separate system? → **Central event bridge**
- [x] What naming convention for query keys? → **Command-based**: `['tauri', 'command_name']`

## Files to Modify

### New Files to Create
- `src/lib/queryClient.ts` - Tanstack Query client configuration
- `src/lib/queryKeys.ts` - Centralized query key definitions
- `src/lib/eventBridge.ts` - Central Tauri event → Query/Zustand dispatcher
- `src/stores/appStore.ts` - Zustand store for global state
- `src/routes.tsx` - React Router configuration

### Files to Migrate
- `src/App.tsx` - Add providers, replace useState routing
- `src/hooks/useRecording.ts` - Convert to query/mutation hooks
- `src/hooks/useListening.ts` - Convert to query/mutation hooks
- `src/hooks/useSettings.ts` - Integrate with Zustand
- `src/hooks/useAudioDevices.ts` - Convert polling to query with refetch
- `src/hooks/useTranscription.ts` - Convert to event subscription pattern
- `src/hooks/useMultiModelStatus.ts` - Convert to queries
- `src/pages/*.tsx` - Use new hooks, update navigation

### Files to Keep (minimal changes)
- `src/hooks/useAudioLevelMonitor.ts` - Real-time events, no query needed
- `src/hooks/useAppStatus.ts` - May derive from Zustand instead
- `src/components/overlays/toast/*` - Already uses Context, keep as-is

## Key Patterns

### State Separation (from research)

Never store server data in Zustand - this is the core principle:

| State Type | Location | Examples |
|------------|----------|----------|
| **Server state** | Tanstack Query | Recordings list, recording state, model status |
| **Client state** | Zustand | Overlay visibility, current tab, settings cache |
| **URL state** | React Router | Current page, route params |
| **Real-time state** | Local useState | Audio levels (20fps too fast for store) |

### Event Bridge Pattern

```typescript
// src/lib/eventBridge.ts
export function setupEventBridge(queryClient: QueryClient, store: AppStore) {
  // Server state events → Query invalidation
  listen('recording_started', () => {
    queryClient.invalidateQueries({ queryKey: ['tauri', 'get_recording_state'] });
  });

  listen('transcription_completed', () => {
    queryClient.invalidateQueries({ queryKey: ['tauri', 'list_recordings'] });
  });

  // UI state events → Zustand updates
  listen('overlay-mode', (e) => {
    store.setOverlayMode(e.payload);
  });
}
```

### Query Key Convention

```typescript
// src/lib/queryKeys.ts
export const queryKeys = {
  tauri: {
    listRecordings: ['tauri', 'list_recordings'] as const,
    getRecordingState: ['tauri', 'get_recording_state'] as const,
    listAudioDevices: ['tauri', 'list_audio_devices'] as const,
    getListeningStatus: ['tauri', 'get_listening_status'] as const,
    checkModelStatus: (type: string) => ['tauri', 'check_parakeet_model_status', type] as const,
  },
} as const;
```

## References

- [Tanstack Query with Custom Functions](https://tanstack.com/query/latest/docs/framework/react/guides/query-functions)
- [Zustand Documentation](https://docs.pmnd.rs/zustand/getting-started/introduction)
- [React Router v6](https://reactrouter.com/en/main)
- [Tauri invoke API](https://v2.tauri.app/reference/javascript/api/namespacetauri/#invoke)
- [Federated State: Zustand + TanStack Query Patterns](https://dev.to/martinrojas/federated-state-done-right-zustand-tanstack-query-and-the-patterns-that-actually-work-27c0)
- [Separating Concerns with Zustand and TanStack Query](https://volodymyrrudyi.com/blog/separating-concerns-with-zustand-and-tanstack-query/)

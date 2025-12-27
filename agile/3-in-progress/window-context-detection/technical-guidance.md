---
last-updated: 2025-12-24
status: draft
---

# Technical Guidance: Window Context Detection for Context-Sensitive Commands

## Architecture Overview

This feature adds application-aware voice command and dictionary customization by detecting the active window and applying context-specific configurations. It follows established patterns from `docs/ARCHITECTURE.md`.

### Integration Points

| Layer | Component | Integration |
|-------|-----------|-------------|
| Backend | `window_context/` module | New Rust module following `voice_commands/` pattern |
| Backend | `TranscriptionService` | Inject `ContextResolver` for command resolution |
| Backend | App initialization (`lib.rs`) | Initialize store, monitor, wire to services |
| Frontend | Settings pages | New "Window Contexts" section |
| Frontend | Event Bridge | Handle `active_window_changed`, `window_contexts_updated` |

---

## Complete Data Flow

### DF-1: Window Context CRUD Flow

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (React)                                    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │                     WindowContexts Page (Settings)                          ││
│  │                                                                             ││
│  │  [New Context] ──▶ Modal ──▶ useWindowContext.addContext()                  ││
│  │  [Edit]        ──▶ Modal ──▶ useWindowContext.updateContext()               ││
│  │  [Delete]      ──▶ Confirm ──▶ useWindowContext.deleteContext()             ││
│  │                                                                             ││
│  └────────────────────────────────┬────────────────────────────────────────────┘│
│                                   │                                              │
│                                   ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │                    useWindowContext Hook (TanStack Query)                   ││
│  │                                                                             ││
│  │  contexts = useQuery(['windowContext', 'list'], invoke('list_window_...'))  ││
│  │  addContext = useMutation(invoke('add_window_context', input))              ││
│  │  updateContext = useMutation(invoke('update_window_context', input))        ││
│  │  deleteContext = useMutation(invoke('delete_window_context', id))           ││
│  │                                                                             ││
│  │  NOTE: NO onSuccess invalidation - Event Bridge handles cache updates       ││
│  │                                                                             ││
│  └────────────────────────────────┬────────────────────────────────────────────┘│
│                                   │                                              │
└───────────────────────────────────┼──────────────────────────────────────────────┘
                                    │
                     ═══════════════╪═══════════════  Tauri IPC  ════════════════
                                    │
┌───────────────────────────────────┼──────────────────────────────────────────────┐
│                                   ▼              BACKEND (Rust)                  │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │                     #[tauri::command] Handlers                              ││
│  │                     (src-tauri/src/commands/window_context.rs)              ││
│  │                                                                             ││
│  │  list_window_contexts(state: State<WindowContextStoreState>)                ││
│  │  add_window_context(input, state) ──▶ emit("window_contexts_updated")       ││
│  │  update_window_context(input, state) ──▶ emit("window_contexts_updated")    ││
│  │  delete_window_context(id, state) ──▶ emit("window_contexts_updated")       ││
│  │                                                                             ││
│  └────────────────────────────────┬────────────────────────────────────────────┘│
│                                   │                                              │
│                                   ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────────────┐│
│  │                         WindowContextStore                                  ││
│  │                    (src-tauri/src/window_context/store.rs)                  ││
│  │                                                                             ││
│  │  contexts: HashMap<Uuid, WindowContext>                                     ││
│  │  config_path: PathBuf (~/.config/heycat/window_contexts.json)               ││
│  │                                                                             ││
│  │  load() ──▶ read JSON ──▶ parse ──▶ populate HashMap                        ││
│  │  save() ──▶ serialize ──▶ write temp ──▶ atomic rename                      ││
│  │                                                                             ││
│  └─────────────────────────────────────────────────────────────────────────────┘│
│                                                                                  │
└──────────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                     ┌──────────────────────────────┐
                     │     window_contexts.json     │
                     │  ~/.config/heycat/           │
                     └──────────────────────────────┘
```

**Reference:** See `docs/ARCHITECTURE.md` Section 1 (Frontend-Backend Communication) for invoke/emit patterns.

---

### DF-2: Active Window Monitoring Flow

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                              BACKEND (Rust)                                       │
│                                                                                   │
│  ┌────────────────────────────────────────────────────────────────────────────┐  │
│  │                        WindowMonitor (Background Thread)                    │  │
│  │                      (src-tauri/src/window_context/monitor.rs)              │  │
│  │                                                                             │  │
│  │  ┌─────────────────────────────────────────────────────────────────────┐   │  │
│  │  │                     Polling Loop (~200ms)                            │   │  │
│  │  │                                                                      │   │  │
│  │  │  loop {                                                              │   │  │
│  │  │    1. get_active_window() ────────────────────────────────────────┐  │   │  │
│  │  │    2. compare with last_window                                    │  │   │  │
│  │  │    3. if changed:                                                 │  │   │  │
│  │  │       - find_matching_context() ──────────────────────────────┐   │  │   │  │
│  │  │       - update current_context                                │   │  │   │  │
│  │  │       - emit("active_window_changed", payload) ───────────────┼───┼───┼───┼──▶ Frontend
│  │  │    4. sleep(200ms)                                            │   │  │   │  │
│  │  │  }                                                            │   │  │   │  │
│  │  │                                                               │   │  │   │  │
│  │  └───────────────────────────────────────────────────────────────┼───┼───┘   │  │
│  │                                                                  │   │       │  │
│  └──────────────────────────────────────────────────────────────────┼───┼───────┘  │
│                                                                     │   │          │
│                                                                     ▼   ▼          │
│  ┌─────────────────────────────────────┐  ┌─────────────────────────────────────┐ │
│  │         ActiveWindowDetector        │  │         WindowContextStore          │ │
│  │  (src-tauri/src/window_context/     │  │                                     │ │
│  │         detector.rs)                │  │  find_matching_context(window):     │ │
│  │                                     │  │    1. Filter enabled contexts       │ │
│  │  get_active_window():               │  │    2. Match app_name (case-insens)  │ │
│  │    1. NSWorkspace.frontmostApp      │  │    3. Match title_pattern (regex)   │ │
│  │    2. CGWindowListCopyWindowInfo    │  │    4. Sort by priority (desc)       │ │
│  │    3. Return ActiveWindowInfo       │  │    5. Return highest match or None  │ │
│  │       - app_name                    │  │                                     │ │
│  │       - bundle_id                   │  └─────────────────────────────────────┘ │
│  │       - window_title                │                                          │
│  │       - pid                         │                                          │
│  │                                     │                                          │
│  └─────────────────────────────────────┘                                          │
│                                                                                   │
└───────────────────────────────────────────────────────────────────────────────────┘
                                    │
                     ═══════════════╪═══════════════  Tauri IPC (Event)  ═══════════
                                    │
                                    ▼
┌───────────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (React)                                      │
│                                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────────────┐ │
│  │                            Event Bridge                                       │ │
│  │                        (src/lib/eventBridge.ts)                               │ │
│  │                                                                               │ │
│  │  listen('active_window_changed', (event) => {                                 │ │
│  │    store.setActiveWindow(event.payload);                                      │ │
│  │    store.setMatchedContextId(event.payload.matchedContextId);                 │ │
│  │  });                                                                          │ │
│  │                                                                               │ │
│  │  listen('window_contexts_updated', () => {                                    │ │
│  │    queryClient.invalidateQueries({ queryKey: queryKeys.windowContext.all });  │ │
│  │  });                                                                          │ │
│  │                                                                               │ │
│  └──────────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                               │
│                                    ▼                                               │
│  ┌──────────────────────────────────────────────────────────────────────────────┐ │
│  │                     useActiveWindow Hook + Zustand                            │ │
│  │                                                                               │ │
│  │  // Real-time active window state (client state - Zustand)                    │ │
│  │  const { activeWindow, matchedContextId } = useActiveWindow();                │ │
│  │                                                                               │ │
│  │  // Display in status bar: "Context: Slack" or "Context: None"                │ │
│  │                                                                               │ │
│  └──────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                    │
└────────────────────────────────────────────────────────────────────────────────────┘
```

**Reference:** See `docs/ARCHITECTURE.md` Section 2 (State Management) for Event Bridge pattern.

---

### DF-3: Context-Aware Command Execution Flow (CRITICAL PATH)

```
┌────────────────────────────────────────────────────────────────────────────────────┐
│                            USER INTERACTION                                         │
│                                                                                     │
│   User speaks: "send message"                                                       │
│   Active window: Slack                                                              │
│   Matched context: "Slack" (Replace mode, has "send message" → custom action)       │
│                                                                                     │
└─────────────────────────────────────┬───────────────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────────────┐
│                         ENTRY POINTS (Multiple)                                     │
│                                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                              │
│  │  UI Button   │  │   Hotkey     │  │  Wake Word   │                              │
│  │  (Frontend)  │  │  (Backend)   │  │  (Backend)   │                              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                              │
│         │                 │                 │                                       │
│         └─────────────────┴─────────────────┘                                       │
│                           │                                                         │
│                           ▼                                                         │
│              ┌────────────────────────┐                                             │
│              │   stop_recording()     │                                             │
│              │   (Converged path)     │                                             │
│              └───────────┬────────────┘                                             │
│                          │                                                          │
└──────────────────────────┼──────────────────────────────────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────────────────────────────────┐
│                              BACKEND (Rust)                                         │
│                                                                                     │
│  ┌────────────────────────────────────────────────────────────────────────────┐    │
│  │                    RecordingTranscriptionService                            │    │
│  │                 (src-tauri/src/transcription/service.rs)                    │    │
│  │                                                                             │    │
│  │  process_recording(file_path):                                              │    │
│  │    1. Transcribe audio ──▶ "send message"                                   │    │
│  │    2. Apply dictionary expansion (context-aware) ◄─────────────────────┐    │    │
│  │    3. try_command_matching() ◄─────────────────────────────────────────┤    │    │
│  │                                                                        │    │    │
│  └────────────────────────────────────────────────────┬───────────────────┼────┘    │
│                                                       │                   │         │
│                                                       ▼                   │         │
│  ┌────────────────────────────────────────────────────────────────────────┼────┐    │
│  │                         ContextResolver                                │    │    │
│  │                  (src-tauri/src/window_context/resolver.rs)            │    │    │
│  │                                                                        │    │    │
│  │  ┌───────────────────────────────────────────────────────────────────┐ │    │    │
│  │  │  get_effective_commands(global_registry):                         │ │    │    │
│  │  │                                                                   │ │    │    │
│  │  │    1. Get current context from WindowMonitor ─────────────────────┼─┘    │    │
│  │  │       current_context_id = monitor.get_current_context()          │      │    │
│  │  │                                                                   │      │    │
│  │  │    2. If no context: return global_registry.all_commands()        │      │    │
│  │  │                                                                   │      │    │
│  │  │    3. Look up context in store                                    │      │    │
│  │  │       context = store.get(current_context_id)                     │      │    │
│  │  │                                                                   │      │    │
│  │  │    4. Apply mode logic:                                           │      │    │
│  │  │       ┌─────────────────────────────────────────────────────────┐ │      │    │
│  │  │       │  if context.command_mode == Replace:                    │ │      │    │
│  │  │       │    return context_commands_only                         │ │      │    │
│  │  │       │                                                         │ │      │    │
│  │  │       │  if context.command_mode == Merge:                      │ │      │    │
│  │  │       │    merged = global_commands.clone()                     │ │      │    │
│  │  │       │    for cmd in context_commands:                         │ │      │    │
│  │  │       │      merged.insert_or_replace(cmd)  // context wins     │ │      │    │
│  │  │       │    return merged                                        │ │      │    │
│  │  │       └─────────────────────────────────────────────────────────┘ │      │    │
│  │  │                                                                   │      │    │
│  │  └───────────────────────────────────────────────────────────────────┘      │    │
│  │                                                                             │    │
│  │  get_effective_dictionary(global_store):                                    │    │
│  │    (Same logic as above for dictionary entries)  ───────────────────────────┘    │
│  │                                                                                  │
│  └──────────────────────────────────────────────────────────────────────────────────┘
│                                                       │                              │
│                                                       ▼                              │
│  ┌──────────────────────────────────────────────────────────────────────────────┐   │
│  │                           CommandMatcher                                      │   │
│  │                   (src-tauri/src/voice_commands/matcher.rs)                   │   │
│  │                                                                               │   │
│  │  Input: "send message"                                                        │   │
│  │  Commands: [context-specific "send message" → Slack action]                   │   │
│  │  Result: Exact match ──▶ Execute Slack-specific action                        │   │
│  │                                                                               │   │
│  └──────────────────────────────────────────────────────────────────────────────┘   │
│                                                       │                              │
│                                                       ▼                              │
│  ┌──────────────────────────────────────────────────────────────────────────────┐   │
│  │                          ActionDispatcher                                     │   │
│  │                   (src-tauri/src/voice_commands/executor.rs)                  │   │
│  │                                                                               │   │
│  │  execute(command) ──▶ emit("command_executed", payload)                       │   │
│  │                                                                               │   │
│  └──────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                      │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

**Reference:** See `docs/ARCHITECTURE.md` Section 3 (Multiple Entry Points Pattern) - all paths converge on same implementation.

---

### DF-4: App Initialization Flow

```
┌────────────────────────────────────────────────────────────────────────────────────┐
│                          App Startup (src-tauri/src/lib.rs)                         │
│                                                                                     │
│  setup() {                                                                          │
│    // ... existing initialization ...                                               │
│                                                                                     │
│    // 1. Initialize WindowContextStore                                              │
│    let window_context_store = {                                                     │
│      let mut store = WindowContextStore::with_default_path(worktree_ctx)?;          │
│      store.load()?;                                                                 │
│      Arc::new(Mutex::new(store))                                                    │
│    };                                                                               │
│    app.manage(window_context_store.clone());                                        │
│                                                                                     │
│    // 2. Start WindowMonitor (background thread)                                    │
│    let window_monitor = Arc::new(Mutex::new(WindowMonitor::new()));                 │
│    {                                                                                │
│      let mut monitor = window_monitor.lock().unwrap();                              │
│      monitor.start(                                                                 │
│        app.handle().clone(),                                                        │
│        window_context_store.clone()                                                 │
│      );                                                                             │
│    }                                                                                │
│    app.manage(window_monitor.clone());                                              │
│                                                                                     │
│    // 3. Create ContextResolver                                                     │
│    let context_resolver = Arc::new(ContextResolver::new(                            │
│      window_monitor.clone(),                                                        │
│      window_context_store.clone()                                                   │
│    ));                                                                              │
│                                                                                     │
│    // 4. Wire to TranscriptionService                                               │
│    transcription_service = transcription_service                                    │
│      .with_context_resolver(context_resolver);                                      │
│                                                                                     │
│    // 5. Register commands                                                          │
│    // (window context commands added to invoke_handler)                             │
│  }                                                                                  │
│                                                                                     │
└────────────────────────────────────────────────────────────────────────────────────┘
```

**Reference:** See `docs/ARCHITECTURE.md` Section 5 (Audio System Architecture) for similar initialization patterns.

---

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Polling-based window detection (~200ms) | Simpler than event-based; macOS doesn't have reliable focus-change notifications across all apps | 2025-12-23 |
| Priority-based context matching | Higher priority wins when multiple contexts match; avoids user disambiguation prompts | 2025-12-23 |
| Merge/Replace modes per-context | Flexibility: some apps need full replacement, others just additions | 2025-12-23 |
| Context stores command IDs, not copies | Commands remain in global registry; contexts reference by ID for single source of truth | 2025-12-23 |
| Graceful fallback on detection failure | Window detection errors should not block voice commands; fall back to global | 2025-12-23 |

---

## Data Structures

### Backend (Rust)

```rust
// src-tauri/src/window_context/types.rs

/// Information about the currently active window
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActiveWindowInfo {
    pub app_name: String,
    pub bundle_id: Option<String>,
    pub window_title: Option<String>,
    pub pid: u32,
}

/// Pattern for matching windows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowMatcher {
    pub app_name: String,
    pub title_pattern: Option<String>,  // Regex pattern
    pub bundle_id: Option<String>,
}

/// Override behavior for commands/dictionary
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OverrideMode {
    #[default]
    Merge,
    Replace,
}

/// A window context definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowContext {
    pub id: Uuid,
    pub name: String,
    pub matcher: WindowMatcher,
    pub command_mode: OverrideMode,
    pub dictionary_mode: OverrideMode,
    pub command_ids: Vec<Uuid>,
    pub dictionary_entry_ids: Vec<String>,
    pub enabled: bool,
    pub priority: i32,
}
```

### Frontend (TypeScript)

```typescript
// src/types/windowContext.ts

export interface ActiveWindowInfo {
  appName: string;
  bundleId?: string;
  windowTitle?: string;
  pid: number;
}

export interface WindowMatcher {
  appName: string;
  titlePattern?: string;
  bundleId?: string;
}

export type OverrideMode = "merge" | "replace";

export interface WindowContext {
  id: string;
  name: string;
  matcher: WindowMatcher;
  commandMode: OverrideMode;
  dictionaryMode: OverrideMode;
  commandIds: string[];
  dictionaryEntryIds: string[];
  enabled: boolean;
  priority: number;
}

export interface ActiveWindowChangedPayload {
  appName: string;
  bundleId?: string;
  windowTitle?: string;
  matchedContextId?: string;
  matchedContextName?: string;
}
```

---

## Files to Modify

### New Files (Backend)

| File | Purpose | Spec |
|------|---------|------|
| `src-tauri/src/window_context/mod.rs` | Module exports | window-context-types |
| `src-tauri/src/window_context/types.rs` | Data structures | window-context-types |
| `src-tauri/src/window_context/detector.rs` | macOS window detection | active-window-detector |
| `src-tauri/src/window_context/store.rs` | CRUD + persistence | window-context-store |
| `src-tauri/src/window_context/monitor.rs` | Background polling | window-monitor |
| `src-tauri/src/window_context/resolver.rs` | Merge/replace logic | context-resolver |
| `src-tauri/src/commands/window_context.rs` | Tauri commands | window-context-store |

### Modified Files (Backend)

| File | Changes | Spec |
|------|---------|------|
| `src-tauri/src/lib.rs` | Initialize store, monitor, resolver; wire to service | transcription-integration |
| `src-tauri/src/commands/mod.rs` | Export window context commands | window-context-store |
| `src-tauri/src/transcription/service.rs` | Add context_resolver, modify matching | transcription-integration |
| `src-tauri/src/events.rs` | Add window context event types | window-monitor |

### New Files (Frontend)

| File | Purpose | Spec |
|------|---------|------|
| `src/types/windowContext.ts` | TypeScript interfaces | window-context-types |
| `src/hooks/useWindowContext.ts` | CRUD hook | window-contexts-ui |
| `src/hooks/useActiveWindow.ts` | Real-time window state | window-contexts-ui |
| `src/pages/WindowContexts.tsx` | Management page | window-contexts-ui |

### Modified Files (Frontend)

| File | Changes | Spec |
|------|---------|------|
| `src/lib/queryKeys.ts` | Add windowContext keys | window-contexts-ui |
| `src/lib/eventBridge.ts` | Handle window context events | window-contexts-ui |
| `src/routes.tsx` | Add /contexts route | window-contexts-ui |

---

## Open Questions

- [x] How should window matching work? → Both app name AND optional title patterns
- [x] Should context-specific commands supplement or replace global? → Configurable per-context
- [x] When to detect active window? → Continuous monitoring (~200ms polling)
- [ ] Should we show active context indicator in the main UI? → Likely yes, in status bar

---

## References

- `docs/ARCHITECTURE.md` - Core patterns (Event Bridge, Multiple Entry Points, State Management)
- `src-tauri/src/dictionary/store.rs` - Pattern for JSON persistence with atomic writes
- `src-tauri/src/voice_commands/registry.rs` - Pattern for command storage
- `src-tauri/src/listening/pipeline.rs` - Pattern for background thread with polling
- `src/hooks/useDictionary.ts` - Pattern for TanStack Query CRUD hooks
- `src/pages/Dictionary.tsx` - Pattern for Settings management page

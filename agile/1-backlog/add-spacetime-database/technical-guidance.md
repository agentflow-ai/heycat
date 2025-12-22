---
last-updated: 2025-12-22
status: active
---

# Technical Guidance: Add Spacetime Database

## Architecture Overview

### High-Level Design

SpacetimeDB will be integrated as a **sidecar process** that runs alongside heycat, providing persistent data storage with real-time sync capabilities. The architecture follows this pattern:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              heycat (Tauri App)                             │
│                                                                             │
│  ┌─────────────────┐      ┌─────────────────────────────────────────────┐  │
│  │  React Frontend │ ←──→ │             Rust Backend                    │  │
│  │                 │ IPC  │                                             │  │
│  │  • Queries via  │      │  ┌─────────────────────────────────────┐   │  │
│  │    Tauri invoke │      │  │   SpacetimeDB Client (SDK)          │   │  │
│  │  • Real-time    │      │  │                                     │   │  │
│  │    updates via  │      │  │   • DbConnection (WebSocket)        │   │  │
│  │    events       │      │  │   • Subscription cache              │   │  │
│  └────────┬────────┘      │  │   • Reducer invocations             │   │  │
│           │               │  └─────────────┬───────────────────────┘   │  │
│           │               │                │ WebSocket (localhost)     │  │
│           │               └────────────────┼───────────────────────────┘  │
│           │                                │                               │
└───────────┼────────────────────────────────┼───────────────────────────────┘
            │                                │
            │ Tauri Events                   │
            │ (state sync)                   ▼
            │                    ┌─────────────────────────┐
            │                    │  SpacetimeDB Server     │
            │                    │  (Sidecar Process)      │
            │                    │                         │
            │                    │  • Module: heycat-data  │
            │                    │  • Tables & Reducers    │
            │                    │  • WAL persistence      │
            └────────────────────│  • Port: 3000 (local)   │
                                 └─────────────────────────┘
```

### Layers Involved

| Layer | Current State | SpacetimeDB Integration |
|-------|---------------|-------------------------|
| **Frontend (React)** | Uses Tanstack Query for Tauri commands | No direct changes; continues using Tauri commands |
| **Backend (Rust)** | Uses Tauri Store (settings.json), file-based storage | New SpacetimeDB client module, commands wrap SDK |
| **Persistence** | JSON files, WAV files | SpacetimeDB tables + WAL, WAV files remain on disk |
| **Event System** | `app_handle.emit()` to frontend | SDK subscriptions trigger emit() calls |

### Integration Pattern

SpacetimeDB integrates at the **Rust backend layer** only. The frontend remains unchanged:

1. **Frontend** continues calling `invoke("list_recordings")`, `invoke("get_settings")`, etc.
2. **Tauri commands** (in `commands/`) delegate to SpacetimeDB client instead of file I/O
3. **SpacetimeDB SDK** manages connection, caching, and subscriptions
4. **Event Bridge** receives SpacetimeDB change notifications and emits to frontend

This preserves the existing architecture pattern: **Commands return data, Events push state changes**.

### Data Model

SpacetimeDB module (`heycat-data`) will define these tables:

| Table | Primary Key | Fields | Notes |
|-------|-------------|--------|-------|
| `recordings` | `id: u64` (auto-inc) | file_path, duration_ms, sample_rate, created_at, transcription, transcription_model | Audio files remain on disk; metadata only |
| `settings` | `key: String` | value (JSON string), updated_at | Replaces settings.json |
| `dictionary_entries` | `id: u64` (auto-inc) | trigger, replacement, enabled | Replaces dictionary.json |
| `voice_commands` | `id: u64` (auto-inc) | phrase, action_type, action_payload (JSON), enabled | User-defined voice commands |

### Sidecar Management

SpacetimeDB runs as a Tauri sidecar, managed in `lib.rs`:

```rust
// In setup(), spawn SpacetimeDB sidecar
let (mut rx, child) = app
    .shell()
    .sidecar("spacetimedb")
    .expect("failed to create sidecar command")
    .args(["start", "--listen-addr", "127.0.0.1:3000", "--data-dir", data_dir])
    .spawn()
    .expect("failed to spawn spacetimedb");

// Store child handle for cleanup
app.manage(Arc::new(Mutex::new(Some(child))));

// Wait for server to be ready, then connect SDK
```

### SDK Connection Lifecycle

```rust
// New module: src-tauri/src/spacetime/mod.rs
pub struct SpacetimeConnection {
    conn: DbConnection,
    // Cached client-side view from subscriptions
}

impl SpacetimeConnection {
    pub async fn connect(app_handle: AppHandle) -> Result<Self, Error> {
        let conn = DbConnection::builder()
            .with_uri("ws://127.0.0.1:3000")
            .with_module_name("heycat-data")
            .on_connect(|ctx| {
                // Subscribe to all tables
                ctx.subscription_builder()
                    .on_applied(|_| info!("Subscriptions applied"))
                    .subscribe(vec![
                        "SELECT * FROM recordings",
                        "SELECT * FROM settings",
                        "SELECT * FROM dictionary_entries",
                        "SELECT * FROM voice_commands",
                    ]);
            })
            .on_disconnect(|ctx, err| {
                error!("SpacetimeDB disconnected: {:?}", err);
            })
            .build()?;

        // Use run_threaded() for background processing
        conn.run_threaded();

        Ok(Self { conn })
    }
}
```

### Constraints & Non-Functional Requirements

| Requirement | Approach |
|-------------|----------|
| **Startup latency** | Sidecar spawns async; app functions with cached/empty state until ready |
| **Offline resilience** | SDK cache provides read access; writes queue until reconnected |
| **Resource usage** | Single SpacetimeDB process (~50-100MB); WAL files for persistence |
| **Platform support** | Sidecar binaries for macOS (arm64, x64), Windows, Linux |
| **Data migration** | One-time migration from existing JSON files on first run |

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Sidecar deployment (not embedded) | SpacetimeDB doesn't support in-process embedding; sidecar is the only option for local-first | 2025-12-22 |
| All data types in SpacetimeDB | Recordings, settings, dictionary, voice commands - consistent data layer | 2025-12-22 |
| Auto-start with app | Seamless UX; no manual server management for users | 2025-12-22 |
| Frontend unchanged | Keep React layer simple; SpacetimeDB is a backend concern | 2025-12-22 |
| Audio files on disk | Binary blobs stay in filesystem; only metadata in SpacetimeDB | 2025-12-22 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-22 | SpacetimeDB has no explicit offline-first support | Writes require server connection; need graceful degradation |
| 2025-12-22 | SDK uses `run_threaded()` for background processing | Fits well with Tauri's async model |
| 2025-12-22 | Tauri already has `tauri-plugin-shell` installed | Sidecar infrastructure ready to use |
| 2025-12-22 | SpacetimeDB client cache is read-only mirror | All mutations go through reducers; good for consistency |

## Open Questions

- [ ] How to handle first-time module deployment? Does sidecar need to `spacetime publish` on first run?
- [ ] What happens if SpacetimeDB sidecar crashes? Auto-restart strategy?
- [ ] How to bundle SpacetimeDB binary for each platform (macOS arm64/x64, Windows, Linux)?
- [ ] Should migrations run as reducers or via CLI before SDK connects?
- [ ] How to test SpacetimeDB integration in CI? Mock server or actual sidecar?

## Files to Modify

### New Files
- `src-tauri/src/spacetime/mod.rs` - SpacetimeDB client connection, lifecycle
- `src-tauri/src/spacetime/subscriptions.rs` - Table subscriptions, cache management
- `src-tauri/src/spacetime/migration.rs` - One-time data migration from JSON files
- `heycat-data/` - SpacetimeDB module (Rust, compiles to WASM)
- `heycat-data/src/lib.rs` - Tables, reducers, module entry point
- `src-tauri/binaries/` - SpacetimeDB sidecar binaries per platform

### Modified Files
- `src-tauri/src/lib.rs` - Sidecar spawn in setup(), SDK connection management
- `src-tauri/src/commands/mod.rs` - Delegate to SpacetimeDB instead of file I/O
- `src-tauri/src/commands/dictionary.rs` - Use SpacetimeDB reducers
- `src-tauri/src/dictionary/store.rs` - Backed by SpacetimeDB table
- `src-tauri/src/voice_commands/mod.rs` - Backed by SpacetimeDB table
- `src-tauri/tauri.conf.json` - Add `externalBin` for SpacetimeDB sidecar
- `src-tauri/capabilities/default.json` - Shell permissions for sidecar
- `src-tauri/Cargo.toml` - Add `spacetimedb-sdk` dependency

### Migration/Removal Candidates
- `src-tauri/src/dictionary/store.rs` - May become thin wrapper over SpacetimeDB
- Settings via Tauri Store - Migrated to SpacetimeDB `settings` table

## References

- [SpacetimeDB Rust SDK Reference](https://spacetimedb.com/docs/sdks/rust)
- [SpacetimeDB Rust Quickstart](https://spacetimedb.com/docs/sdks/rust/quickstart)
- [SpacetimeDB Self-Hosting](https://spacetimedb.com/docs/deploying/spacetimedb-standalone)
- [Tauri v2 Sidecar Documentation](https://v2.tauri.app/develop/sidecar/)
- [SpacetimeDB GitHub](https://github.com/clockworklabs/SpacetimeDB)
- [spacetimedb-sdk crate](https://crates.io/crates/spacetimedb-sdk)

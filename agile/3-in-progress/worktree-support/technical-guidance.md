---
last-updated: 2025-12-23
status: active
---

# Technical Guidance: Worktree Support

## Architecture Overview

### Layers Involved

```
┌─────────────────────────────────────────────────────────────┐
│                      Developer Scripts                       │
│  scripts/create-worktree.ts    scripts/cleanup-worktree.ts  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Backend (Rust)                     │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ worktree module │  │  paths module   │  │ lib.rs setup │ │
│  │  (detection)    │──│  (resolution)   │──│   (init)     │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│           │                   │                    │         │
│           ▼                   ▼                    ▼         │
│  ┌─────────────────────────────────────────────────────────┐│
│  │              Existing Path Consumers                     ││
│  │  model/download.rs  commands/logic.rs  dictionary/store ││
│  │  voice_commands/registry.rs                              ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      File System                             │
│  ~/.local/share/heycat/worktrees/{name}/                    │
│  ~/.config/heycat/worktrees/{name}/                         │
└─────────────────────────────────────────────────────────────┘
```

### Integration Pattern

1. **Worktree Detection** (new `src-tauri/src/worktree/mod.rs`)
   - Runs at app startup in `lib.rs::setup()`
   - Checks if `.git` is a file (worktree) vs directory (main repo)
   - Extracts worktree directory name as identifier
   - Stores context in Tauri app state

2. **Path Resolution** (new `src-tauri/src/paths.rs`)
   - Centralized module for all data/config path resolution
   - Queries worktree context from app state
   - Returns `heycat/worktrees/{name}/` suffix when in worktree
   - Returns `heycat/` for main repo (no change)

3. **Settings Isolation** (modify `lib.rs::setup()`)
   - Tauri plugin store uses worktree-specific filename
   - `settings.json` → `worktrees/{name}/settings.json`

4. **Developer Scripts** (new `scripts/*.ts`)
   - TypeScript scripts using same identifier algorithm
   - Create/cleanup worktrees with proper data setup

### Data Flow

```
App Start
    │
    ▼
detect_worktree()
    │
    ├── .git is directory → None (main repo)
    │
    └── .git is file → parse gitdir → extract dir name → Some("feature-x")
                                                              │
                                                              ▼
                                                    Store in app state
                                                              │
                    ┌─────────────────────────────────────────┴─────────────────────────────────────────┐
                    ▼                                         ▼                                         ▼
            get_data_dir()                           get_config_dir()                          store_path()
                    │                                         │                                         │
                    ▼                                         ▼                                         ▼
    ~/.local/share/heycat/worktrees/feature-x/   ~/.config/heycat/worktrees/feature-x/   worktrees/feature-x/settings.json
```

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Runtime worktree detection | More flexible than build-time; works if worktree is moved/renamed | 2025-12-23 |
| Use worktree directory name as identifier | Human-readable; easy to correlate data with worktree | 2025-12-23 |
| Subdirectory pattern (`heycat/worktrees/{name}/`) | Keeps worktree data grouped; easier cleanup; cleaner organization | 2025-12-23 |
| Centralized path resolution module | Single source of truth; easier to maintain; consistent behavior | 2025-12-23 |
| Lock file for collision detection | Standard pattern; allows detecting stale locks via PID check | 2025-12-23 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-23 | Settings stored via `tauri-plugin-store` in app data dir | Need to customize store path based on worktree context |
| 2025-12-23 | Models dir: `dirs::data_dir().join("heycat/models")` | Modify to include worktree subdirectory |
| 2025-12-23 | Recordings dir: `dirs::data_dir().join("heycat/recordings")` | Modify to include worktree subdirectory |
| 2025-12-23 | Config files: `dirs::config_dir().join("heycat")` | Modify to include worktree subdirectory |
| 2025-12-23 | Hotkey stored in settings as `hotkey.recordingShortcut` | Will be isolated per worktree via settings isolation |

## Open Questions

- [x] Runtime vs build-time detection? → Runtime
- [x] Path hash vs directory name? → Directory name
- [x] Suffix vs subdirectory pattern? → Subdirectory

## Files to Modify

### New Files
- `src-tauri/src/worktree/mod.rs` - Worktree detection module
- `src-tauri/src/paths.rs` - Centralized path resolution
- `scripts/create-worktree.ts` - Worktree creation script
- `scripts/cleanup-worktree.ts` - Worktree cleanup script

### Modified Files
- `src-tauri/src/lib.rs` - Add worktree detection to setup(), modify store path
- `src-tauri/src/main.rs` - Add worktree module
- `src-tauri/src/model/download.rs` - Use centralized path resolution
- `src-tauri/src/commands/logic.rs` - Use centralized path resolution
- `src-tauri/src/voice_commands/registry.rs` - Use centralized path resolution
- `src-tauri/src/dictionary/store.rs` - Use centralized path resolution

## References

- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
- [Tauri Plugin Store](https://v2.tauri.app/plugin/store/)
- [dirs crate](https://docs.rs/dirs/latest/dirs/)

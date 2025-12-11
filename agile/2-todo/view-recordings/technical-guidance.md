---
last-updated: 2025-12-01
status: active
---

# Technical Guidance: View Recordings

## Architecture Overview

The View Recordings feature adds a history view accessible via a collapsible sidebar. It requires coordinated changes across frontend (React) and backend (Rust/Tauri).

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (React)                                           │
├─────────────────────────────────────────────────────────────┤
│  App.tsx                                                    │
│    ├── Sidebar (collapsible)                                │
│    │     ├── RecordTab (current recording view)             │
│    │     └── HistoryTab → RecordingsView                    │
│    └── MainContent (current App content)                    │
│                                                             │
│  RecordingsView                                             │
│    ├── FilterBar (date range, duration)                     │
│    ├── RecordingsList (virtual scrolling)                   │
│    │     └── RecordingItem (expandable)                     │
│    └── EmptyState                                           │
│                                                             │
│  Hooks                                                      │
│    ├── useRecordings() - list, filter, refresh              │
│    └── useRecordingDetails() - expand/open                  │
└──────────────────────┬──────────────────────────────────────┘
                       │ invoke() / events
┌──────────────────────▼──────────────────────────────────────┐
│  Backend (Rust/Tauri)                                       │
├─────────────────────────────────────────────────────────────┤
│  Commands                                                   │
│    ├── list_recordings() → Vec<RecordingInfo>               │
│    └── open_recording(path) → Result<(), Error>             │
│                                                             │
│  Utilities                                                  │
│    ├── wav::parse_duration(path) → f64                      │
│    └── recordings_dir() → PathBuf                           │
└─────────────────────────────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│  File System                                                │
│    {app_data_dir}/heycat/recordings/                        │
│      ├── recording-2025-12-01-143025.wav                    │
│      └── ...                                                │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **List Recordings**: Frontend calls `invoke("list_recordings")` → Backend reads directory, parses WAV headers for duration → Returns `Vec<RecordingInfo>`
2. **Filter Recordings**: Frontend filters the list client-side (no backend round-trip needed)
3. **Open Recording**: Frontend calls `invoke("open_recording", { path })` → Backend uses `tauri-plugin-opener` or `std::process::Command` to open in default player

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Collapsible sidebar | Standard UX pattern, can collapse to save space | 2025-12-01 |
| Parse WAV header for duration | Accurate duration; WAV header is small (<100 bytes) | 2025-12-01 |
| Virtual scrolling for list | Supports large recording libraries efficiently | 2025-12-01 |
| Client-side filtering | Simpler implementation; recordings list fits in memory | 2025-12-01 |
| No separate metadata store | Derives from filesystem; matches existing pattern | 2025-12-01 |
| Auto-refresh on window focus | Catches new recordings made via hotkey in background | 2025-12-01 |

## Backend Implementation

### RecordingInfo Struct

```rust
#[derive(Debug, Clone, Serialize)]
pub struct RecordingInfo {
    pub filename: String,           // "recording-2025-12-01-143025.wav"
    pub file_path: String,          // Full path for opening
    pub duration_secs: f64,         // Parsed from WAV header
    pub created_at: String,         // ISO 8601 timestamp
    pub file_size_bytes: u64,       // For display
    pub is_valid: bool,             // False if corrupted/unreadable
    pub error_message: Option<String>, // If is_valid=false
}
```

### WAV Header Parsing

WAV duration calculation from header:
```
duration = (file_size - 44) / (sample_rate * channels * bits_per_sample / 8)
```

The existing `src-tauri/src/audio/wav.rs` has `hound` crate available - use `hound::WavReader` to read header metadata.

### Commands to Add

```rust
// In commands/mod.rs
#[tauri::command]
pub fn list_recordings() -> Result<Vec<RecordingInfo>, String>

#[tauri::command]
pub fn open_recording(path: String) -> Result<(), String>
```

### Error Handling

- Directory not found: Return empty list (not an error)
- File unreadable: Include in list with `is_valid: false` and error message
- Invalid WAV header: Mark as invalid, use file size estimate for duration

## Frontend Implementation

### Component Structure

```
src/
├── components/
│   ├── Sidebar/
│   │   ├── Sidebar.tsx          # Collapsible container
│   │   ├── Sidebar.css
│   │   └── SidebarTab.tsx       # Tab item
│   ├── RecordingsView/
│   │   ├── RecordingsView.tsx   # Main view container
│   │   ├── RecordingsList.tsx   # Virtual scroll list
│   │   ├── RecordingItem.tsx    # Single item (expandable)
│   │   ├── FilterBar.tsx        # Date/duration filters
│   │   └── EmptyState.tsx       # No recordings / no matches
│   └── RecordingIndicator.tsx   # Existing
├── hooks/
│   ├── useRecording.ts          # Existing
│   └── useRecordings.ts         # New: list/filter/refresh
└── App.tsx                      # Add Sidebar integration
```

### useRecordings Hook

```typescript
interface UseRecordingsReturn {
  recordings: RecordingInfo[];
  filteredRecordings: RecordingInfo[];
  isLoading: boolean;
  error: string | null;
  filters: RecordingFilters;
  setFilters: (filters: RecordingFilters) => void;
  refresh: () => Promise<void>;
  openRecording: (path: string) => Promise<void>;
}

interface RecordingFilters {
  dateRange?: { start: Date; end: Date };
  durationRange?: { min: number; max: number };
}
```

**Auto-refresh behavior**: The hook listens to `window.focus` event and refreshes the list automatically. Also refreshes when `recording_stopped` event fires (in case user made a new recording).

### Virtual Scrolling

Options for virtual scrolling:
- `@tanstack/react-virtual` - lightweight, no dependencies
- `react-window` - established, smaller bundle

Recommendation: `@tanstack/react-virtual` (modern, well-maintained)

### Sidebar State

Store collapsed state in localStorage for persistence:
```typescript
const [isCollapsed, setIsCollapsed] = useState(() =>
  localStorage.getItem('sidebar-collapsed') === 'true'
);
```

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-01 | Recordings stored at `{app_data_dir}/heycat/recordings/` | Known directory for list_recordings |
| 2025-12-01 | WAV files use pattern `recording-YYYY-MM-DD-HHMMSS.wav` | Can extract timestamp from filename as fallback |
| 2025-12-01 | No router in frontend - single page app | Sidebar tabs, not routes |
| 2025-12-01 | `hound` crate available for WAV parsing | Reuse existing dependency |
| 2025-12-01 | `tauri-plugin-opener` already in deps | Use for opening recordings |

## Open Questions

- [x] Sidebar style → Collapsible sidebar
- [x] Duration extraction → Parse WAV header
- [x] List pagination → Virtual scrolling
- [x] Auto-refresh on window focus → Yes

## Files to Modify

### Backend (Rust)
- `src-tauri/src/commands/mod.rs` - Add list_recordings, open_recording commands
- `src-tauri/src/commands/logic.rs` - Implement command logic
- `src-tauri/src/audio/wav.rs` - Add parse_duration_from_header function
- `src-tauri/src/lib.rs` - Register new commands

### Frontend (TypeScript/React)
- `src/App.tsx` - Integrate sidebar layout
- `src/App.css` - Layout styles for sidebar
- `src/hooks/useRecordings.ts` - New hook (create)
- `src/components/Sidebar/*` - New components (create)
- `src/components/RecordingsView/*` - New components (create)

### Dependencies
- `package.json` - Add `@tanstack/react-virtual`

## References

- [WAV File Format](https://docs.fileformat.com/audio/wav/)
- [hound crate docs](https://docs.rs/hound)
- [TanStack Virtual](https://tanstack.com/virtual/latest)
- [tauri-plugin-opener](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/opener)

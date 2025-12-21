---
last-updated: 2025-12-21
status: draft
---

# Technical Guidance: Dictionary Expansion

## Architecture Overview

### Layers Involved

1. **Frontend (React/TypeScript)**
   - New `/dictionary` route and Dictionary page component
   - Dictionary management UI (add/edit/delete entries)
   - Integration with transcription display to show expansions

2. **State Management**
   - **Zustand store**: Cache dictionary entries for fast UI access
   - **Tauri Store** (`settings.json` or new `dictionary.json`): Persist entries for backend access
   - Dual-write pattern (same as existing settings pattern in `useSettings.ts`)

3. **Backend (Rust)**
   - Dictionary expansion logic in transcription pipeline
   - Integrate into `RecordingTranscriptionService::process_recording()` after transcription, before clipboard/paste
   - Case-insensitive, whole-word matching using regex or string manipulation

### Integration Points

```
Transcription Flow (current):
  Audio → Parakeet → transcription text → command matching → clipboard/paste

Transcription Flow (with dictionary):
  Audio → Parakeet → transcription text → DICTIONARY EXPANSION → command matching → clipboard/paste
```

The dictionary expansion step:
1. Loads dictionary entries from Tauri Store (or in-memory cache)
2. Applies whole-word, case-insensitive replacements
3. Passes expanded text to command matching and clipboard

### Architectural Patterns to Follow

- **Event-driven UI updates**: Backend emits events, Event Bridge dispatches to Query/Zustand
- **Dual-write for persistence**: Zustand for fast reads, Tauri Store for backend access
- **New route pattern**: Add `/dictionary` route in `routes.tsx`, create `Dictionary.tsx` page
- **Settings pattern**: Follow `useSettings.ts` pattern for dictionary CRUD hooks

## Data Flow Diagram

This diagram aligns with the existing architecture in `docs/ARCHITECTURE.md`.

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                                    FRONTEND (React)                                      │
│                                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────────────────────┐│
│  │                              React Components                                        ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  ┌──────────────────┐   ││
│  │  │  Dashboard  │  │  Dictionary │  │  Transcription View │  │  Other Pages     │   ││
│  │  │             │  │    Page     │  │  (shows expanded)   │  │                  │   ││
│  │  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  └────────┬─────────┘   ││
│  │         │                │                    │                      │              ││
│  │         ▼                ▼                    ▼                      ▼              ││
│  │  ┌─────────────────────────────────────────────────────────────────────────────┐   ││
│  │  │                              Hooks Layer                                     │   ││
│  │  │  useDictionary()          useTranscription()         useSettings()          │   ││
│  │  │  - listEntries()          - transcribedText          - settings             │   ││
│  │  │  - addEntry()             - isTranscribing                                  │   ││
│  │  │  - updateEntry()                                                            │   ││
│  │  │  - deleteEntry()                                                            │   ││
│  │  └────────┬─────────────────────────┬─────────────────────────┬────────────────┘   ││
│  └───────────┼─────────────────────────┼─────────────────────────┼────────────────────┘│
│              │                         │                         │                      │
│   ┌──────────┼─────────────────────────┼─────────────────────────┼───────────────────┐ │
│   ▼          ▼                         ▼                         ▼                   │ │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────┐               │ │
│  │     Zustand Store   │  │   Tanstack Query    │  │   Tauri Store   │               │ │
│  │                     │  │                     │  │                 │               │ │
│  │ • dictionaryCache   │  │ ┌─────────────────┐ │  │ dictionary.json │               │ │
│  │   (fast reads)      │  │ │  Query Cache    │ │  │ (persistence)   │               │ │
│  │ • settings          │  │ │ ['tauri',...]   │ │  │                 │               │ │
│  │ • transcription     │  │ └────────┬────────┘ │  │                 │               │ │
│  └─────────┬───────────┘  │          │          │  └────────┬────────┘               │ │
│            │              │          ▼          │           │                        │ │
│            │              │  ┌───────────────┐  │           │                        │ │
│            │              │  │ queryFn:      │  │           │                        │ │
│            │              │  │ invoke(cmd)   │──┼───────────┼────────────────────────┘ │
│            │              │  └───────┬───────┘  │           │                          │
│            │              └──────────┼──────────┘           │                          │
│            │                         │                      │                          │
│            │                         ▼                      │                          │
│            │              ┌─────────────────────────────────┼──────────────────────┐   │
│            │              │           Event Bridge          │                      │   │
│            │              │                                 │                      │   │
│            │              │  listen('dictionary_updated') ──┼──▶ invalidateQueries │   │
│            │              │  listen('transcription_done') ──┼──▶ store.update()    │   │
│            │              │                                 │                      │   │
│            └──────────────┼─────────────────────────────────┘                      │   │
│                           ▲                                                        │   │
└───────────────────────────┼────────────────────────────────────────────────────────────┘
                            │
             ═══════════════╪═══════════════  Tauri IPC Boundary  ════════════════
                            │
┌───────────────────────────┼────────────────────────────────────────────────────────────┐
│                           │           BACKEND (Rust)                                   │
│                           ▼                                                            │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│  │                           #[tauri::command] Handlers                              │ │
│  │                                                                                   │ │
│  │  Dictionary Commands:              Recording Commands:                            │ │
│  │  list_dictionary_entries()         start_recording()                              │ │
│  │  add_dictionary_entry()            stop_recording()                               │ │
│  │  update_dictionary_entry()                                                        │ │
│  │  delete_dictionary_entry()                                                        │ │
│  │         │                                  │                                      │ │
│  └─────────┼──────────────────────────────────┼──────────────────────────────────────┘ │
│            │                                  │                                        │
│            ▼                                  ▼                                        │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│  │                              Core Business Logic                                  │ │
│  │                                                                                   │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────────┐   │ │
│  │  │ DictionaryStore │  │ RecordingManager│  │ RecordingTranscriptionService   │   │ │
│  │  │                 │  │                 │  │                                 │   │ │
│  │  │ • load()        │  │ Arc<Mutex<T>>   │  │ process_recording():            │   │ │
│  │  │ • save()        │  │                 │  │   1. Transcribe (Parakeet)      │   │ │
│  │  │ • get_entries() │  │                 │  │   2. ──▶ EXPAND (Dictionary) ◀──│   │ │
│  │  │ • add_entry()   │◀─┼─────────────────┼──│   3. Command matching           │   │ │
│  │  │ • update/delete │  │                 │  │   4. Clipboard + paste          │   │ │
│  │  └────────┬────────┘  └────────┬────────┘  └────────────────┬────────────────┘   │ │
│  │           │                    │                            │                    │ │
│  └───────────┼────────────────────┼────────────────────────────┼────────────────────┘ │
│              │                    │                            │                      │
│              ▼                    ▼                            ▼                      │
│  ┌───────────────────────────────────────────────────────────────────────────────────┐ │
│  │                            app_handle.emit()                                      │ │
│  │                                                                                   │ │
│  │   emit("dictionary_updated")   emit("recording_started")   emit("transcription_  │ │
│  │              │                          │                   completed")           │ │
│  └──────────────┼──────────────────────────┼────────────────────────┼────────────────┘ │
│                 │                          │                        │                  │
└─────────────────┼──────────────────────────┼────────────────────────┼──────────────────┘
                  │                          │                        │
                  └──────────────────────────┴────────────────────────┘
                                             │
                              Tauri IPC (events to frontend)
                                             │
                                             ▼
                              ┌─────────────────────────┐
                              │      Event Bridge       │
                              │   (receives & routes)   │
                              └─────────────────────────┘
```

## Transcription + Expansion Pipeline Detail

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TRANSCRIPTION + EXPANSION PIPELINE                        │
│                                                                              │
│  ┌────────────┐    ┌────────────┐    ┌────────────────┐    ┌─────────────┐ │
│  │   Audio    │───▶│  Parakeet  │───▶│   Dictionary   │───▶│  Command    │ │
│  │   File     │    │ Transcribe │    │   Expander     │    │  Matcher    │ │
│  └────────────┘    └────────────┘    └────────────────┘    └─────────────┘ │
│                           │                  │                     │        │
│                           ▼                  ▼                     ▼        │
│                    "i need to brb"    "i need to be      Match? ──▶ Execute │
│                                        right back"              │           │
│                                              │                  No          │
│                                              │                  ▼           │
│                                              └──────────▶ Clipboard + Paste │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

Dictionary Expander Logic:
┌─────────────────────────────────────────────────────────────────────────────┐
│  Input: "i need to brb and check the api docs"                              │
│                                                                              │
│  Dictionary Entries:                                                         │
│    { trigger: "brb", expansion: "be right back" }                           │
│    { trigger: "api", expansion: "API" }                                     │
│                                                                              │
│  Processing (case-insensitive, whole-word):                                 │
│    1. Find "\bbrb\b" (case-insensitive) → replace with "be right back"     │
│    2. Find "\bapi\b" (case-insensitive) → replace with "API"               │
│                                                                              │
│  Output: "i need to be right back and check the API docs"                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Apply expansion in Rust backend | Keeps expansion close to transcription, ensures consistency across all entry points (hotkey, button, wake word) | 2025-12-21 |
| Use Tauri Store for persistence | Follows existing pattern, backend can access without frontend context | 2025-12-21 |
| Separate `dictionary.json` store | Keeps dictionary data isolated from settings, allows larger entry counts | 2025-12-21 |
| Whole-word matching via regex | Prevents partial matches (e.g., "cat" in "concatenate") | 2025-12-21 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-21 | Transcription happens in `RecordingTranscriptionService::process_recording()` | Dictionary expansion should be injected here, after transcription result, before command matching |
| 2025-12-21 | Settings use dual-write pattern (Zustand + Tauri Store) | Dictionary should follow same pattern for consistency |
| 2025-12-21 | Routes defined in `src/routes.tsx` | Add `/dictionary` route alongside existing `/settings`, `/recordings`, etc. |

## Open Questions

- [x] Where in the pipeline should expansion happen? → After Parakeet transcription, before command matching
- [ ] Should we limit dictionary size? (performance consideration for large dictionaries)
- [ ] How to handle conflicting entries (e.g., "api" → "API" vs "api" → "Application Programming Interface")?

## Files to Modify

**Frontend:**
- `src/routes.tsx` - Add `/dictionary` route
- `src/pages/index.ts` - Export new Dictionary page
- `src/pages/Dictionary.tsx` - New page component (create)
- `src/hooks/useDictionary.ts` - Dictionary CRUD hooks (create)
- `src/stores/appStore.ts` - Add dictionary cache slice
- `src/components/layout/AppShell.tsx` - Add dictionary nav item

**Backend:**
- `src-tauri/src/dictionary/mod.rs` - Dictionary module (create)
- `src-tauri/src/dictionary/expander.rs` - Expansion logic (create)
- `src-tauri/src/transcription/service.rs` - Integrate dictionary expansion
- `src-tauri/src/commands/mod.rs` - Add dictionary CRUD commands
- `src-tauri/src/lib.rs` - Register dictionary commands and module

## References

- `src/hooks/useSettings.ts` - Pattern for dual-write (Zustand + Tauri Store)
- `src-tauri/src/transcription/service.rs` - Transcription pipeline integration point
- `docs/ARCHITECTURE.md` - State management and event patterns

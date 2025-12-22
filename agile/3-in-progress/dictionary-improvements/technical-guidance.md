---
last-updated: 2025-12-22
status: active
---

# Technical Guidance: Dictionary Improvements

## Architecture Overview

This feature extends the existing dictionary system to support per-entry configuration options: **suffix** (text appended after expansion) and **autoEnter** (simulates enter keypress after expansion).

The feature follows the established frontend-backend communication pattern:
- **Data model changes** flow from Rust types → TypeScript types
- **CRUD operations** use existing Tauri commands with extended payloads
- **Expansion logic** is enhanced in the backend DictionaryExpander
- **UI updates** propagate via Event Bridge pattern

### Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    FRONTEND (React)                                              │
│                                                                                                  │
│  ┌──────────────────────────────────────────────────────────────────────────────────────────┐   │
│  │                              Dictionary Page (/dictionary)                                 │   │
│  │                                                                                            │   │
│  │  ┌─────────────────────┐    ┌─────────────────────────────────────────────────────────┐   │   │
│  │  │   AddEntryForm      │    │                    EntryItem                             │   │   │
│  │  │                     │    │  ┌─────────────────────────────────────────────────┐    │   │   │
│  │  │  • trigger input    │    │  │  Entry Display: trigger → expansion             │    │   │   │
│  │  │  • expansion input  │    │  │  (Edit) (Delete) (⚙️ Settings)                   │    │   │   │
│  │  │  • (⚙️ Settings)    │    │  └─────────────────────────────────────────────────┘    │   │   │
│  │  │                     │    │                         │                                │   │   │
│  │  │  Settings Panel:    │    │                         ▼                                │   │   │
│  │  │  ┌────────────────┐ │    │  ┌─────────────────────────────────────────────────┐    │   │   │
│  │  │  │ Suffix: _____  │ │    │  │  Settings Panel (collapsible)                   │    │   │   │
│  │  │  │ Auto-Enter: ☐  │ │    │  │  ┌──────────────────────────────────────────┐   │    │   │   │
│  │  │  └────────────────┘ │    │  │  │ Suffix: .?! (max 5 chars)                │   │    │   │   │
│  │  └─────────────────────┘    │  │  │ Auto-Enter: (toggle)                     │   │    │   │   │
│  │           │                  │  │  └──────────────────────────────────────────┘   │    │   │   │
│  │           │                  │  └─────────────────────────────────────────────────┘    │   │   │
│  │           │                  └─────────────────────────────────────────────────────────┘   │   │
│  └───────────┼────────────────────────────────────────────────────────────────────────────────┘   │
│              │                                           │                                         │
│              ▼                                           ▼                                         │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                                useDictionary Hook                                          │    │
│  │                                                                                            │    │
│  │  • addEntry(trigger, expansion, suffix?, autoEnter?)                                      │    │
│  │  • updateEntry(id, trigger, expansion, suffix?, autoEnter?)                               │    │
│  │  • deleteEntry(id)                                                                         │    │
│  │  • entries: DictionaryEntry[] (via Tanstack Query)                                        │    │
│  │                                                                                            │    │
│  │  Mutations call invoke() → Tauri commands                                                 │    │
│  └────────────────────────────────────────┬──────────────────────────────────────────────────┘    │
│                                           │                                                        │
│                                           ▼                                                        │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                                  Event Bridge                                              │    │
│  │                                                                                            │    │
│  │  listen("dictionary_updated") ───────────▶ invalidateQueries(["dictionary"])              │    │
│  │                                                                                            │    │
│  └───────────────────────────────────────────────────────────────────────────────────────────┘    │
│                                           ▲                                                        │
└───────────────────────────────────────────┼────────────────────────────────────────────────────────┘
                                            │
                         ═══════════════════╪═══════════════  Tauri IPC Boundary  ═══════════════════
                                            │
┌───────────────────────────────────────────┼────────────────────────────────────────────────────────┐
│                                           │           BACKEND (Rust)                               │
│                                           ▼                                                        │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                              #[tauri::command] Handlers                                    │    │
│  │                                                                                            │    │
│  │  add_dictionary_entry(trigger, expansion, suffix?, auto_enter?)                           │    │
│  │  update_dictionary_entry(id, trigger, expansion, suffix?, auto_enter?)                    │    │
│  │  list_dictionary_entries() → Vec<DictionaryEntry>                                         │    │
│  │  delete_dictionary_entry(id)                                                               │    │
│  │                                                                                            │    │
│  └────────────────────────────────────────┬──────────────────────────────────────────────────┘    │
│                                           │                                                        │
│                                           ▼                                                        │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                                  DictionaryStore                                           │    │
│  │                                                                                            │    │
│  │  DictionaryEntry {                                                                         │    │
│  │      id: String,                                                                           │    │
│  │      trigger: String,                                                                      │    │
│  │      expansion: String,                                                                    │    │
│  │      suffix: Option<String>,      // NEW: appended to expansion                           │    │
│  │      auto_enter: bool,            // NEW: triggers enter keypress                         │    │
│  │  }                                                                                         │    │
│  │                                                                                            │    │
│  │  • Persists to ~/.config/heycat/dictionary.json                                           │    │
│  │  • Backward compatible: missing fields default to None/false                              │    │
│  │                                                                                            │    │
│  └────────────────────────────────────────┬──────────────────────────────────────────────────┘    │
│                                           │                                                        │
│                                           ▼                                                        │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                                DictionaryExpander                                          │    │
│  │                                                                                            │    │
│  │  expand(text: &str) → ExpansionResult {                                                   │    │
│  │      expanded_text: String,       // text with suffix appended                            │    │
│  │      should_press_enter: bool,    // true if any expanded entry has auto_enter            │    │
│  │  }                                                                                         │    │
│  │                                                                                            │    │
│  │  • Applies suffix to expansion text                                                        │    │
│  │  • Tracks whether enter should be pressed                                                  │    │
│  │                                                                                            │    │
│  └────────────────────────────────────────┬──────────────────────────────────────────────────┘    │
│                                           │                                                        │
│                                           ▼                                                        │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                          RecordingTranscriptionService                                     │    │
│  │                                                                                            │    │
│  │  After transcription:                                                                      │    │
│  │    1. expand(transcribed_text) → ExpansionResult                                          │    │
│  │    2. Type expanded_text via keyboard simulation                                           │    │
│  │    3. If should_press_enter → simulate_enter_keypress()                                   │    │
│  │                                                                                            │    │
│  └────────────────────────────────────────┬──────────────────────────────────────────────────┘    │
│                                           │                                                        │
│                                           ▼                                                        │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                              KeyboardSimulator (NEW)                                       │    │
│  │                                                                                            │    │
│  │  simulate_enter_keypress()                                                                 │    │
│  │  • Uses enigo or rdev crate for cross-platform keyboard simulation                        │    │
│  │  • Sends Return/Enter key event                                                            │    │
│  │                                                                                            │    │
│  └───────────────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                                    │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                                  app_handle.emit()                                         │    │
│  │                                                                                            │    │
│  │  emit("dictionary_updated") ─────────────────────────────────────────▶ Event Bridge       │    │
│  │                                                                                            │    │
│  └───────────────────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                                    │
└────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Entry Points

| Entry Point | Flow |
|-------------|------|
| UI: Add/Edit entry with settings | Dictionary Page → useDictionary hook → invoke() → Tauri command → DictionaryStore |
| Transcription expansion | TranscriptionService → DictionaryExpander.expand() → type text + optional enter |

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Use `Option<String>` for suffix | Backward compatible, allows empty/unset state | 2025-12-22 |
| Use `bool` for auto_enter | Simple on/off toggle, defaults to false | 2025-12-22 |
| Return ExpansionResult struct | Cleaner than tuple; extensible for future options | 2025-12-22 |
| Use enigo for keyboard simulation | Cross-platform, well-maintained, already used by similar apps | 2025-12-22 |
| Settings panel collapsible per-entry | Keeps UI clean, shows only when needed | 2025-12-22 |
| Max 5 chars for suffix | Prevents abuse, sufficient for punctuation use cases | 2025-12-22 |

## Testing Strategy

Based on TESTING.md principles: test behavior, not implementation.

### Backend Testing Pattern

| Spec | Test Focus | Test Type |
|------|------------|-----------|
| data-model-update | Serialization round-trip with new fields | Unit |
| backend-storage-update | Load/save with backward compatibility | Unit |
| expander-suffix-support | Expansion output includes suffix | Unit |
| keyboard-simulation | Enter keypress simulation | Integration (requires permissions) |

### Frontend Testing Pattern

| Spec | Test Focus | Test Type |
|------|------------|-----------|
| settings-panel-ui | Panel expands/collapses, fields update entry | Component |
| suffix-validation | Error shown for suffix > 5 chars | Component |

### Test Commands

```bash
# Quick tests (during spec implementation)
bun run test && cd src-tauri && cargo test

# Coverage tests (during feature review)
bun run test:coverage && cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'
```

## Files to Modify

### Backend (Rust)

| File | Changes |
|------|---------|
| `src-tauri/src/dictionary/store.rs` | Add `suffix: Option<String>`, `auto_enter: bool` to DictionaryEntry struct |
| `src-tauri/src/dictionary/expander.rs` | Return ExpansionResult with suffix applied and should_press_enter flag |
| `src-tauri/src/commands/dictionary.rs` | Accept suffix/auto_enter params in add/update commands |
| `src-tauri/src/keyboard/mod.rs` | NEW: KeyboardSimulator module with simulate_enter_keypress() |
| `src-tauri/src/transcription/service.rs` | Use ExpansionResult, call keyboard simulation when needed |
| `src-tauri/src/lib.rs` | Register keyboard module |

### Frontend (TypeScript/React)

| File | Changes |
|------|---------|
| `src/types/dictionary.ts` | Add `suffix?: string`, `autoEnter?: boolean` to DictionaryEntry interface |
| `src/hooks/useDictionary.ts` | Pass suffix/autoEnter to invoke calls |
| `src/pages/Dictionary.tsx` | Add collapsible settings panel per entry with suffix input and auto-enter toggle |

## Open Questions

- RESOLVED: Which keyboard simulation crate to use? → Decided: enigo
- OPEN: Should we debounce enter keypress if multiple entries expand in same text?

## References

- docs/ARCHITECTURE.md - Frontend-Backend Communication section
- docs/ARCHITECTURE.md - Event Bridge Pattern section
- docs/TESTING.md - Testing Philosophy

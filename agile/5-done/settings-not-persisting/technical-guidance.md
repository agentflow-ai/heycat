---
last-updated: 2025-12-15
status: finalized
---

# Technical Guidance: Settings Not Persisting

## Root Cause Analysis

**Persistence architecture mismatch**: Settings are persisted in the frontend store but the backend never reads them on startup.

### The Problem

1. **Frontend persistence works correctly**
   - `useSettings.ts` saves `listening.enabled` and `listening.autoStartOnLaunch` to Tauri store
   - Settings persist in `settings.json` via `@tauri-apps/plugin-store`

2. **Backend ignores persisted settings**
   - `ListeningManager::new()` hardcodes `listening_enabled: false` (manager.rs:75)
   - `lib.rs:64` creates fresh instance on every app start
   - No code reads the store to initialize backend state

3. **Result**: UI loads correct toggle states from store, but backend state is always `false` on startup

### Key Files

| File | Role | Issue |
|------|------|-------|
| `src/hooks/useSettings.ts:86-122` | Saves settings to store | Works correctly |
| `src-tauri/src/listening/manager.rs:75` | Initializes listening state | Hardcoded to `false` |
| `src-tauri/src/lib.rs:64` | Creates ListeningManager | Never reads from store |
| `src/hooks/useAutoStartListening.ts:20-35` | Auto-start on launch | Invokes backend but backend is out of sync |

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Backend should read settings from store on init | Single source of truth, frontend already persists correctly | 2025-12-15 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-15 | `ListeningManager::new()` hardcodes `listening_enabled: false` | Backend always starts in disabled state |
| 2025-12-15 | Frontend store saves settings correctly | Only need to fix backend initialization |
| 2025-12-15 | No backend code reads from Tauri store | Need to add store reading at startup |

## Open Questions

- [x] Where are settings stored? → Tauri store plugin (`settings.json`)
- [x] Does frontend persistence work? → Yes, correctly saves to store
- [x] Why doesn't backend read settings? → Never implemented

## Files to Modify

- `src-tauri/src/listening/manager.rs` - Add method to initialize from stored settings
- `src-tauri/src/lib.rs` - Read settings from store and initialize ListeningManager with them

## References

- Tauri Store Plugin: https://v2.tauri.app/plugin/store/

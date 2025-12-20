---
last-updated: 2025-12-20
status: active
---

# Technical Guidance: Listening Settings Not Persisting

## Root Cause Analysis

### Problem Summary
Settings key mismatch between frontend and backend initialization.

### Observed vs Expected Behavior
- **Expected:** When "Auto Start Listening" is disabled, app should start with listening off
- **Observed:** App starts with listening enabled regardless of the setting

### Root Cause
The backend reads the wrong settings key at startup:

| Component | Settings Key | Purpose |
|-----------|-------------|---------|
| Frontend useSettings | `listening.enabled` | Current listening state |
| Frontend useAutoStartListening | `listening.autoStartOnLaunch` | Auto-start preference |
| **Backend on startup** | `listening.enabled` ❌ | Should read `listening.autoStartOnLaunch` |

**What happens:**
1. User sets `autoStartOnLaunch = false` in settings
2. Backend starts and reads `listening.enabled` (different key)
3. If `listening.enabled = true` from last session, listening starts
4. The `autoStartOnLaunch` preference is ignored

### Code Locations
- **Backend initialization:** `src-tauri/src/lib.rs:68-76` - reads wrong key
- **Frontend auto-start hook:** `src/hooks/useAutoStartListening.ts:23-34` - correctly reads `autoStartOnLaunch`
- **Frontend settings:** `src/hooks/useSettings.ts:133-150` - correctly saves to `autoStartOnLaunch`

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Fix backend to read `autoStartOnLaunch` | Aligns backend with frontend intent for auto-start behavior | 2025-12-20 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-20 | Backend reads `listening.enabled` instead of `listening.autoStartOnLaunch` | Direct cause of bug |
| 2025-12-20 | Frontend correctly uses `listening.autoStartOnLaunch` for auto-start logic | Frontend is correct, backend needs fix |

## Open Questions

- [x] Root cause identified

## Files to Modify

- `src-tauri/src/lib.rs` - Change startup initialization to read `listening.autoStartOnLaunch` instead of `listening.enabled`

## References

- `src/hooks/useAutoStartListening.ts` - Frontend auto-start logic (reference implementation)
- `src/hooks/useSettings.ts` - Settings hook showing correct key usage

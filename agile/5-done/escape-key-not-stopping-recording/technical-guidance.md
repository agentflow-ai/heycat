---
last-updated: 2025-12-22
status: validated
---

# Technical Guidance: Escape Key Not Stopping Recording

## Root Cause Analysis

The frontend Event Bridge (`src/lib/eventBridge.ts`) is missing a listener for the `recording_cancelled` event.

**Event flow:**
1. User double-taps Escape during recording
2. Backend detects double-tap via `DoubleTapDetector` (300ms window)
3. Backend calls `cancel_recording()` which:
   - Stops audio capture
   - Transitions state to Idle
   - Emits `recording_cancelled` event
4. **Frontend has no listener** for `recording_cancelled`
5. Query cache is never invalidated → stale "recording" state persists
6. UI components derive state from stale cache → overlay stays visible

## Key Files

| File | Purpose |
|------|---------|
| `src/lib/eventBridge.ts` | **FIX HERE** - Add event listener |
| `src-tauri/src/hotkey/integration.rs` | Backend cancellation (working correctly) |
| `src-tauri/src/events.rs` | Event definitions (has `RECORDING_CANCELLED`) |
| `src/hooks/useRecording.ts` | Recording state query (will auto-update after fix) |
| `src/hooks/useCatOverlay.ts` | Overlay mode derivation (will auto-update after fix) |

## Fix Approach

1. Add `RECORDING_CANCELLED: "recording_cancelled"` to `eventNames` object
2. Add listener that calls `queryClient.invalidateQueries({ queryKey: queryKeys.tauri.getRecordingState })`

Pattern matches existing `RECORDING_STARTED`, `RECORDING_STOPPED`, `RECORDING_ERROR` handlers.

## Regression Risk

**Low risk:**
- Adding a new event listener doesn't affect existing listeners
- Query invalidation is idempotent (safe to call multiple times)
- Only triggers on `recording_cancelled` event, which is only emitted during cancellation

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-22 | Backend logs confirm correct event emission | Backend is working |
| 2025-12-22 | `eventNames` missing `RECORDING_CANCELLED` | Root cause identified |
| 2025-12-22 | No listener in `setupEventBridge()` | Fix location confirmed |

## Open Questions

None - root cause fully identified.

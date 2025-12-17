---
last-updated: 2025-12-17
status: draft
---

# Technical Guidance: Audio Input Device Selection

## Architecture Overview

The implementation spans both Rust backend and React frontend:

**Backend (Rust):**
- CPAL library (0.15) handles audio device enumeration and capture
- Current implementation uses `host.default_input_device()` in `cpal_backend.rs:176-180`
- Need to add device enumeration function and modify `start()` to accept device name
- Use Tauri store plugin for persistence (already in use)

**Frontend (React/TypeScript):**
- Existing `useSettings` hook pattern for persistence (`src/hooks/useSettings.ts`)
- Add device selector to `ListeningSettings` component (existing UI pattern)
- New `useAudioDevices` hook for device enumeration

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Use device name as identifier | CPAL doesn't provide stable unique IDs; names are human-readable | 2025-12-15 |
| Fall back to default on missing device | Bluetooth devices may disconnect; graceful degradation | 2025-12-15 |
| Add to ListeningSettings tab | Audio input is closely related to listening mode | 2025-12-15 |
| Store selection in frontend | Follow existing `useSettings` pattern; simpler than backend state | 2025-12-15 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-15 | CPAL `default_input_device()` triggers CoreAudio device enumeration | This causes Bluetooth profile switching on macOS |
| 2025-12-15 | Existing settings use `tauri-plugin-store` via `useSettings` hook | Can extend same pattern for audio device selection |
| 2025-12-15 | `AudioCaptureBackend` trait defines `start()` signature | Need to update trait and all implementations |

## Open Questions

- [x] How to identify devices - Use device name
- [x] Where to store settings - Extend `useSettings` hook with audio section
- [x] UI placement - Add section to ListeningSettings component

## Files to Modify

**Backend (Create):**
- None (all modifications)

**Backend (Modify):**
- `src-tauri/src/audio/mod.rs` - Add `AudioInputDevice` struct, update trait
- `src-tauri/src/audio/cpal_backend.rs` - Add `list_input_devices()`, `find_device_by_name()`, modify `start()`
- `src-tauri/src/audio/thread.rs` - Update `AudioCommand::Start` to include device name
- `src-tauri/src/commands/mod.rs` - Add `list_audio_devices` command
- `src-tauri/src/commands/logic.rs` - Pass device name to audio start
- `src-tauri/src/lib.rs` - Register new command
- `src-tauri/src/listening/pipeline.rs` - Pass device name when starting listening

**Frontend (Create):**
- `src/types/audio.ts` - TypeScript types for audio devices
- `src/hooks/useAudioDevices.ts` - Hook for device enumeration
- `src/components/ListeningSettings/AudioDeviceSelector.tsx` - UI component
- `src/components/ListeningSettings/AudioDeviceSelector.css` - Styling

**Frontend (Modify):**
- `src/hooks/useSettings.ts` - Add audio settings section
- `src/components/ListeningSettings/ListeningSettings.tsx` - Add device selector
- `src/components/ListeningSettings/index.ts` - Export new component

## References

- [Plan file](/Users/michaelhindley/.claude/plans/polished-snacking-bengio.md) - Detailed implementation plan
- [CPAL documentation](https://docs.rs/cpal/latest/cpal/) - Audio library docs
- [Tauri Store Plugin](https://v2.tauri.app/plugin/store/) - Settings persistence

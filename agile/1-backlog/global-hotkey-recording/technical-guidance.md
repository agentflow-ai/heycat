---
last-updated: 2025-11-26
status: draft
---

# Technical Guidance: Global Hotkey Microphone Recording

## Architecture Overview

Three-layer architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│                    LAYER 3: Frontend                     │
│  Recording State Hook → Recording Indicator → App.tsx   │
└─────────────────────────────────────────────────────────┘
                            ↕ Events / IPC
┌─────────────────────────────────────────────────────────┐
│                 LAYER 2: IPC/Integration                 │
│  State Manager → Coordinator → Commands → Events        │
│                     ↕ Hotkey Integration                │
└─────────────────────────────────────────────────────────┘
                            ↕
┌─────────────────────────────────────────────────────────┐
│                  LAYER 1: Backend Core                   │
│  Audio Capture (cpal) │ WAV Encoding (hound) │ Hotkey   │
└─────────────────────────────────────────────────────────┘
```

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Layered architecture (12 specs) | Better testability, clear separation of concerns, each layer independently verifiable | 2025-11-26 |
| cpal for audio capture | Cross-platform, pure Rust, well-maintained | 2025-11-26 |
| hound for WAV encoding | Simple API, pure Rust, good cpal compatibility | 2025-11-26 |
| tauri-plugin-global-shortcut | Official Tauri v2 plugin, well-documented | 2025-11-26 |
| Include transcription buffer in MVP | Completes all feature requirements, enables future integration | 2025-11-26 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-11-26 | Current codebase has minimal dependencies (only tauri-plugin-opener) | Clean slate for adding audio/hotkey dependencies |
| 2025-11-26 | Project uses TCR workflow with 100% coverage | Need coverage exclusions for hardware interaction code |
| 2025-11-26 | Frontend uses simple useState, no Redux | useRecording hook fits existing patterns |

## Open Questions

- [ ] Default recording output directory (`~/heycat-recordings/` vs app data folder)
- [ ] Audio buffer retention policy for transcription (keep last N recordings?)
- [ ] Sample rate configuration (fixed 44.1kHz vs user-configurable)

## Files to Modify

**Backend (Rust):**
- `src-tauri/src/lib.rs` - Register commands, state, plugins
- `src-tauri/src/audio/mod.rs` - New module for capture
- `src-tauri/src/audio/capture.rs` - cpal audio capture
- `src-tauri/src/audio/wav.rs` - hound WAV encoding
- `src-tauri/src/recording/mod.rs` - New module for coordinator
- `src-tauri/src/recording/state.rs` - State manager
- `src-tauri/src/recording/coordinator.rs` - Orchestration logic
- `src-tauri/Cargo.toml` - Add dependencies
- `src-tauri/capabilities/default.json` - Add permissions

**Frontend (React):**
- `src/hooks/useRecording.ts` - New hook
- `src/components/RecordingIndicator.tsx` - New component
- `src/components/RecordingIndicator.css` - Component styles
- `src/App.tsx` - Integration
- `package.json` - Add @tauri-apps/plugin-global-shortcut

## Dependencies to Add

**Cargo.toml:**
```toml
cpal = "0.15"
hound = "3.5"
tauri-plugin-global-shortcut = "2"
```

**package.json:**
```json
"@tauri-apps/plugin-global-shortcut": "^2.0.0"
```

## References

- [Tauri v2 Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri v2 State Management](https://v2.tauri.app/develop/state-management/)
- [Tauri v2 Events API](https://v2.tauri.app/develop/calling-frontend/)
- [cpal - Cross-platform Audio](https://github.com/RustAudio/cpal)
- [hound - WAV encoding](https://github.com/ruuda/hound)

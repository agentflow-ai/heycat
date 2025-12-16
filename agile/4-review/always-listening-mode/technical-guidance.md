---
last-updated: 2025-12-14
status: draft
---

# Technical Guidance: Always Listening Mode

## Architecture Overview

### System Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (React)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │ useListening │  │ useRecording │  │ CatOverlay + Settings  │ │
│  └──────┬───────┘  └──────┬───────┘  └───────────┬────────────┘ │
└─────────┼─────────────────┼──────────────────────┼──────────────┘
          │ Tauri Events    │                      │
          ▼                 ▼                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Tauri Commands Layer                         │
│  enable_listening, disable_listening, get_listening_status       │
│  (existing: start_recording, stop_recording, transcribe_file)    │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────┐
│                    State Management (Rust)                       │
│  ┌──────────────────┐    ┌──────────────────────────────────┐   │
│  │ ListeningManager │◄──►│ RecordingManager (extended)      │   │
│  │ - enabled flag   │    │ - Idle/Listening/Recording/Proc  │   │
│  │ - mic available  │    │ - returns to Listening if enabled│   │
│  └────────┬─────────┘    └──────────────────────────────────┘   │
└───────────┼─────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Audio Pipeline (Rust)                        │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ AudioThreadHandle                                        │    │
│  │  - Continuous capture mode (listening)                   │    │
│  │  - Recording capture mode (existing)                     │    │
│  │  - Routes to: WakeWordDetector OR RecordingBuffer        │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│              ┌───────────────┼───────────────┐                   │
│              ▼               ▼               ▼                   │
│  ┌───────────────┐  ┌──────────────┐  ┌─────────────────┐       │
│  │WakeWordDetect │  │SilenceDetect │  │ CancelDetector  │       │
│  │ "Hey Cat"     │  │ RMS-based    │  │ "cancel"        │       │
│  └───────────────┘  └──────────────┘  └─────────────────┘       │
└─────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Parakeet Integration                           │
│  TranscriptionManager (existing) - batch transcription           │
│  WakeWordDetector (new) - small-window phrase detection          │
└─────────────────────────────────────────────────────────────────┘
```

### Module Structure

All listening-related code lives in a unified `listening/` module:

```
src-tauri/src/listening/
├── mod.rs           # Module exports, ListeningManager
├── detector.rs      # WakeWordDetector using Parakeet
├── silence.rs       # Simple energy-based silence detection (RMS)
└── buffer.rs        # Circular buffer for wake word analysis
```

### Architectural Patterns

1. **Event-Driven State Machine**: Extends existing pattern from RecordingManager
   - States: `Idle` → `Listening` ↔ `Recording` → `Processing` → `Listening|Idle`
   - Events emitted via Tauri AppHandle for frontend synchronization

2. **Dedicated Audio Thread**: Existing pattern via `AudioThreadHandle`
   - Non-Send cpal::Stream isolated on dedicated thread
   - Commands sent via mpsc channel: `StartListening`, `StopListening`, `StartRecording`, etc.

3. **Circular Buffer for Wake Word**: New pattern
   - Rolling ~3 second window for wake word analysis
   - Fixed-size ring buffer, separate from main recording buffer (Vec<f32>)

4. **Shared State via Arc<Mutex>**: Existing pattern
   - All managers wrapped in Arc<Mutex<T>> for thread-safe access
   - Managed via Tauri's `app.manage()` system

### State Machine Edge Cases

| From State | Event | To State | Notes |
|------------|-------|----------|-------|
| Idle | enable_listening | Listening | Start audio capture |
| Listening | disable_listening | Idle | Stop audio capture |
| Listening | wake_word_detected | Recording | listening_enabled stays true |
| Listening | hotkey_pressed | Recording | listening_enabled stays true |
| Recording | stop_recording | Listening* | *if listening_enabled, else Idle |
| Recording | wake_word_detected | Recording | Ignored (already recording) |
| Listening | mic_unavailable | Listening** | **emit listening_unavailable event |
| Processing | complete | Listening* | *if listening_enabled, else Idle |

**Key behaviors:**
- `listening_enabled` is a flag, not a state - it persists across Recording
- Hotkey during Listening suspends listening (not disables)
- Wake word ignored while already Recording
- Mic unavailability pauses listening, auto-resumes when available

### Integration Points

| Component | Integrates With | Integration Type |
|-----------|----------------|------------------|
| WakeWordDetector | AudioThread | Receives samples via channel |
| WakeWordDetector | ListeningManager | Emits detection events |
| ListeningManager | RecordingManager | Coordinates state transitions |
| SilenceDetector | AudioThread | Receives samples during recording |
| SilenceDetector | RecordingManager | Triggers auto-stop |
| useListening hook | Tauri events | Subscribes to state changes |
| CatOverlay | useListening | Displays listening/recording indicator |

### Non-Functional Requirements

- **Memory**: Bounded by circular buffer (~3s * 16kHz * 4 bytes = ~192KB)
- **Privacy**: All processing on-device, no network required

> **MVP Note**: CPU and latency targets deferred to post-MVP optimization phase.

## Spec Dependencies & Implementation Order

```
                    ┌─────────────────────┐
                    │ 1. wake-word-       │
                    │    detector         │
                    └─────────┬───────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     │
┌───────────────┐   ┌─────────────────────┐        │
│ 2. listening- │   │ 3. listening-audio- │        │
│    state-     │   │    pipeline         │        │
│    machine    │   │ (depends: 1, 2)     │        │
└───────┬───────┘   └─────────┬───────────┘        │
        │                     │                    │
        │     ┌───────────────┤                    │
        │     │               │                    │
        │     ▼               ▼                    │
        │  ┌──────────┐  ┌────────────────┐        │
        │  │ 5. auto- │  │ 4. activation- │        │
        │  │    stop- │  │    feedback    │        │
        │  │    det.  │  │ (depends: 2,3) │        │
        │  │(dep: 3)  │  └────────────────┘        │
        │  └────┬─────┘                            │
        │       │                                  │
        │       └──────────────┬───────────────────┤
        │                      │                   │
        │                      ▼                   │
        │           ┌──────────────────┐           │
        │           │ 6. cancel-       │           │
        │           │    commands      │           │
        │           │ (depends: 1, 5)  │           │
        │           └──────────────────┘           │
        │                                          │
        └──────────────────────┬───────────────────┘
                               │
                               ▼
                    ┌──────────────────┐
                    │ 7. frontend-     │
                    │    listening-    │
                    │    hook          │
                    │ (depends: 2, 4)  │
                    └─────────┬────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │ 8. settings-     │
                    │    persistence   │
                    │ (depends: 7)     │
                    └──────────────────┘
```

### Recommended Implementation Order

| Order | Spec | Rationale |
|-------|------|-----------|
| 1 | `wake-word-detector` | Core detection component, no dependencies, enables testing in isolation |
| 2 | `listening-state-machine` | State management foundation, no dependencies, enables frontend work |
| 3 | `listening-audio-pipeline` | Connects detector to audio, requires 1 & 2 |
| 4 | `auto-stop-detection` | Silence detection needed before feedback makes sense, requires 3 |
| 5 | `activation-feedback` | Visual feedback, requires working pipeline (2, 3) |
| 6 | `cancel-commands` | Builds on wake word detector + silence detection, requires 1, 5 |
| 7 | `frontend-listening-hook` | React integration, requires backend (2, 4) |
| 8 | `settings-persistence` | Final polish, requires frontend hook (7) |

**Parallelization opportunity**: Specs 1 and 2 can be developed in parallel.

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Extend RecordingState enum (not separate state machine) | Simpler coordination, single source of truth for audio state | 2025-12-14 |
| Use Parakeet for wake word detection | Reuse existing model, on-device privacy, no additional dependencies | 2025-12-14 |
| Circular buffer separate from recording buffer | Bounded memory, wake word needs rolling window not accumulation | 2025-12-14 |
| Event-driven frontend updates | Consistent with existing useRecording pattern | 2025-12-14 |
| Unified `listening/` module | Simpler organization than separate wakeword/ and vad/ modules | 2025-12-14 |
| Simple energy-based silence detection | No external dependencies (avoid webrtc-vad), MVP simplicity | 2025-12-14 |
| Visual feedback only (no audio) | Reuse existing CatOverlay, avoid audio playback complexity | 2025-12-14 |
| Use tauri-plugin-store for settings | Official Tauri v2 plugin, proper persistence pattern | 2025-12-14 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-14 | AudioThreadHandle uses dedicated thread with mpsc commands | Can add listening commands without major refactor |
| 2025-12-14 | RecordingManager has states: Idle, Recording, Processing | Need to add Listening state, handle transitions |
| 2025-12-14 | Parakeet TDT is batch model, not streaming | Will use small-window batching for wake word detection |
| 2025-12-14 | Hotkey integration has debouncing logic | Need to coordinate with listening mode activation |
| 2025-12-14 | tauri-plugin-store not currently installed | Must add to Cargo.toml and capabilities |

## Open Questions

- [x] Can Parakeet handle streaming detection efficiently? → Will use small-window batching (~1-2s chunks)
- [ ] What's the optimal confidence threshold for wake word detection?
- [ ] Should listening mode auto-disable when app loses focus?
- [ ] How to handle multiple microphones / device switching?

## Dependencies to Add

```toml
# src-tauri/Cargo.toml
tauri-plugin-store = "2"
```

```json
// src-tauri/capabilities/default.json - add to permissions array
"store:default"
```

## Files to Modify

### New Files
- `src-tauri/src/listening/mod.rs` - ListeningManager, module exports
- `src-tauri/src/listening/detector.rs` - WakeWordDetector (Parakeet-based)
- `src-tauri/src/listening/silence.rs` - Simple energy-based silence detection
- `src-tauri/src/listening/buffer.rs` - CircularBuffer for audio window
- `src/hooks/useListening.ts` - Frontend listening hook

### Modified Files
- `src-tauri/Cargo.toml` - Add tauri-plugin-store dependency
- `src-tauri/capabilities/default.json` - Add store permissions
- `src-tauri/src/recording/state.rs` - Add Listening state
- `src-tauri/src/audio/thread.rs` - Add continuous capture mode
- `src-tauri/src/commands/mod.rs` - Add listening commands
- `src-tauri/src/events.rs` - Add listening events
- `src-tauri/src/lib.rs` - Register new state managers, init store plugin
- `src-tauri/src/hotkey/integration.rs` - Coordinate with listening
- `src/components/CatOverlay.tsx` - Listening indicator
- `src/App.tsx` - Integrate useListening hook

## References

- [Parakeet-rs crate](https://crates.io/crates/parakeet-rs) - On-device ASR
- [cpal](https://crates.io/crates/cpal) - Cross-platform audio
- [tauri-plugin-store](https://v2.tauri.app/plugin/store/) - Settings persistence
- Existing code: `src-tauri/src/recording/state.rs`, `src-tauri/src/audio/thread.rs`

---
last-updated: 2025-12-12
status: active
---

# Technical Guidance: ai-transcription

## Architecture Overview

### Three-Layer Architecture (Following existing patterns)

```
LAYER 3: Frontend (React/TypeScript)
├── useTranscription hook - manages transcription state, listens to events
├── useModelStatus hook - tracks model download/availability
├── ModelDownloadButton component - UI for model download
├── TranscriptionIndicator component - loading state during transcription
└── Notifications - success/error feedback
        ↕ Events / IPC (Tauri)
LAYER 2: IPC/Integration
├── TranscriptionManager - coordinates transcription workflow
├── Commands - download_model, check_model_status, get_transcription_state
└── Events - transcription_started, transcription_completed, transcription_error
        ↕
LAYER 1: Backend Core (Rust)
├── whisper/ module - WhisperContext wrapper, model loading
├── model/ module - download manager, file storage
└── Audio pipeline modification - 16kHz recording
```

### Audio Flow (Recording → Transcription → Clipboard)

```
1. User triggers recording (existing hotkey)
2. Audio captured at 16kHz (modified from 48kHz)
3. Recording stops → WAV saved
4. Transcription auto-starts:
   - Load audio samples from buffer (get_last_recording_buffer)
   - Pass to WhisperContext::transcribe()
   - Emit transcription_completed with text
5. Copy text to clipboard
6. Show success notification
```

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Record at 16kHz | Eliminates resampling complexity; 16kHz is Whisper's native rate | 2025-12-12 |
| Eager model loading | User prioritizes instant transcription over startup time | 2025-12-12 |
| Large v3 Turbo model | Best accuracy per feature.md specification (1.5GB) | 2025-12-12 |
| whisper-rs crate | Rust bindings to whisper.cpp, well-maintained, MIT licensed | 2025-12-12 |
| Mutex-wrapped context | whisper.cpp is not thread-safe; Mutex ensures sequential access | 2025-12-12 |
| Trait-based design | Enables testing with mocks (TranscriptionService trait) | 2025-12-12 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-12 | VoiceInk uses Swift actor pattern for thread safety | Use Mutex wrapper in Rust for same guarantee |
| 2025-12-12 | cpal uses device default sample rate | Need to request 16kHz config or add resampling fallback |
| 2025-12-12 | get_last_recording_buffer() already returns AudioData | Direct integration point for transcription pipeline |

## Open Questions

- [x] Sample rate: Record at 16kHz (decided)
- [x] Model loading: Eager at startup (decided)
- [x] Model variant: Large v3 Turbo (decided)
- [x] Download progress UI: Just "downloading..." state per MVP scope

## Files to Modify

### Backend (Rust)
| File | Purpose |
|------|---------|
| `src-tauri/Cargo.toml` | Add `whisper-rs`, `reqwest`, `rubato` dependencies |
| `src-tauri/src/lib.rs` | Register transcription commands, manage state |
| `src-tauri/src/whisper/mod.rs` | NEW: Whisper context wrapper |
| `src-tauri/src/whisper/context.rs` | NEW: WhisperContext implementation |
| `src-tauri/src/model/mod.rs` | NEW: Model download manager |
| `src-tauri/src/model/download.rs` | NEW: HTTP download with progress |
| `src-tauri/src/transcription/mod.rs` | NEW: TranscriptionManager |
| `src-tauri/src/transcription/commands.rs` | NEW: Tauri command handlers |
| `src-tauri/src/audio/cpal_backend.rs` | Modify sample rate to 16kHz |
| `src-tauri/src/audio/wav.rs` | Update for 16kHz encoding |
| `src-tauri/src/events.rs` | Add transcription events |
| `src-tauri/src/recording/state.rs` | Integrate transcription after recording |

### Frontend (TypeScript/React)
| File | Purpose |
|------|---------|
| `src/hooks/useTranscription.ts` | NEW: Transcription state hook |
| `src/hooks/useModelStatus.ts` | NEW: Model availability hook |
| `src/components/ModelDownloadButton.tsx` | NEW: Download UI |
| `src/components/TranscriptionIndicator.tsx` | NEW: Loading indicator |
| `src/App.tsx` | Integrate transcription components |

## Technical Details

### Model Information
- **Model**: `ggml-large-v3-turbo.bin`
- **Size**: ~1.5 GB
- **Source**: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`
- **Storage**: `{app_data_dir}/heycat/models/`

### Dependencies to Add
```toml
# Cargo.toml
whisper-rs = "0.13"  # Rust bindings to whisper.cpp
reqwest = { version = "0.12", features = ["stream"] }  # HTTP downloads
rubato = "0.15"  # Audio resampling (if device doesn't support 16kHz)
```

### Thread Safety Pattern
```rust
// WhisperContext wrapper pattern
pub struct WhisperManager {
    context: Arc<Mutex<Option<WhisperContext>>>,
    model_path: PathBuf,
}

impl WhisperManager {
    pub fn transcribe(&self, samples: &[f32]) -> Result<String, Error> {
        let guard = self.context.lock().map_err(|_| Error::LockPoisoned)?;
        let ctx = guard.as_ref().ok_or(Error::ModelNotLoaded)?;
        ctx.full_transcribe(samples)
    }
}
```

### Event Names
```rust
pub mod transcription_events {
    pub const TRANSCRIPTION_STARTED: &str = "transcription_started";
    pub const TRANSCRIPTION_COMPLETED: &str = "transcription_completed";
    pub const TRANSCRIPTION_ERROR: &str = "transcription_error";
    pub const MODEL_DOWNLOAD_PROGRESS: &str = "model_download_progress";
    pub const MODEL_DOWNLOAD_COMPLETED: &str = "model_download_completed";
}
```

## References

- [whisper-rs crate](https://crates.io/crates/whisper-rs)
- [whisper.cpp models on Hugging Face](https://huggingface.co/ggerganov/whisper.cpp)
- VoiceInk reference: `/Users/michaelhindley/Documents/git/VoiceInk`
- Existing recording implementation: `src-tauri/src/recording/`
- Architecture docs: `docs/ARCHITECTURE.md`

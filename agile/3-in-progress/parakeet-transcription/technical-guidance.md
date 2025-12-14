---
last-updated: 2025-12-14
status: active
---

# Technical Guidance: Replace Whisper with Parakeet v3 + Streaming

## Architecture Overview

Replace the existing `whisper-rs` transcription layer with the `parakeet-rs` library (https://github.com/altunenes/parakeet-rs), a Rust library for NVIDIA Parakeet models via ONNX Runtime.

### Integration Point

Transcription integration occurs at `src-tauri/src/hotkey/integration.rs:251-320` in `spawn_transcription()`. The new flow:

```
Recording Stops
    → Get audio buffer from RecordingManager
    → TranscriptionManager.transcribe(samples)
        → Batch Mode (TDT): Process full audio → transcription_completed
        → Stream Mode (EOU): Real-time chunks during recording → transcription_partial events
    → Voice Command Matching (existing)
    → Clipboard Copy fallback (existing)
```

### Current vs Target Architecture

**Current:**
```
Audio capture (cpal) → 16kHz mono f32 → WhisperManager → transcription_completed event
```

**Target:**
```
Audio capture (cpal) → 16kHz mono f32 → TranscriptionManager
                                              ├── ParakeetTDT (batch) → transcription_completed
                                              └── ParakeetEOU (streaming) → transcription_partial events
```

### Module Structure

```
src-tauri/src/
├── parakeet/
│   ├── mod.rs              # Module exports, re-exports
│   ├── manager.rs          # TranscriptionManager (TDT + mode control)
│   └── streaming.rs        # StreamingTranscriber (EOU wrapper)
├── model/
│   ├── download.rs         # Multi-file manifest downloads
│   └── mod.rs              # Tauri commands with model_type param
├── audio/
│   └── cpal_backend.rs     # + Streaming channel support
├── events.rs               # + transcription_partial event
└── lib.rs                  # Replace WhisperManager init
```

### Key Components

1. **TranscriptionManager** - Unified manager owning both TDT and EOU instances, implements TranscriptionService trait
2. **ParakeetTDT** - Batch transcription, multilingual (25 languages), auto-detection
3. **ParakeetEOU** - Real-time streaming (160ms chunks = 2560 samples at 16kHz)
4. **StreamingTranscriber** - Receives audio chunks from callback via channel, emits partial events

### Audio Requirements
Both Whisper and Parakeet use identical audio format: **16kHz mono f32** - no audio pipeline changes needed.

### Integration with Existing Code

**Modify `lib.rs`:**
- Replace `WhisperManager` with `TranscriptionManager`
- Eager load both TDT and EOU models if available

**Modify `hotkey/integration.rs`:**
- Replace `with_whisper_manager()` with `with_transcription_manager()`
- Handle mode switching (batch vs streaming)

**New Tauri Commands:**
- `download_model(model_type)` - Download TDT or EOU model
- `check_model_status(model_type)` - Check if specific model exists
- `set_transcription_mode(mode)` - Switch batch/streaming
- `get_transcription_mode()` - Get current mode

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Use parakeet-rs (ONNX) over FluidAudio (CoreML) | FluidAudio is Swift-only; parakeet-rs is pure Rust with ONNX Runtime | 2025-12-13 |
| Directory-based model storage | Each model type has multiple ONNX files | 2025-12-13 |
| Preserve TranscriptionService trait | Maintains compatibility with existing batch mode integration | 2025-12-13 |
| Channel-based streaming | Audio callback sends chunks to StreamingTranscriber via channel | 2025-12-13 |
| Default to CPU execution | CoreML backend for ONNX Runtime noted as "not well optimized" on macOS | 2025-12-13 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-13 | VoiceInk uses FluidAudio Swift library for Parakeet | Cannot use directly - need Rust alternative |
| 2025-12-13 | parakeet-rs exists with TDT + EOU support | Primary library choice for implementation |
| 2025-12-13 | ONNX models available on HuggingFace | Model download URLs confirmed |
| 2025-12-13 | parakeet-rs "incredibly fast on Mac M3 CPU vs Whisper Metal" | Performance should be good on Apple Silicon |
| 2025-12-13 | EOU streaming uses 160ms chunks (2560 samples at 16kHz) | Chunk size for audio callback integration |

## Open Questions

- [x] Which Rust library to use? → parakeet-rs
- [x] Support both models or one? → Both TDT (batch) and EOU (streaming)
- [x] Replace Whisper or add alongside? → Replace entirely
- [ ] Exact HuggingFace URLs for v3 ONNX models - may need to verify/test

## Files to Modify

### Rust Backend (src-tauri/src/)

**Create:**
- `parakeet/mod.rs` - Module definition
- `parakeet/manager.rs` - TranscriptionManager + TranscriptionService impl
- `parakeet/streaming.rs` - StreamingTranscriber for EOU mode

**Modify:**
- `Cargo.toml` - Swap `whisper-rs` for `parakeet-rs`
- `lib.rs` - Replace module import and manager initialization
- `model/download.rs` - Multi-file download support, model manifests
- `model/mod.rs` - Update Tauri commands for model type
- `hotkey/integration.rs` - Update to use TranscriptionManager
- `audio/cpal_backend.rs` - Add streaming channel to callback
- `events.rs` - Add `transcription_partial` event

**Delete:**
- `whisper/mod.rs`
- `whisper/context.rs`

### Frontend (src/)

**Create:**
- `components/Settings/TranscriptionSettings.tsx` - Mode and model selection

**Modify:**
- `hooks/useModelStatus.ts` - Multi-model support
- `hooks/useTranscription.ts` - Partial text for streaming
- `components/ModelDownloadButton.tsx` - Model type selection

### Model Storage Structure

```
{app_data_dir}/heycat/models/
├── parakeet-tdt/
│   ├── encoder-model.onnx
│   ├── encoder-model.onnx.data
│   ├── decoder_joint-model.onnx
│   └── vocab.txt
└── parakeet-eou/
    ├── encoder.onnx
    ├── decoder_joint.onnx
    └── tokenizer.json
```

## References

### Libraries
- [parakeet-rs GitHub](https://github.com/altunenes/parakeet-rs) - Rust library for Parakeet ONNX inference
- [parakeet.cpp](https://github.com/jason-ni/parakeet.cpp) - Alternative C++ implementation

### Models
- [istupakov/parakeet-tdt-0.6b-v2-onnx](https://huggingface.co/istupakov/parakeet-tdt-0.6b-v2-onnx) - TDT v2 ONNX model
- [nvidia/parakeet-tdt-0.6b-v3](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3) - Official v3 model (needs ONNX conversion)
- [FluidInference/parakeet-tdt-0.6b-v3-coreml](https://huggingface.co/FluidInference/parakeet-tdt-0.6b-v3-coreml) - CoreML version (reference)

### Reference Implementation
- `/Users/michaelhindley/Documents/git/VoiceInk/` - VoiceInk app using FluidAudio (Swift)

## API Reference

### parakeet-rs TDT (Batch)
```rust
use parakeet_rs::ParakeetTDT;
let mut parakeet = ParakeetTDT::from_pretrained("./tdt", None)?;
let result = parakeet.transcribe_samples(audio, 16000, 1)?; // 16kHz mono
println!("{}", result.text);
```

### parakeet-rs EOU (Streaming)
```rust
use parakeet_rs::ParakeetEOU;
let mut parakeet = ParakeetEOU::from_pretrained("./eou", None)?;
const CHUNK_SIZE: usize = 2560; // 160ms at 16kHz
for chunk in audio.chunks(CHUNK_SIZE) {
    let text = parakeet.transcribe(chunk, false)?;
    print!("{}", text);
}
// Final chunk with is_final=true
let final_text = parakeet.transcribe(final_chunk, true)?;
```

### Cargo Dependency
```toml
parakeet-rs = "0.2"
# Optional features: sortformer (diarization), cuda, tensorrt, etc.
```

## Spec Dependency Graph

```
Spec 1 (skeleton) ──┬──> Spec 2 (download) ──> Spec 7 (frontend)
                    │         │
                    │         v
                    ├──> Spec 5 (TDT) ─────┐
                    │                      │
                    ├──> Spec 3 (audio) ───┤
                    │         │            │
                    │         v            v
                    └──> Spec 4 (EOU) ──> Spec 8 (wire-up) ──> Spec 6 (cleanup)
```

### Spec Files

| # | File | Title |
|---|------|-------|
| 1 | `parakeet-module-skeleton.spec.md` | Create Parakeet module skeleton |
| 2 | `multi-file-model-download.spec.md` | Multi-file ONNX model download |
| 3 | `streaming-audio-integration.spec.md` | Streaming audio pipeline integration |
| 4 | `eou-streaming-transcription.spec.md` | Implement EOU streaming transcription |
| 5 | `tdt-batch-transcription.spec.md` | Implement TDT batch transcription |
| 6 | `cleanup-whisper.spec.md` | Remove Whisper dependency |
| 7 | `frontend-model-settings.spec.md` | Frontend model and mode settings UI |
| 8 | `wire-up-transcription.spec.md` | Wire up TranscriptionManager |

### Implementation Order

1. `parakeet-module-skeleton` - No dependencies (foundational)
2. `multi-file-model-download` + `streaming-audio-integration` - Depend on spec 1, can run in parallel
3. `tdt-batch-transcription` + `eou-streaming-transcription` - Depend on specs 1-3
4. `frontend-model-settings` + `wire-up-transcription` - Depend on specs 2, 4-5
5. `cleanup-whisper` - **Must be LAST** - depends on all other specs working

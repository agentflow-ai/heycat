---
last-updated: 2025-12-16
status: draft
---

# Technical Guidance: Audio Processing Architecture Technical Debt

## Root Cause Analysis

The audio processing architecture evolved organically as features were added incrementally:

1. **TranscriptionManager** was created first for batch transcription with proper trait abstraction
2. **WakeWordDetector** was added later for always-on listening, but developers bypassed the existing TranscriptionService trait and created a second ParakeetTDT instance for "isolation"
3. **Callback pattern** was used for communication but without considering thread safety (callbacks run on analysis thread while holding locks)
4. **VAD thresholds** were tuned independently by different developers without coordination
5. **No shared infrastructure** for common operations (VAD init, token workarounds)

The root cause is **lack of upfront architectural planning** for multi-component audio processing, leading to:
- Duplication of expensive resources (models)
- Inconsistent abstractions (trait used sometimes, bypassed other times)
- Unsafe inter-thread communication patterns

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Create SharedTranscriptionModel | Single 3GB model instead of two; memory from ~6GB to ~3GB | 2025-12-15 |
| Use async event channel | Move callbacks off analysis thread to prevent deadlocks | 2025-12-15 |
| Unify VAD config | Single source of truth prevents threshold drift | 2025-12-15 |
| Add 60s transcription timeout | Prevent indefinite hangs with graceful recovery | 2025-12-15 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-15 | Two ParakeetTDT instances in `manager.rs:14` and `detector.rs:165` | ~3GB wasted memory |
| 2025-12-15 | Callback at `pipeline.rs:474-477` runs on analysis thread | Deadlock risk if callback acquires locks |
| 2025-12-15 | Token workaround duplicated at `manager.rs:136-143` and `detector.rs:353-355` | Maintenance burden |
| 2025-12-15 | VAD threshold 0.3 in detector vs 0.5 in silence | Behavioral inconsistency |
| 2025-12-15 | State set to Transcribing at `manager.rs:112-122` before operation | Race condition window |

## Open Questions

- [ ] Should VAD threshold be 0.3, 0.5, or configurable? Need to understand the rationale for current values
- [ ] Is 60s the right transcription timeout? What's the longest expected transcription?
- [ ] Can we share the ParakeetTDT model between streaming (wake word) and batch (transcription) use cases, or do they need separate configurations?

## Implementation Approach

### Phase 1: Critical Fixes (Memory & Safety)

**Step 1.1: Create SharedTranscriptionModel**
```rust
// src-tauri/src/parakeet/shared.rs
pub struct SharedTranscriptionModel {
    model: Arc<Mutex<Option<ParakeetTDT>>>,
    sample_rate: u32,
}

impl SharedTranscriptionModel {
    pub fn load(&self, path: &Path) -> Result<()>;
    pub fn transcribe_file(&self, path: &Path) -> Result<String>;
    pub fn transcribe_samples(&self, samples: &[f32], ...) -> Result<String>;
}
```

**Step 1.2: Fix Callback Safety**
```rust
// Replace direct callback with channel
let (wake_word_tx, wake_word_rx) = tokio::sync::mpsc::channel(10);

// In analysis thread: send event instead of calling callback
wake_word_tx.send(WakeWordEvent::Detected { ... }).await;

// In HotkeyIntegration: subscribe to channel
while let Some(event) = wake_word_rx.recv().await {
    handle_wake_word(event);
}
```

### Phase 2: Code Consolidation

**Step 2.1: Unified VadConfig**
```rust
// src-tauri/src/listening/vad.rs
pub struct VadConfig {
    pub speech_threshold: f32,  // Document: 0.4 is good balance
    pub sample_rate: u32,
    pub chunk_size: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            speech_threshold: 0.4,  // Unified threshold
            sample_rate: 16000,
            chunk_size: 512,
        }
    }
}
```

**Step 2.2: Extract Token Workaround**
```rust
// src-tauri/src/parakeet/utils.rs
pub fn fix_parakeet_text(tokens: &[Token]) -> String {
    tokens.iter().map(|t| t.text.as_str()).collect::<String>().trim().to_string()
}
```

### Phase 3: Robustness

**Step 3.1: Transcription Timeout**
```rust
// Wrap transcription with timeout
let result = tokio::time::timeout(
    Duration::from_secs(60),
    tokio::task::spawn_blocking(move || transcriber.transcribe(&path))
).await;

match result {
    Ok(Ok(text)) => /* success */,
    Ok(Err(e)) => /* transcription error */,
    Err(_) => /* timeout - emit error, reset state */,
}
```

## Files to Modify

| File | Purpose |
|------|---------|
| `src-tauri/src/parakeet/mod.rs` | Add shared model exports |
| `src-tauri/src/parakeet/shared.rs` | New SharedTranscriptionModel |
| `src-tauri/src/parakeet/manager.rs` | Use shared model, fix state race |
| `src-tauri/src/listening/detector.rs` | Accept shared model, use trait |
| `src-tauri/src/listening/pipeline.rs` | Replace callback with channel |
| `src-tauri/src/listening/vad.rs` | New unified VadConfig |
| `src-tauri/src/listening/silence.rs` | Use VadConfig |
| `src-tauri/src/hotkey/integration.rs` | Channel subscription, timeout |
| `src-tauri/src/lib.rs` | Initialize shared model |

## References

- [Parakeet-rs documentation](https://github.com/nvidia-riva/parakeet)
- [Silero VAD](https://github.com/snakers4/silero-vad)
- [Tokio channels](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html)

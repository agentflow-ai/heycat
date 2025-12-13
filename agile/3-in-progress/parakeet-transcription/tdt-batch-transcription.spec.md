---
status: completed
created: 2025-12-13
completed: 2025-12-13
dependencies: ["parakeet-module-skeleton.spec.md", "multi-file-model-download.spec.md"]
review_round: 1
---

# Spec: Implement TDT batch transcription

## Description

Implement a `TranscriptionManager` struct that wraps `parakeet_rs::ParakeetTDT` for batch transcription, replacing the existing `WhisperManager`. The manager implements the existing `TranscriptionService` trait to maintain API compatibility, enabling a seamless swap from Whisper to Parakeet TDT.

TDT (Token-and-Duration Transducer) supports 25 European languages with automatic language detection, providing improved accuracy and speed compared to Whisper on Apple Silicon.

## Acceptance Criteria

- [ ] `TranscriptionManager` struct created in `src-tauri/src/parakeet/manager.rs`
- [ ] Implements `TranscriptionService` trait (same interface as WhisperManager)
- [ ] `load_model(path)` loads TDT model from directory (not single file)
- [ ] `transcribe(samples)` processes 16kHz mono f32 audio and returns text
- [ ] Supports multilingual transcription with auto-detection (no language parameter needed)
- [ ] Uses `Arc<Mutex<Option<ParakeetTDT>>>` pattern for thread-safe model access
- [ ] State machine follows same flow: Unloaded -> Idle -> Transcribing -> Completed/Error -> Idle
- [ ] All existing `TranscriptionError` variants are properly mapped

## Test Cases

- [ ] `test_transcription_manager_new_is_unloaded` - New manager starts in Unloaded state
- [ ] `test_transcription_manager_default_is_unloaded` - Default trait implementation returns unloaded
- [ ] `test_transcription_manager_load_model_invalid_path` - Load from nonexistent directory returns ModelLoadFailed
- [ ] `test_transcription_manager_transcribe_not_loaded` - Transcribe without model returns ModelNotLoaded
- [ ] `test_transcription_manager_transcribe_empty_audio` - Empty samples returns InvalidAudio
- [ ] `test_transcription_manager_reset_to_idle_from_completed` - Reset from Completed returns to Idle
- [ ] `test_transcription_manager_reset_to_idle_from_error` - Reset from Error returns to Idle
- [ ] `test_transcription_manager_state_transitions` - Verify state transitions during transcription

## Dependencies

- `parakeet-module-skeleton.spec.md` - Module structure and shared types
- `multi-file-model-download.spec.md` - TDT model files available in directory

## Preconditions

- `parakeet-rs` crate added to `Cargo.toml` with version `0.2`
- TDT model files downloaded to `{app_data_dir}/heycat/models/parakeet-tdt/`
- Directory contains: `encoder-model.onnx`, `encoder-model.onnx.data`, `decoder_joint-model.onnx`, `vocab.txt`

## Implementation Notes

### Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `src-tauri/src/parakeet/manager.rs` | Create | TranscriptionManager implementation |
| `src-tauri/src/parakeet/mod.rs` | Modify | Export manager, re-export TranscriptionService trait |
| `src-tauri/src/lib.rs` | Modify | Replace `whisper` import with `parakeet` |
| `src-tauri/src/hotkey/integration.rs` | Modify | Replace `WhisperManager` with `TranscriptionManager` |
| `src-tauri/Cargo.toml` | Modify | Remove `whisper-rs`, add `parakeet-rs = "0.2"` |

### Struct Design (mirrors WhisperManager pattern)

```rust
use parakeet_rs::ParakeetTDT;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Thread-safe wrapper around ParakeetTDT
/// Uses Mutex to serialize access since ONNX Runtime is not thread-safe for inference
pub struct TranscriptionManager {
    tdt: Arc<Mutex<Option<ParakeetTDT>>>,
    state: Arc<Mutex<TranscriptionState>>,
}

impl Default for TranscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptionManager {
    pub fn new() -> Self {
        Self {
            tdt: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(TranscriptionState::Unloaded)),
        }
    }
}

impl TranscriptionService for TranscriptionManager {
    fn load_model(&self, path: &Path) -> TranscriptionResult<()> {
        // ParakeetTDT::from_pretrained expects directory path, not file
        let tdt = ParakeetTDT::from_pretrained(
            path.to_str().ok_or_else(|| TranscriptionError::ModelLoadFailed("Invalid path".into()))?,
            None, // Use default ONNX options
        ).map_err(|e| TranscriptionError::ModelLoadFailed(e.to_string()))?;

        // Store and update state
        *self.tdt.lock().map_err(|_| TranscriptionError::LockPoisoned)? = Some(tdt);
        *self.state.lock().map_err(|_| TranscriptionError::LockPoisoned)? = TranscriptionState::Idle;
        Ok(())
    }

    fn transcribe(&self, samples: &[f32]) -> TranscriptionResult<String> {
        // Validate input
        if samples.is_empty() {
            return Err(TranscriptionError::InvalidAudio("Empty audio buffer".into()));
        }

        // Update state to Transcribing
        {
            let mut state = self.state.lock().map_err(|_| TranscriptionError::LockPoisoned)?;
            if *state == TranscriptionState::Unloaded {
                return Err(TranscriptionError::ModelNotLoaded);
            }
            *state = TranscriptionState::Transcribing;
        }

        // Perform transcription
        let result = {
            let mut guard = self.tdt.lock().map_err(|_| TranscriptionError::LockPoisoned)?;
            let tdt = guard.as_mut().ok_or(TranscriptionError::ModelNotLoaded)?;

            // transcribe_samples(audio, sample_rate, channels)
            tdt.transcribe_samples(samples, 16000, 1)
                .map(|r| r.text)
                .map_err(|e| TranscriptionError::TranscriptionFailed(e.to_string()))
        };

        // Update state based on result
        {
            let mut state = self.state.lock().map_err(|_| TranscriptionError::LockPoisoned)?;
            *state = if result.is_ok() { TranscriptionState::Completed } else { TranscriptionState::Error };
        }

        result
    }

    fn is_loaded(&self) -> bool {
        self.tdt.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    fn state(&self) -> TranscriptionState {
        self.state.lock().map(|g| *g).unwrap_or(TranscriptionState::Unloaded)
    }

    fn reset_to_idle(&self) -> TranscriptionResult<()> {
        let mut state = self.state.lock().map_err(|_| TranscriptionError::LockPoisoned)?;
        if *state == TranscriptionState::Completed || *state == TranscriptionState::Error {
            *state = TranscriptionState::Idle;
        }
        Ok(())
    }
}
```

### Key API Differences from Whisper

| Whisper | Parakeet TDT |
|---------|--------------|
| Single `.bin` file | Directory with multiple ONNX files |
| `WhisperContext::new_with_params(path, params)` | `ParakeetTDT::from_pretrained(dir, options)` |
| Segments-based output | Direct `result.text` string |
| Language parameter optional | Auto-detection built-in |

### Integration with lib.rs

```rust
// Before (whisper)
mod whisper;
use whisper::{TranscriptionService, WhisperManager};
let whisper_manager = Arc::new(WhisperManager::new());

// After (parakeet)
mod parakeet;
use parakeet::{TranscriptionService, TranscriptionManager};
let transcription_manager = Arc::new(TranscriptionManager::new());
```

## Related Specs

- `parakeet-module-skeleton.spec.md` - Module setup
- `multi-file-model-download.spec.md` - Model download support
- `eou-streaming-transcription.spec.md` - Streaming alternative
- `wire-up-transcription.spec.md` - Integration with HotkeyIntegration
- `cleanup-whisper.spec.md` - Remove old whisper code

## Integration Points

- Production call site: `src-tauri/src/lib.rs` - instantiate TranscriptionManager
- Production call site: `src-tauri/src/hotkey/integration.rs:93` - `with_whisper_manager()` becomes `with_transcription_manager()`
- Connects to: `model/mod.rs` (model path), `events.rs` (transcription events)

## Integration Test

- Test location: `src-tauri/src/parakeet/manager_test.rs`
- Verification: [ ] Integration test passes
- Test approach: With downloaded model, verify load_model succeeds and transcribe returns text for sample audio. Mock audio can be generated or use a small test file.

## Review

**Reviewed:** 2025-12-13
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `TranscriptionManager` struct created in `src-tauri/src/parakeet/manager.rs` | PASS | `src-tauri/src/parakeet/manager.rs:11-16` - struct defined with `tdt_context` and `state` fields |
| Implements `TranscriptionService` trait (same interface as WhisperManager) | PASS | `src-tauri/src/parakeet/manager.rs:42-146` - full implementation of `TranscriptionService` trait |
| `load_model(path)` loads TDT model from directory (not single file) | PASS | `src-tauri/src/parakeet/manager.rs:43-69` - uses `ParakeetTDT::from_pretrained(path)` which expects directory |
| `transcribe(samples)` processes 16kHz mono f32 audio and returns text | PASS | `src-tauri/src/parakeet/manager.rs:71-121` - calls `transcribe_samples(samples.to_vec(), 16000, 1, None)` |
| Supports multilingual transcription with auto-detection (no language parameter needed) | PASS | `src-tauri/src/parakeet/manager.rs:102` - no language parameter passed to `transcribe_samples`, TDT auto-detects |
| Uses `Arc<Mutex<Option<ParakeetTDT>>>` pattern for thread-safe model access | PASS | `src-tauri/src/parakeet/manager.rs:13` - `tdt_context: Arc<Mutex<Option<ParakeetTDT>>>` |
| State machine follows same flow: Unloaded -> Idle -> Transcribing -> Completed/Error -> Idle | PASS | `src-tauri/src/parakeet/manager.rs:65,88,113-117,141-143` - state transitions implemented correctly |
| All existing `TranscriptionError` variants are properly mapped | PASS | `src-tauri/src/parakeet/manager.rs:47,50,57,74-76,84-86,96,104` - all variants used: `ModelLoadFailed`, `LockPoisoned`, `InvalidAudio`, `ModelNotLoaded`, `TranscriptionFailed` |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `test_transcription_manager_new_is_unloaded` | PASS | `src-tauri/src/parakeet/manager.rs:153-157` |
| `test_transcription_manager_default_is_unloaded` | PASS | `src-tauri/src/parakeet/manager.rs:159-164` |
| `test_transcription_manager_load_model_invalid_path` | PASS | `src-tauri/src/parakeet/manager.rs:184-190` (named `test_load_model_fails_with_invalid_path`) |
| `test_transcription_manager_transcribe_not_loaded` | PASS | `src-tauri/src/parakeet/manager.rs:166-173` (named `test_transcribe_returns_error_when_model_not_loaded`) |
| `test_transcription_manager_transcribe_empty_audio` | PASS | `src-tauri/src/parakeet/manager.rs:175-182` (named `test_transcribe_returns_error_for_empty_audio`) |
| `test_transcription_manager_reset_to_idle_from_completed` | PASS | `src-tauri/src/parakeet/manager.rs:192-204` |
| `test_transcription_manager_reset_to_idle_from_error` | PASS | `src-tauri/src/parakeet/manager.rs:206-218` |
| `test_transcription_manager_state_transitions` | PASS | `src-tauri/src/parakeet/manager.rs:243-285` (named `test_transcription_manager_state_transitions`) |

### Code Quality

**Strengths:**
- Clean implementation following the established pattern from WhisperManager
- Proper thread-safe access using `Arc<Mutex<Option<>>>` pattern
- Comprehensive error handling with proper mapping to `TranscriptionError` variants
- Good separation of concerns between state management and model operations
- All trait methods properly implemented with `Send + Sync` bounds satisfied
- Additional bonus tests included (`test_reset_to_idle_noop_from_idle`, `test_reset_to_idle_noop_from_unloaded`)

**Concerns:**
- None identified

### Verdict

**APPROVED** - The implementation correctly satisfies all acceptance criteria. The `TranscriptionManager` is properly integrated into `lib.rs:72` and `hotkey/integration.rs:93` (via `with_transcription_manager`). The `parakeet-rs` dependency is added in `Cargo.toml:37`. All required test cases are present with meaningful assertions, and the code follows project patterns.

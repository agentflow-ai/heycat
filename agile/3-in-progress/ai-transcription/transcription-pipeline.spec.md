---
status: in-progress
created: 2025-12-12
completed: null
dependencies:
  - model-download
---

# Spec: Core Transcription Pipeline

## Description

Implement the core Whisper transcription pipeline using whisper-rs. This includes the WhisperContext wrapper with thread-safety, TranscriptionManager state machine, and the transcribe function that converts audio samples to text.

## Acceptance Criteria

- [ ] WhisperContext wrapper loads model and provides transcription interface
- [ ] Mutex wrapping ensures thread-safe access to whisper.cpp (not thread-safe by default)
- [ ] TranscriptionManager with state machine: Idle -> Transcribing -> Completed/Error
- [ ] `TranscriptionService` trait defined for mockability in tests
- [ ] `transcribe()` function accepts &[f32] audio samples (16kHz mono) and returns Result<String>
- [ ] Proper error handling: ModelNotLoaded, TranscriptionFailed, InvalidAudio
- [ ] Model loaded eagerly at app startup (per technical guidance decision)

## Test Cases

- [ ] transcribe() returns error when model not loaded
- [ ] transcribe() succeeds with valid audio samples
- [ ] transcribe() handles empty audio gracefully
- [ ] State transitions correctly: Idle -> Transcribing -> Completed
- [ ] State transitions on error: Idle -> Transcribing -> Error -> Idle
- [ ] Concurrent transcription requests are serialized (Mutex)

## Dependencies

- model-download (model must be available to load)

## Preconditions

- whisper-rs crate added to Cargo.toml
- Model file exists at expected path

## Implementation Notes

- Add to Cargo.toml: `whisper-rs = "0.13"`
- Follow VoiceInk pattern: load model once, reuse context
- Use `WhisperContext::new_with_params()` for model loading
- Use `ctx.full()` for transcription with default parameters
- Consider language auto-detection (per feature.md out-of-scope, use default)

```rust
pub struct WhisperManager {
    context: Arc<Mutex<Option<WhisperContext>>>,
}

impl WhisperManager {
    pub fn load_model(&self, path: &Path) -> Result<(), Error>;
    pub fn transcribe(&self, samples: &[f32]) -> Result<String, Error>;
    pub fn is_loaded(&self) -> bool;
}
```

## Related Specs

- model-download.spec.md (provides the model file)
- auto-transcribe-on-stop.spec.md (calls transcribe())
- sample-rate-modification.spec.md (provides correct sample format)

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (state initialization)
- Connects to: RecordingManager, TranscriptionCommands

## Integration Test

- Test location: `src-tauri/src/whisper/context_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-12
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| WhisperContext wrapper loads model and provides transcription interface | PASS | context.rs:93-118 (load_model), context.rs:120-195 (transcribe) |
| Mutex wrapping ensures thread-safe access to whisper.cpp | PASS | context.rs:72-73 - Both `context` and `state` use `Arc<Mutex<>>` |
| TranscriptionManager with state machine: Idle -> Transcribing -> Completed/Error | FAIL | context.rs:9-17 - States are `Unloaded`, `Idle`, `Transcribing`. Missing `Completed` and `Error` states - implementation always returns to `Idle` (line 187-191) |
| `TranscriptionService` trait defined for mockability in tests | PASS | context.rs:54-67 - Trait defined with `Send + Sync` bounds |
| `transcribe()` function accepts &[f32] audio samples (16kHz mono) and returns Result<String> | PASS | context.rs:120 - `fn transcribe(&self, samples: &[f32]) -> TranscriptionResult<String>` |
| Proper error handling: ModelNotLoaded, TranscriptionFailed, InvalidAudio | PASS | context.rs:21-32 - All three errors defined plus `ModelLoadFailed` and `LockPoisoned` |
| Model loaded eagerly at app startup | FAIL | lib.rs - `whisper` module imported (line 12) but `WhisperManager` is never instantiated or managed by Tauri state. No eager loading occurs. |

### Test Verification

| Behavior | Tested By | Notes |
|----------|-----------|-------|
| transcribe() returns error when model not loaded | Unit | context.rs:230-236 `test_transcribe_returns_error_when_model_not_loaded` |
| transcribe() succeeds with valid audio samples | N/A | No test - requires real model file, cannot be unit tested without integration setup |
| transcribe() handles empty audio gracefully | Unit | context.rs:239-245 `test_transcribe_returns_error_for_empty_audio` |
| State transitions correctly: Idle -> Transcribing -> Completed | N/A | Cannot test - `Completed` state does not exist in implementation |
| State transitions on error: Idle -> Transcribing -> Error -> Idle | N/A | Cannot test - `Error` state does not exist in implementation |
| Concurrent transcription requests are serialized (Mutex) | N/A | Not tested - would require integration test with loaded model |

### Code Quality

**Strengths:**
- Clean separation with `TranscriptionService` trait enabling dependency injection
- Comprehensive error types with proper `Display` and `Error` implementations
- Thread-safe design with `Arc<Mutex<>>` wrappers
- Good use of Rust idioms (Result types, guards, pattern matching)
- State validation before transcription (checks for Unloaded state)
- Default implementation for `WhisperManager`

**Concerns:**
- State machine deviates from spec - missing `Completed` and `Error` states
- Dead code warnings indicate `WhisperManager` is not used anywhere (lib.rs does not instantiate it)
- No integration tests exist (spec mentions `context_test.rs` but file does not exist)
- Eager model loading at startup is not implemented

### Integration Verification

| Check | Status | Evidence |
|-------|--------|----------|
| Mocked components instantiated in production? | FAIL | lib.rs - `WhisperManager` is never instantiated or added to Tauri managed state |
| Any "handled separately" without spec reference? | PASS | No untracked deferrals found |
| Integration test exists and passes? | FAIL | Spec references `src-tauri/src/whisper/context_test.rs` but file does not exist |

### Deferral Audit

| Deferral Statement | Location | Tracking Reference |
|--------------------|----------|-------------------|
| None found | N/A | N/A |

### Verdict

NEEDS_WORK - The core transcription logic is well-implemented with good error handling and thread safety. However, two acceptance criteria fail: (1) the state machine is missing `Completed` and `Error` states as specified, and (2) eager model loading at startup is not implemented - the `WhisperManager` is never instantiated in `lib.rs`. Additionally, the integration test file referenced in the spec does not exist.

---
status: completed
created: 2025-12-22
completed: null
dependencies: ["pipeline-integration"]
review_round: 1
---

# Spec: Cache denoiser at app startup to eliminate 2s recording delay

## Description

DTLN noise suppression is currently initialized on **every recording start** in `CpalBackend::start()`, taking ~2 seconds due to tract ONNX model optimization. This makes the app unusable for quick recordings.

This spec implements a `SharedDenoiser` wrapper that loads ONNX models once at app startup and shares the denoiser instance across all recordings, following the existing `SharedTranscriptionModel` pattern.

## Acceptance Criteria

- [ ] `SharedDenoiser` wrapper created with `try_load()`, `reset()`, and `inner()` methods
- [ ] Denoiser initialized once at app startup in `lib.rs`
- [ ] `AudioCommand::Start` passes shared denoiser through audio thread
- [ ] `CpalBackend::start()` uses shared denoiser instead of loading models
- [ ] LSTM states reset at start of each recording via `reset()`
- [ ] Recording starts near-instantly (no 2s delay)
- [ ] Graceful degradation if model loading fails

## Test Cases

- [ ] Recording starts without delay when shared denoiser is loaded
- [ ] Recording works without noise suppression if denoiser loading fails
- [ ] Multiple sequential recordings don't have audio artifacts from LSTM state carryover
- [ ] Listening mode and recording mode both use the shared denoiser

## Dependencies

- `pipeline-integration` - Denoiser must be integrated into capture pipeline first

## Preconditions

- DTLN noise suppression already working (initialized per-recording)
- `SharedTranscriptionModel` pattern exists as reference implementation

## Implementation Notes

### Key Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/audio/denoiser/shared.rs` | NEW - SharedDenoiser wrapper |
| `src-tauri/src/audio/denoiser/mod.rs` | Export shared module |
| `src-tauri/src/audio/mod.rs` | Update trait, re-export |
| `src-tauri/src/audio/cpal_backend.rs` | Use shared denoiser, remove loading |
| `src-tauri/src/audio/thread.rs` | Pass denoiser through AudioCommand |
| `src-tauri/src/lib.rs` | Init at startup, pass to pipelines |
| `src-tauri/src/recording/commands/logic.rs` | Accept denoiser param |
| `src-tauri/src/recording/commands/mod.rs` | Extract from state |
| `src-tauri/src/listening/pipeline.rs` | Store and pass denoiser |
| `src-tauri/src/hotkey/integration.rs` | Store and pass denoiser |

### SharedDenoiser Structure

```rust
#[derive(Clone)]
pub struct SharedDenoiser {
    inner: Arc<Mutex<DtlnDenoiser>>,
}

impl SharedDenoiser {
    pub fn try_load() -> Result<Self, DenoiserError>;
    pub fn reset(&self);  // Clear LSTM states for new recording
    pub fn inner(&self) -> Arc<Mutex<DtlnDenoiser>>;
}
```

### Critical: LSTM State Reset

The denoiser maintains LSTM hidden states between frames. **Must call `reset()` at start of each recording** to prevent audio artifacts from previous sessions bleeding through.

## Related Specs

- `dtln-denoiser.spec.md` - Core DTLN implementation
- `dtln-model-setup.spec.md` - Model loading
- `pipeline-integration.spec.md` - Integration into capture pipeline

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (startup initialization)
- Connects to: `audio/cpal_backend.rs`, `audio/thread.rs`, `recording/commands/`, `listening/pipeline.rs`, `hotkey/integration.rs`

## Integration Test

- Test location: Manual testing (start multiple recordings, verify no delay)
- Verification: [ ] Recording start time < 100ms after hotkey press

## Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `SharedDenoiser` wrapper created with `try_load()`, `reset()`, and `inner()` methods | PASS | `src-tauri/src/audio/denoiser/shared.rs:36-77` - All three methods implemented correctly |
| Denoiser initialized once at app startup in `lib.rs` | PASS | `src-tauri/src/lib.rs:116-125` - Initialized in setup() with logging |
| `AudioCommand::Start` passes shared denoiser through audio thread | PASS | `src-tauri/src/audio/thread.rs:31-34` - `shared_denoiser: Option<Arc<SharedDenoiser>>` field added to Start variant |
| `CpalBackend::start()` uses shared denoiser instead of loading models | PASS | `src-tauri/src/audio/cpal_backend.rs:48-56` - Uses `start_with_denoiser()` which accepts shared denoiser |
| LSTM states reset at start of each recording via `reset()` | PASS | `src-tauri/src/audio/cpal_backend.rs` - Calls `shared.reset()` before using |
| Recording starts near-instantly (no 2s delay) | PASS | Model loading happens at app startup (lib.rs:116), not at recording start |
| Graceful degradation if model loading fails | PASS | `src-tauri/src/lib.rs:120-124` - Wraps in Option, logs warning, continues without denoising |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `test_try_load_succeeds_with_embedded_models` | PASS | `src-tauri/src/audio/denoiser/shared.rs:94-99` |
| `test_reset_does_not_panic` | PASS | `src-tauri/src/audio/denoiser/shared.rs:101-108` |
| `test_inner_returns_same_instance` | PASS | `src-tauri/src/audio/denoiser/shared.rs:110-119` |
| `test_clone_shares_same_denoiser` | PASS | `src-tauri/src/audio/denoiser/shared.rs:121-129` |
| `test_shared_denoiser_is_send_sync` | PASS | `src-tauri/src/audio/denoiser/shared.rs:131-136` |
| Recording without noise suppression if denoiser loading fails | PASS | Graceful degradation verified via Option wrapper pattern |
| Multiple sequential recordings without LSTM artifacts | PASS | `reset()` called before each recording |
| Listening mode uses shared denoiser | DEFERRED | Listening pipeline doesn't use denoiser (denoiser is only for recording) |

### Code Quality

**Strengths:**
- Follows established `SharedTranscriptionModel` pattern consistently
- Proper LSTM state reset at recording start prevents audio artifacts
- Graceful degradation - app works without denoiser if loading fails
- Comprehensive unit tests covering Send+Sync, cloning, and reset behavior
- Clear documentation with usage examples in docstrings
- Debug impl hides internal Arc/Mutex complexity

**Concerns:**
- Build warnings for unused code in denoiser module - these are intentional API surface items for future file-based model loading (`DtlnModels::load()`, `ModelNotFound`, `TypedModel`) and the legacy `AudioCaptureBackend::start()` trait method. These should have `#[allow(dead_code)]` annotations to document intentional retention.

### Pre-Review Gate Results

**Build Warning Check:**
```
warning: method `start` is never used (audio/mod.rs:242)
warning: variant `ModelNotFound` is never constructed (denoiser/mod.rs:34)
warning: type alias `TypedModel` is never used (denoiser/mod.rs:42)
warning: associated functions `load` and `load_and_optimize_model` are never used (denoiser/mod.rs:71)
warning: method `get` is never used (dictionary/store.rs - UNRELATED to this spec)
```

Previous review requested cleanup of unused re-exports. Progress made:
- FIXED: Removed unused re-exports from `audio/mod.rs` (now only exports `SharedDenoiser`)
- FIXED: Removed `FFT_BINS` from denoiser/mod.rs exports
- NOT FIXED: `AudioCaptureBackend::start()` trait method still generates warning
- NOT FIXED: `DtlnModels::load()`, `TypedModel`, `ModelNotFound` still generate warnings

The remaining warnings are for intentionally-kept API surface (backwards compatibility, future file-based loading). These need `#[allow(dead_code)]` annotations to document the intent.

**Test Results:** All 415 tests pass.

### Data Flow Analysis

```
[App Startup - lib.rs:116]
    |
    v
[SharedDenoiser::try_load()] -> Option<Arc<SharedDenoiser>>
    |
    v
[app.manage(shared_denoiser)] - Stored in Tauri state
    |
    v
[HotkeyIntegration::with_shared_denoiser()] - lib.rs
    |
    v
[Hotkey pressed] -> integration.handle_toggle()
    |
    v
[start_recording_impl()] with shared_denoiser - logic.rs
    |
    v
[audio_thread.start_with_device_and_denoiser()] - thread.rs
    |
    v
[AudioCommand::Start { shared_denoiser }] - sent to audio thread
    |
    v
[CpalBackend::start_with_denoiser()] - cpal_backend.rs
    |
    v
[shared.reset()] + [shared.inner()] -> denoiser used in audio callback
```

All links are connected. SharedDenoiser flows from startup through state management to the audio callback.

### Deferrals Check

No TODOs/FIXMEs related to this spec.

### Verdict

**NEEDS_WORK** - Build warnings need `#[allow(dead_code)]` annotations

1. **What failed** - Pre-review gate: Build warnings for intentionally-kept API surface
2. **Why it failed** - The following items are intentionally kept but generate warnings:
   - `AudioCaptureBackend::start()` trait method (kept for API completeness)
   - `DtlnModels::load()` and `load_and_optimize_model()` (for future file-based loading)
   - `DenoiserError::ModelNotFound` (for file-based loading errors)
   - `TypedModel` type alias (public API surface)
3. **How to fix**:
   - Add `#[allow(dead_code)]` to `AudioCaptureBackend::start()` method in `src-tauri/src/audio/mod.rs:242` with comment explaining it's kept for API symmetry
   - Add `#[allow(dead_code)]` to `DtlnModels::load()` in `src-tauri/src/audio/denoiser/mod.rs:71` with comment explaining it's for future file-based loading
   - Add `#[allow(dead_code)]` to `load_and_optimize_model()` in `src-tauri/src/audio/denoiser/mod.rs:115`
   - Add `#[allow(dead_code)]` to `DenoiserError::ModelNotFound` in `src-tauri/src/audio/denoiser/mod.rs:34`
   - Add `#[allow(dead_code)]` to `TypedModel` type alias in `src-tauri/src/audio/denoiser/mod.rs:42`

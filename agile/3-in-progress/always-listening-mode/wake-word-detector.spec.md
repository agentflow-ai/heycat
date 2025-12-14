---
status: in-progress
created: 2025-12-14
completed: null
dependencies: []
review_round: 1
review_history:
  - round: 1
    date: 2025-12-14
    verdict: NEEDS_WORK
    failedCriteria: ["Emits `wake_word_detected` event via Tauri event system"]
    concerns: ["The spec explicitly requires event emission but the implementation only returns `WakeWordResult` - no event is emitted", "`WakeWordDetector` is not instantiated or used anywhere in the application (lib.rs or elsewhere)", "Some test cases from spec are missing (background noise, non-blocking processing)", "The integration test file specified in the spec (`detector_test.rs`) does not exist"]
---

# Spec: Core wake word detection engine

## Description

Implement the core wake word detection engine that analyzes streaming audio to detect the "Hey Cat" phrase. Uses on-device speech recognition via Parakeet with small-window batching for privacy-preserving detection. Emits events when the wake word is confidently detected.

> **MVP Note**: This implementation uses Parakeet (batch transcription model) with small-window batching (~1-2 seconds). CPU/latency optimization deferred to post-MVP.

## Acceptance Criteria

- [ ] `WakeWordDetector` struct created in `src-tauri/src/listening/detector.rs`
- [ ] Processes audio samples using Parakeet in small windows (~1-2 seconds)
- [ ] Detects "Hey Cat" phrase with configurable confidence threshold (default 0.8)
- [ ] Emits `wake_word_detected` event via Tauri event system
- [ ] Thread-safe implementation compatible with audio thread

## Test Cases

- [ ] Correctly detects "Hey Cat" spoken clearly
- [ ] Correctly detects "Hey Cat" with varying intonations
- [ ] Rejects similar phrases ("Hey Matt", "Pay Cat", "Hey")
- [ ] Handles background noise without false triggers
- [ ] Handles silence periods without errors
- [ ] Processes samples without blocking audio capture

## Dependencies

None

## Preconditions

- Parakeet TDT model available and loadable
- Audio capture system functional

## Implementation Notes

- Use Parakeet's `transcribe_file` or similar API with small audio windows
- Small-window batching: accumulate ~1-2 seconds of audio, run transcription, check for wake phrase
- Case-insensitive matching for "hey cat" variants
- Consider fuzzy matching for phonetic variations
- All code in unified `listening/` module

## Related Specs

- listening-audio-pipeline.spec.md (provides audio samples)
- listening-state-machine.spec.md (consumes detection events)

## Integration Points

- Production call site: `src-tauri/src/listening/detector.rs`
- Connects to: audio thread (receives samples), event system (emits detection)

## Integration Test

- Test location: `src-tauri/src/listening/detector_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-14
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `WakeWordDetector` struct created in `src-tauri/src/listening/detector.rs` | PASS | `detector.rs:77` - `pub struct WakeWordDetector` |
| Processes audio samples using Parakeet in small windows (~1-2 seconds) | PASS | `detector.rs:23-30` - default config `window_duration_secs: 2.0`; `detector.rs:160-162` - uses `tdt.transcribe_samples()` |
| Detects "Hey Cat" phrase with configurable confidence threshold (default 0.8) | PASS | `detector.rs:16` - `confidence_threshold: f32`; `detector.rs:26` - default `0.8`; `detector.rs:192-238` - `check_wake_phrase()` method |
| Emits `wake_word_detected` event via Tauri event system | FAIL | No event emission code exists in `detector.rs`. The `WakeWordDetector` only returns `WakeWordResult`; no Tauri event system integration is present. |
| Thread-safe implementation compatible with audio thread | PASS | `detector.rs:81` - `model: Arc<Mutex<Option<ParakeetTDT>>>`, `detector.rs:83` - `buffer: Mutex<CircularBuffer>` |

### Integration Path Trace

This spec describes a backend-only component, but event emission was specified as a requirement.

| Step | Expected | Actual Location | Status |
|------|----------|-----------------|--------|
| Event defined | `WAKE_WORD_DETECTED` constant | `src-tauri/src/events.rs:28` | PASS |
| Payload defined | `WakeWordDetectedPayload` | `src-tauri/src/events.rs:31-40` | PASS |
| Event emitted | `emit!()` in detector | None | FAIL |
| Detector instantiated | Production use in app | Not used in `lib.rs` or elsewhere | FAIL |

### Registration Audit

| Item | Type | Registered? | Evidence |
|------|------|-------------|----------|
| `listening` module | module | YES | `src-tauri/src/lib.rs:10` - `mod listening;` |
| `WakeWordDetector` | struct (exported) | YES | `src-tauri/src/listening/mod.rs:8` - `pub use detector::WakeWordDetector` |
| `WakeWordDetector` instantiation | production use | NO | Not instantiated anywhere in `lib.rs` setup |
| `wake_word_detected` event emission | production event | NO | No code emits this event |

### Mock-to-Production Audit

No mocks are used in the detector tests. Tests use the real `WakeWordDetector` struct with unit test methods that don't require the model to be loaded (testing `check_wake_phrase` directly).

| Mock | Test Location | Production Counterpart | Production Instantiation |
|------|---------------|----------------------|-------------------------|
| N/A | N/A | N/A | N/A |

### Event Subscription Audit

| Event Name | Emission Location | Frontend Listener | Listener Location |
|------------|-------------------|-------------------|-------------------|
| `wake_word_detected` | NONE (not emitted) | NO | N/A - no frontend `useListening` hook exists yet |

Note: The event is defined in `events.rs:28` but never actually emitted by the `WakeWordDetector`.

### Deferral Tracking

| Deferral Text | Location | Referenced Spec | Status |
|---------------|----------|-----------------|--------|
| MVP Note in spec | spec header | N/A | OK - explicitly acknowledged |

No TODO/FIXME/XXX/HACK found in the `listening/` module.

### Test Coverage Audit

| Test Case (from spec) | Test Location | Status |
|----------------------|---------------|--------|
| Correctly detects "Hey Cat" spoken clearly | `detector.rs:318-323` (`test_check_wake_phrase_exact_match`) | PASS |
| Correctly detects "Hey Cat" with varying intonations | `detector.rs:326-335` (`test_check_wake_phrase_case_insensitive`), `detector.rs:346-367` (variants) | PASS |
| Rejects similar phrases ("Hey Matt", "Pay Cat", "Hey") | `detector.rs:377-395` (`test_check_wake_phrase_rejects_similar_phrases`) | PASS |
| Handles background noise without false triggers | MISSING | Not directly tested - would require real audio |
| Handles silence periods without errors | `detector.rs:301-305` (`test_analyze_empty_buffer_returns_error`) | PARTIAL - tests empty buffer, not silence audio |
| Processes samples without blocking audio capture | MISSING | No async/threading tests exist |

Note: The integration test file `src-tauri/src/listening/detector_test.rs` specified in the spec does not exist.

### Code Quality

**Strengths:**
- Clean separation of concerns with `WakeWordDetectorConfig` struct
- Comprehensive error handling with `WakeWordError` enum
- Thread-safe design with `Arc<Mutex<>>` wrappers
- Good test coverage for the `check_wake_phrase` logic including rejection of similar phrases
- Follows existing Parakeet integration patterns (uses `transcribe_samples`)

**Concerns:**
- The spec explicitly requires event emission but the implementation only returns `WakeWordResult` - no event is emitted
- `WakeWordDetector` is not instantiated or used anywhere in the application (lib.rs or elsewhere)
- Some test cases from spec are missing (background noise, non-blocking processing)
- The integration test file specified in the spec (`detector_test.rs`) does not exist

### Verdict

**NEEDS_WORK** - The implementation is a solid foundation but fails one key acceptance criterion:

1. **What failed:** Acceptance criterion "Emits `wake_word_detected` event via Tauri event system"
2. **Why it failed:** The `WakeWordDetector` struct only returns a `WakeWordResult` from `analyze()`. There is no code to emit events via the Tauri event system. The event constant and payload are defined in `events.rs` but never used.
3. **How to fix:**
   - Option A: Add an `AppHandle` parameter to `WakeWordDetector` and emit the event in `analyze()` when detection succeeds
   - Option B: Clarify that event emission is the responsibility of the caller (e.g., the state machine) and update the spec to reflect this architectural decision

If Option B is chosen (event emission is caller's responsibility), the spec should be updated to remove the event emission criterion from this spec and ensure it's covered in `listening-state-machine.spec.md` instead.

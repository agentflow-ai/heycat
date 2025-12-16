---
status: completed
created: 2025-12-16
completed: 2025-12-16
dependencies: []
review_round: 1
priority: P1
---

# Spec: Create centralized audio constants module

## Description

Magic numbers are scattered throughout the audio processing code:
- `512` chunk size (5+ locations)
- `0.3` / `0.5` VAD thresholds
- `8000` samples (0.5s at 16kHz)
- `150ms` analysis interval
- `2.0s` window duration

Create a centralized `audio_constants.rs` module to hold all audio-related constants with documentation explaining their purpose and constraints.

## Acceptance Criteria

- [ ] Create `src-tauri/src/audio_constants.rs` module
- [ ] Define named constants for all magic numbers
- [ ] Add documentation explaining each constant's purpose
- [ ] Note constraints (e.g., chunk size depends on sample rate)
- [ ] Update all files to use constants instead of magic numbers
- [ ] Export constants from main lib.rs

## Test Cases

- [ ] Test constants have correct values
- [ ] Test chunk size calculation matches formula
- [ ] Verify no remaining magic numbers in audio code

## Dependencies

- sample-rate-validation.spec.md (uses OPTIMAL_CHUNK_DURATION_MS)

## Preconditions

None

## Implementation Notes

**New file:** `src-tauri/src/audio_constants.rs`

```rust
//! Centralized constants for audio processing.
//!
//! All audio-related magic numbers should be defined here with
//! documentation explaining their purpose and constraints.

/// Sample rate used throughout the audio pipeline (Hz).
/// Silero VAD only supports 8000 or 16000 Hz.
pub const DEFAULT_SAMPLE_RATE: u32 = 16000;

/// Optimal chunk duration for VAD processing (milliseconds).
/// Silero VAD works best with 32ms windows.
pub const OPTIMAL_CHUNK_DURATION_MS: u32 = 32;

/// Chunk size for VAD at 16kHz: 16000 * 32 / 1000 = 512 samples.
pub const VAD_CHUNK_SIZE_16KHZ: usize = 512;

/// Chunk size for VAD at 8kHz: 8000 * 32 / 1000 = 256 samples.
pub const VAD_CHUNK_SIZE_8KHZ: usize = 256;

/// VAD speech threshold for wake word detection.
/// Lower threshold = more sensitive to speech.
pub const VAD_THRESHOLD_WAKE_WORD: f32 = 0.3;

/// VAD speech threshold for silence/end-of-speech detection.
/// Higher threshold = more confident speech is present.
pub const VAD_THRESHOLD_SILENCE: f32 = 0.5;

/// Default analysis interval for wake word pipeline (milliseconds).
/// How often we analyze accumulated audio for wake word.
pub const ANALYSIS_INTERVAL_MS: u64 = 150;

/// Minimum new samples before analysis (at 16kHz).
/// 8000 samples = 0.5 seconds of audio.
pub const MIN_NEW_SAMPLES_FOR_ANALYSIS: usize = 8000;

/// Wake word detection window duration (seconds).
/// How much audio history to analyze for wake word.
pub const WAKE_WORD_WINDOW_SECS: f32 = 2.0;

/// Fingerprint overlap threshold for duplicate detection.
/// Audio segments with >50% overlap are considered duplicates.
pub const FINGERPRINT_OVERLAP_THRESHOLD: f32 = 0.5;

/// Transcription timeout for wake word detection (seconds).
pub const TRANSCRIPTION_TIMEOUT_SECS: u64 = 10;

/// Calculate chunk size for a given sample rate.
pub const fn chunk_size_for_sample_rate(sample_rate: u32) -> usize {
    (sample_rate * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
}
```

**Files to update:**
- `src-tauri/src/listening/vad.rs` - Use VAD_THRESHOLD_*, chunk_size_for_sample_rate
- `src-tauri/src/listening/detector.rs` - Use all detector constants
- `src-tauri/src/listening/silence.rs` - Use silence detection constants
- `src-tauri/src/listening/pipeline.rs` - Use ANALYSIS_INTERVAL_MS
- `src-tauri/src/listening/buffer.rs` - Reference constants in capacity comments
- `src-tauri/src/lib.rs` - Export the module

## Related Specs

- sample-rate-validation.spec.md (uses OPTIMAL_CHUNK_DURATION_MS)
- unified-vad-config.spec.md (completed - defines some thresholds)

## Integration Points

- N/A - This is a utility module with constants only
- Connects to: All audio processing modules

## Integration Test

- N/A (compile-time verification via usage)
- Verification: [x] N/A

## Review

**Reviewer:** Claude (Subagent)
**Date:** 2025-12-16
**Round:** 1

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Create `src-tauri/src/audio_constants.rs` module | ✅ | File exists at `/Users/michaelhindley/Documents/git/heycat/src-tauri/src/audio_constants.rs` with 243 lines of well-organized code |
| Define named constants for all magic numbers | ✅ | Constants defined: `DEFAULT_SAMPLE_RATE` (16000), `OPTIMAL_CHUNK_DURATION_MS` (32), `VAD_CHUNK_SIZE_16KHZ` (512), `VAD_CHUNK_SIZE_8KHZ` (256), `VAD_THRESHOLD_WAKE_WORD` (0.3), `VAD_THRESHOLD_BALANCED` (0.4), `VAD_THRESHOLD_SILENCE` (0.5), `VAD_THRESHOLD_AGGRESSIVE` (0.6), `WAKE_WORD_TRANSCRIPTION_TIMEOUT_SECS` (10), `WAKE_WORD_WINDOW_SECS` (2.0), `WAKE_WORD_MIN_NEW_SAMPLES` (12000), `WAKE_WORD_MIN_SPEECH_FRAMES` (4), `FINGERPRINT_OVERLAP_THRESHOLD` (0.5), `SILENCE_DURATION_MS` (2000), `NO_SPEECH_TIMEOUT_MS` (5000), `PAUSE_TOLERANCE_MS` (1000), `SILENCE_MIN_SPEECH_FRAMES` (2), `ANALYSIS_INTERVAL_MS` (150), `MIN_SAMPLES_FOR_ANALYSIS` (4000), `EVENT_CHANNEL_BUFFER_SIZE` (16) |
| Add documentation explaining each constant's purpose | ✅ | Each constant has detailed rustdoc comments explaining purpose, rationale, and usage context. File is organized into logical sections with clear headers |
| Note constraints (e.g., chunk size depends on sample rate) | ✅ | Constraints documented: e.g., "Silero VAD only supports 8000 or 16000 Hz", "Must be 512 for Silero VAD at 16kHz (32ms window)", memory bounds noted for buffer sizes |
| Update all files to use constants instead of magic numbers | ✅ | `vad.rs` imports and uses `DEFAULT_SAMPLE_RATE`, `VAD_CHUNK_SIZE_16KHZ`, `VAD_THRESHOLD_*`; `detector.rs` imports 7 constants; `silence.rs` imports 6 constants; `pipeline.rs` imports 3 constants |
| Export constants from main lib.rs | ✅ | `mod audio_constants;` declared at line 7 of `lib.rs` |

### Test Coverage Verification

| Test Case | Status | Evidence |
|-----------|--------|----------|
| Test constants have correct values | ✅ | Tests exist: `test_sample_rate_valid_for_silero` (validates 8000/16000), `test_threshold_ordering` (validates threshold order), `test_analysis_interval_reasonable` (50-500ms range) |
| Test chunk size calculation matches formula | ✅ | Tests: `test_chunk_size_calculation` verifies `chunk_size_for_sample_rate(16000) == VAD_CHUNK_SIZE_16KHZ`, `test_chunk_sizes_match_formula` verifies constants match calculated formula |
| Verify no remaining magic numbers in audio code | ✅ | Grep search found only: (1) comments/documentation explaining the constants, (2) test data literals (e.g., `vec![0.1, 0.2, 0.3]`), (3) test configuration values. No production magic numbers remain |

### Code Quality Notes

1. **Excellent organization**: Constants are grouped into logical sections (Sample Rate/Timing, VAD Chunk Sizes, VAD Thresholds, Wake Word Detection, Silence Detection, Pipeline Configuration, Utility Functions).

2. **Comprehensive documentation**: Every constant has rustdoc comments explaining not just what it is, but why that value was chosen and what constraints apply.

3. **Helper function**: `chunk_size_for_sample_rate()` is provided as a const fn for calculating chunk sizes dynamically.

4. **Additional constants**: The implementation goes beyond the spec by adding useful related constants like `VAD_THRESHOLD_BALANCED` (0.4), `VAD_THRESHOLD_AGGRESSIVE` (0.6), `SILENCE_MIN_SPEECH_FRAMES`, and `EVENT_CHANNEL_BUFFER_SIZE`.

5. **Dead code annotations**: Unused constants (like `VAD_CHUNK_SIZE_8KHZ`, `PAUSE_TOLERANCE_MS`) are properly annotated with `#[allow(dead_code)]` to indicate they are reserved for future use.

6. **Proper imports**: All consuming files properly import only the constants they need rather than using wildcard imports.

7. **Test coverage**: The module includes 9 comprehensive tests covering value correctness, formula verification, threshold ordering, memory bounds, and reasonable value ranges.

### Issues Found

None

### Verdict

**APPROVED**

The implementation fully meets all acceptance criteria. The `audio_constants.rs` module is well-organized, thoroughly documented, and properly integrated throughout the codebase. All magic numbers from the spec have been replaced with named constants, and comprehensive tests verify the constants' correctness. The remaining numeric literals found in the codebase are either in comments/documentation, test data, or test configuration - no production magic numbers remain.

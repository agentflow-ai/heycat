---
status: completed
created: 2025-12-16
completed: 2025-12-16
dependencies: []
review_round: 1
priority: P0
---

# Spec: Add sample rate validation in VAD creation

## Description

The VAD (Silero) only supports 8kHz or 16kHz sample rates, and the chunk size (512) assumes 16kHz (32ms window). Currently there is **no runtime validation** - passing the wrong sample rate causes silent failure. The chunk size should also adapt based on sample rate.

Add explicit validation in `create_vad()` to fail fast with a clear error when an unsupported sample rate is used, and auto-calculate the correct chunk size.

## Acceptance Criteria

- [ ] `create_vad()` returns error for sample rates other than 8000 or 16000
- [ ] Error message clearly states supported sample rates
- [ ] Chunk size is calculated from sample rate (32ms window): `sample_rate * 32 / 1000`
- [ ] VadConfig fields `chunk_size` becomes derived, not user-specified
- [ ] Add constant for optimal chunk duration (32ms)
- [ ] Update all callers if VadConfig API changes

## Test Cases

- [ ] Test `create_vad()` with 8000 Hz succeeds with chunk_size=256
- [ ] Test `create_vad()` with 16000 Hz succeeds with chunk_size=512
- [ ] Test `create_vad()` with 44100 Hz returns clear error
- [ ] Test `create_vad()` with 0 Hz returns clear error
- [ ] Test error message mentions "8000 or 16000"

## Dependencies

None

## Preconditions

- Existing VAD module with `VadConfig` and `create_vad()` function

## Implementation Notes

**File:** `src-tauri/src/listening/vad.rs`

**Current state:**
- Lines 51-128: `VadConfig` struct with `sample_rate` and `chunk_size` fields
- Lines 139-145: `create_vad()` doesn't validate sample rate
- Chunk size hardcoded as 512 in 5+ places (vad.rs:70, silence.rs:150, detector.rs:539)

**Proposed changes:**

```rust
pub const OPTIMAL_CHUNK_DURATION_MS: u32 = 32;

impl VadConfig {
    pub fn chunk_size_for_sample_rate(sample_rate: u32) -> usize {
        (sample_rate * OPTIMAL_CHUNK_DURATION_MS / 1000) as usize
    }
}

pub fn create_vad(config: &VadConfig) -> Result<VoiceActivityDetector, VadError> {
    // Validate sample rate
    match config.sample_rate {
        8000 | 16000 => {},
        other => return Err(VadError::ConfigurationInvalid(
            format!("Unsupported sample rate: {}. Must be 8000 or 16000 Hz.", other)
        )),
    }

    let chunk_size = VadConfig::chunk_size_for_sample_rate(config.sample_rate);

    VoiceActivityDetector::builder()
        .sample_rate(config.sample_rate as i32)
        .chunk_size(chunk_size)
        .build()
        .map_err(|e| VadError::InitializationFailed(e.to_string()))
}
```

**Also add new error variant:**
```rust
pub enum VadError {
    InitializationFailed(String),
    ConfigurationInvalid(String),  // NEW
}
```

## Related Specs

- unified-vad-config.spec.md (completed)
- audio-constants-module.spec.md (should define OPTIMAL_CHUNK_DURATION_MS there)

## Integration Points

- Production call site: `src-tauri/src/listening/detector.rs:271`
- Production call site: `src-tauri/src/listening/silence.rs:90`
- Connects to: WakeWordDetector, SilenceDetector

## Integration Test

- Test location: `src-tauri/src/listening/vad.rs` (test module)
- Verification: [ ] Integration test passes

---

## Review

**Reviewer:** Claude (subagent)
**Date:** 2025-12-16
**Round:** 1

### 1. Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `create_vad()` returns error for sample rates other than 8000 or 16000 | PASS | `vad.rs:145-153` - match on 8000/16000, else returns `VadError::ConfigurationInvalid` |
| Error message clearly states supported sample rates | PASS | `vad.rs:148-151` - message: "Unsupported sample rate: {} Hz. Must be 8000 or 16000 Hz." |
| Chunk size is calculated from sample rate (32ms window) | PASS | `vad.rs:156` calls `chunk_size_for_sample_rate()`, which is defined in `audio_constants.rs:174-176` as `(sample_rate * OPTIMAL_CHUNK_DURATION_MS / 1000)` |
| VadConfig fields `chunk_size` becomes derived, not user-specified | PASS | `vad.rs:61-82` - VadConfig struct has no `chunk_size` field; it's calculated in `create_vad()` |
| Add constant for optimal chunk duration (32ms) | PASS | `audio_constants.rs:22` - `OPTIMAL_CHUNK_DURATION_MS: u32 = 32` |
| Update all callers if VadConfig API changes | PASS | `detector.rs:269-290` and `silence.rs:87-93,117-123` both use updated VadConfig without chunk_size |

### 2. Integration Path Trace

This spec is backend-only (Rust VAD module). No frontend-backend interaction required.

**Call sites verified:**
- `detector.rs:284` calls `create_vad(&vad_config)` with validated config
- `silence.rs:93` calls `create_vad(&vad_config).ok()` with validated config
- `silence.rs:123` (in reset) calls `create_vad(&vad_config).ok()` with validated config

### 3. Registration Audit

| Item | Type | Registered? | Evidence |
|------|------|-------------|----------|
| VadError::ConfigurationInvalid | error variant | YES | `vad.rs:16` - added to VadError enum |
| chunk_size_for_sample_rate | function | YES | `audio_constants.rs:174-176` - public function |
| OPTIMAL_CHUNK_DURATION_MS | constant | YES | `audio_constants.rs:22` - public constant |

No new Tauri commands, events, or frontend hooks required for this spec.

### 4. Mock-to-Production Audit

No mocks used in the implementation. Tests use the real `create_vad()` function.

### 5. Event Subscription Audit

No events emitted by this spec. VAD errors are returned to callers who handle them.

### 6. Deferral Tracking

No deferrals found in the implementation (`TODO`, `FIXME`, `XXX`, `HACK` search returned no results in vad.rs).

### 7. Test Coverage Audit

| Test Case (from spec) | Test Location | Status |
|----------------------|---------------|--------|
| Test `create_vad()` with 8000 Hz succeeds with chunk_size=256 | `vad.rs:250-259` `test_create_vad_with_8khz_succeeds` | PASS |
| Test `create_vad()` with 16000 Hz succeeds with chunk_size=512 | `vad.rs:261-271` `test_create_vad_with_16khz_succeeds` | PASS |
| Test `create_vad()` with 44100 Hz returns clear error | `vad.rs:273-283` `test_create_vad_with_44100hz_returns_error` | PASS |
| Test `create_vad()` with 0 Hz returns clear error | `vad.rs:285-295` `test_create_vad_with_0hz_returns_error` | PASS |
| Test error message mentions "8000 or 16000" | `vad.rs:297-308` `test_sample_rate_error_message_mentions_supported_rates` | PASS |

All 17 VAD tests pass (`cargo test --lib listening::vad`).

### 8. Build Warning Audit

**Backend (Rust):**
```
warning: unused imports: `VAD_CHUNK_SIZE_16KHZ` and `VAD_CHUNK_SIZE_8KHZ`
 --> src/listening/vad.rs:5:54
```

**Analysis:** These constants ARE used in the test module (`vad.rs:258,270`) for asserting correct chunk size calculation. The warning appears because they're not used in non-test code. This is a pre-existing pattern in the codebase (note `VAD_CHUNK_SIZE_8KHZ` already had `#[allow(dead_code)]` in audio_constants.rs:37). The imports in vad.rs tests are intentional for verification.

**Verdict:** Warning is acceptable - constants are used in tests for verification of the spec's acceptance criteria.

### 9. Code Quality Notes

- [x] Error handling appropriate - returns typed `VadError::ConfigurationInvalid`
- [x] No unwrap() on user-facing code paths - all errors propagated via Result
- [x] Types are explicit - no untyped any/unknown
- [x] Consistent with existing patterns in codebase - matches VadError::InitializationFailed pattern

### 10. Verdict

**APPROVED**

All acceptance criteria pass with line-level evidence. The implementation:
1. Validates sample rate in `create_vad()` returning clear error for unsupported rates
2. Calculates chunk size dynamically using `chunk_size_for_sample_rate()`
3. Removes `chunk_size` from `VadConfig` (now derived)
4. Adds `OPTIMAL_CHUNK_DURATION_MS` constant in audio_constants module
5. Updates all callers (detector.rs, silence.rs) to use the new API
6. Has complete test coverage matching all spec test cases
7. Build warning is for test-only imports (acceptable)

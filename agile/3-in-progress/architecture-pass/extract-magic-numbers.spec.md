---
status: pending
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Extract magic numbers to named constants

## Description

Several magic numbers appear in the codebase without explanation. Extract these to named constants with documentation to improve readability and maintainability.

**Severity:** Low (code quality improvement)

## Acceptance Criteria

- [ ] All identified magic numbers replaced with named constants
- [ ] Constants have doc comments explaining their purpose
- [ ] Constants are placed in appropriate location (module-level or `audio_constants.rs`)
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Test Cases

- [ ] Grep confirms no remaining undocumented magic numbers in target files
- [ ] Existing tests continue to pass (no behavioral change)

## Dependencies

None

## Preconditions

None

## Implementation Notes

**Magic numbers to extract:**

1. **coordinator.rs:231** - `1600` samples
   ```rust
   // Before:
   if samples_since_last_check.len() >= 1600 {

   // After (in audio_constants.rs):
   /// Minimum samples to process for detection (100ms at 16kHz)
   pub const MIN_DETECTION_SAMPLES: usize = 1600;

   // In coordinator.rs:
   if samples_since_last_check.len() >= MIN_DETECTION_SAMPLES {
   ```

2. **coordinator.rs:168** - `100` ms interval
   ```rust
   // Before:
   let interval = Duration::from_millis(100);

   // After:
   /// Detection check interval in milliseconds
   pub const DETECTION_INTERVAL_MS: u64 = 100;
   let interval = Duration::from_millis(DETECTION_INTERVAL_MS);
   ```

3. **cpal_backend.rs:248,273** - `1024` chunk size
   ```rust
   // Before:
   let chunk_size = 1024;

   // After (in audio_constants.rs or cpal_backend.rs):
   /// Chunk size for real-time resampling
   pub const RESAMPLE_CHUNK_SIZE: usize = 1024;
   ```

4. **cgeventtap.rs** - Various bitmask constants (already well-named, just verify docs)

5. **matcher.rs:10** - `0.8` threshold (already has `DEFAULT_THRESHOLD` constant - good!)

**Files to modify:**
- `src-tauri/src/audio_constants.rs` (add new constants)
- `src-tauri/src/listening/coordinator.rs`
- `src-tauri/src/audio/cpal_backend.rs`

**Pattern to follow:**
Look at existing `audio_constants.rs` for naming conventions and documentation style.

## Related Specs

None

## Integration Points

- Production call site: N/A (pure refactor, no behavior change)
- Connects to: N/A

## Integration Test

- Test location: N/A (unit-only spec)
- Verification: [x] N/A

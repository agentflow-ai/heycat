---
status: in-progress
created: 2025-12-23
completed: null
dependencies: []
review_round: 1
review_history:
  - round: 1
    date: 2025-12-23
    verdict: NEEDS_WORK
    failedCriteria: ["Configuration flag to adjust buffer size (for troubleshooting)"]
    concerns: ["**Missing configuration flag**: The acceptance criteria specifies \"Configuration flag to adjust buffer size (for troubleshooting)\" but only a compile-time constant was implemented. Users cannot adjust the buffer size at runtime for troubleshooting without recompiling."]
---

# Spec: Configure cpal audio buffer size for reduced latency and glitches

## Description

Configure cpal's audio stream with an explicit buffer size instead of relying on platform defaults. Currently, `build_input_stream()` is called with `None` for the buffer size, letting the OS choose. This can cause variable latency, potential dropouts, and audio glitches.

Setting a fixed buffer size (256 samples) provides more consistent timing and may reduce artifacts on some systems.

## Acceptance Criteria

- [ ] Request specific buffer size (256 samples) via `StreamConfig::buffer_size`
- [ ] Handle fallback gracefully if platform rejects the requested size
- [ ] Add `PREFERRED_BUFFER_SIZE` constant to `audio_constants.rs`
- [ ] Log actual buffer size used (may differ from requested)
- [ ] No increase in audio dropouts or CPU usage
- [ ] Configuration flag to adjust buffer size (for troubleshooting)

## Test Cases

- [ ] Audio capture works with requested buffer size (256)
- [ ] Audio capture falls back gracefully if 256 is rejected
- [ ] Buffer size is logged at stream creation
- [ ] No audible glitches in test recordings
- [ ] Performance: callback processing completes within buffer period

## Dependencies

None (can be implemented independently)

## Preconditions

- Audio capture pipeline is functional
- cpal 0.15+ supports `BufferSize::Fixed`

## Implementation Notes

### Changes to cpal_backend.rs

Currently (around line 523):
```rust
device.build_input_stream(
    &config.into(),
    // ...
)
```

Should become:
```rust
let mut stream_config: cpal::StreamConfig = config.into();
stream_config.buffer_size = cpal::BufferSize::Fixed(PREFERRED_BUFFER_SIZE);

device.build_input_stream(
    &stream_config,
    // ...
)
```

### Buffer Size Selection

| Buffer Size | Latency @ 16kHz | Latency @ 48kHz | Notes |
|-------------|-----------------|-----------------|-------|
| 128 | 8ms | 2.7ms | Very low latency, higher CPU |
| 256 | 16ms | 5.3ms | Good balance (recommended) |
| 512 | 32ms | 10.7ms | Lower CPU, higher latency |

### Constant (add to audio_constants.rs)
```rust
/// Preferred audio buffer size for consistent timing.
/// 256 samples = ~16ms at 16kHz, ~5ms at 48kHz.
/// Smaller values reduce latency but increase CPU usage.
pub const PREFERRED_BUFFER_SIZE: u32 = 256;
```

### Error Handling

cpal may reject the requested buffer size on some platforms. If `BufferSize::Fixed` fails:
1. Log warning with details
2. Fall back to `BufferSize::Default`
3. Continue with platform-chosen buffer

## Related Specs

- All other specs benefit from consistent timing
- Independent of other specs (no dependencies)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs:build_input_stream()` (multiple locations for different sample formats)
- Connects to: Audio callback processing

## Integration Test

- Test location: Manual A/B testing with existing recordings
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-23
**Reviewer:** Claude

### Pre-Review Gates (Automated)

**1. Build Warning Check:**
```
warning: associated function `with_worktree_context` is never used
warning: associated items `with_default_path` and `get` are never used
warning: associated function `with_config` is never used
warning: associated function `new` is never used
warning: associated function `with_default_path` is never used
```
**Result:** PASS - All warnings are pre-existing (unrelated to this spec's changes)

**2. Command Registration Check:** N/A - No new commands added

**3. Event Subscription Check:** N/A - No new events added

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Request specific buffer size (256 samples) via `StreamConfig::buffer_size` | PASS | `cpal_backend.rs:115` - `config.buffer_size = BufferSize::Fixed(PREFERRED_BUFFER_SIZE);` |
| Handle fallback gracefully if platform rejects the requested size | PASS | `cpal_backend.rs:567-588`, `611-633`, `656-678` - Each sample format tries fixed size first, falls back to `BufferSize::Default` on error |
| Add `PREFERRED_BUFFER_SIZE` constant to `audio_constants.rs` | PASS | `audio_constants.rs:206` - `pub const PREFERRED_BUFFER_SIZE: u32 = 256;` with documentation |
| Log actual buffer size used (may differ from requested) | PASS | `cpal_backend.rs:116-121` logs requested size, `cpal_backend.rs:569/613/658` logs success, `cpal_backend.rs:573-576/617-620/662-665` logs fallback |
| No increase in audio dropouts or CPU usage | DEFERRED | Manual A/B testing required |
| Configuration flag to adjust buffer size (for troubleshooting) | FAIL | No configuration flag implemented - only a compile-time constant |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Audio capture works with requested buffer size (256) | DEFERRED | Manual testing required (cpal requires real audio device) |
| Audio capture falls back gracefully if 256 is rejected | DEFERRED | Manual testing required (cpal requires real audio device) |
| Buffer size is logged at stream creation | PASS | Code contains `crate::info!()` calls at `cpal_backend.rs:116-121`, `569`, `573-576` etc. |
| No audible glitches in test recordings | DEFERRED | Manual A/B testing required |
| Performance: callback processing completes within buffer period | DEFERRED | Manual testing required |
| Buffer size constant is reasonable (power of 2, 64-1024 range) | PASS | `audio_constants.rs:300-308` - `test_preferred_buffer_size_reasonable` |
| Latency calculations are correct | PASS | `audio_constants.rs:311-319` - `test_buffer_latency_calculation` |

### Code Quality

**Strengths:**
- Clean implementation with proper separation: constant in `audio_constants.rs`, usage in `cpal_backend.rs`
- Excellent fallback pattern: tries fixed buffer, logs warning, falls back to platform default
- Consistent implementation across all three sample formats (F32, I16, U16)
- Good logging at each decision point (request, success, fallback)
- Helper function `create_stream_config_with_buffer_size()` encapsulates buffer configuration
- Unit tests verify the constant's properties and latency calculations

**Concerns:**
- **Missing configuration flag**: The acceptance criteria specifies "Configuration flag to adjust buffer size (for troubleshooting)" but only a compile-time constant was implemented. Users cannot adjust the buffer size at runtime for troubleshooting without recompiling.

### Data Flow Analysis

```
[Stream Creation] CpalBackend::start()
     |
     v
[Config Helper] create_stream_config_with_buffer_size()
     | Sets BufferSize::Fixed(256)
     v
[Device] device.build_input_stream(&stream_config, ...)
     |
     +--> [Success] Log "Stream created with fixed buffer size: 256 samples"
     |
     +--> [Failure] Log warning, retry with BufferSize::Default
```

All new code is wired into production at `cpal_backend.rs:540` where `create_stream_config_with_buffer_size()` is called.

### Verdict

**NEEDS_WORK** - Missing acceptance criterion: "Configuration flag to adjust buffer size (for troubleshooting)"

The implementation currently only provides a compile-time constant. To satisfy the acceptance criteria, one of these approaches is needed:
1. Add an environment variable override (e.g., `HEYCAT_BUFFER_SIZE`)
2. Add a config file setting
3. Add a Tauri command to adjust buffer size dynamically

If the configuration flag is intentionally deferred, update the acceptance criteria to remove it or mark it as deferred with a tracking spec.

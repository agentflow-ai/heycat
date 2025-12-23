---
status: pending
created: 2025-12-23
completed: null
dependencies: ["channel-mixing", "resampler-quality-upgrade", "audio-preprocessing", "audio-gain-normalization"]
---

# Spec: Quality metrics and diagnostic tooling

## Description

Add comprehensive quality metrics and diagnostic tooling to the audio pipeline. This enables debugging audio quality issues, A/B comparison of processing stages, and provides visibility into pipeline health. Includes logging of key metrics, optional raw/processed audio capture, and quality warnings sent to the frontend.

## Acceptance Criteria

- [ ] Track and log per-recording metrics: input level (peak/RMS), output level, clipping events, AGC gain applied
- [ ] Add debug mode to save raw (pre-processing) audio alongside processed audio
- [ ] Emit quality warning events to frontend (e.g., "input too quiet", "clipping detected")
- [ ] Log sample count at each pipeline stage to detect data loss
- [ ] Include pipeline stage timing metrics (useful for performance tuning)
- [ ] Diagnostics can be enabled/disabled via settings (default: minimal logging)

## Test Cases

- [ ] Quiet recording triggers "input too quiet" warning event
- [ ] Clipping input triggers "clipping detected" warning event
- [ ] Debug mode saves raw audio file alongside processed audio
- [ ] Sample counts at pipeline input/output match expected ratios
- [ ] Metrics logged correctly for normal recording session
- [ ] Disabled diagnostics produce no additional logging/files

## Dependencies

- `channel-mixing` - diagnostics tracks channel mixing stage
- `resampler-quality-upgrade` - diagnostics tracks resampler stage
- `audio-preprocessing` - diagnostics tracks preprocessing stage
- `audio-gain-normalization` - diagnostics tracks AGC gain levels

## Preconditions

- All other pipeline specs are implemented
- Event emission infrastructure exists (app_handle.emit)

## Implementation Notes

- Extend existing `CallbackState::log_sample_diagnostics()` with more metrics
- Create `src-tauri/src/audio/diagnostics.rs` module for metric collection
- Metrics to track:
  - Input peak/RMS level (before processing)
  - Output peak/RMS level (after processing)
  - Clipping count (samples at or near ±1.0)
  - AGC current gain
  - Processing stage latencies
- For debug mode:
  - Save raw audio to separate file with `-raw` suffix
  - Use existing WAV encoding infrastructure
- Frontend events:
  - `recording_quality_warning` with payload: `{ type: "quiet" | "clipping", severity: "info" | "warning" }`

## Related Specs

- All other specs in this feature (provides observability for entire pipeline)

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs` (metrics collection throughout pipeline)
- Connects to: All pipeline stages, frontend via events

## Integration Test

- Test location: `src-tauri/src/audio/diagnostics.rs` (unit tests)
- Verification: [ ] Integration test passes

---
status: pending
created: 2025-11-26
completed: null
dependencies: []
---

# Spec: Audio Capture Module

## Description

Implement a pure Rust module using cpal to capture audio from the default microphone into a thread-safe buffer. This module provides the core audio capture primitives without any Tauri dependencies.

## Acceptance Criteria

- [ ] Initialize audio capture from default input device
- [ ] Collect audio samples in thread-safe buffer (`Arc<Mutex<Vec<f32>>>`)
- [ ] Expose `start()` and `stop()` methods for capture control
- [ ] Handle audio device errors with Result types
- [ ] Support configurable sample rate (default 44.1kHz)

## Test Cases

- [ ] Capture module initializes without errors when audio device available
- [ ] Start/stop methods transition internal state correctly
- [ ] Error returned when no audio device available (graceful handling)
- [ ] Sample rate configuration applied correctly

## Dependencies

None

## Preconditions

- `cpal` crate added to Cargo.toml
- Audio input device available on system (or mock for tests)

## Implementation Notes

- Create new module: `src-tauri/src/audio/capture.rs`
- Use `cpal::default_host()` and `default_input_device()`
- Thread-safe buffer pattern: `Arc<Mutex<Vec<f32>>>`
- Mark hardware interaction code with `#[cfg_attr(coverage_nightly, coverage(off))]`

## Related Specs

- [wav-encoding.spec.md](wav-encoding.spec.md) - Uses captured audio samples
- [recording-state-manager.spec.md](recording-state-manager.spec.md) - Manages capture state

---
status: pending
created: 2025-11-26
completed: null
dependencies: []
---

# Spec: WAV Encoding Module

## Description

Implement a pure Rust module using hound to encode audio samples as WAV files. Takes `Vec<f32>` samples and writes them to disk in standard WAV format.

## Acceptance Criteria

- [ ] Accept `Vec<f32>` audio samples as input
- [ ] Write WAV file with 16-bit PCM format
- [ ] Generate unique timestamped filenames (e.g., `recording-2025-11-26-143052.wav`)
- [ ] Create output directory if it doesn't exist
- [ ] Return file path on success

## Test Cases

- [ ] WAV file created with correct format headers
- [ ] Filename includes ISO timestamp
- [ ] Directory creation works when parent exists but target doesn't
- [ ] Empty sample vector handled gracefully
- [ ] File path returned matches actual file location

## Dependencies

None

## Preconditions

- `hound` crate added to Cargo.toml
- Write access to output directory

## Implementation Notes

- Create new module: `src-tauri/src/audio/wav.rs`
- Use `hound::WavWriter` with spec: 16-bit, mono, 44.1kHz
- Convert f32 samples to i16: `(sample * i16::MAX as f32) as i16`
- Default output: `~/heycat-recordings/`

## Related Specs

- [audio-capture.spec.md](audio-capture.spec.md) - Provides audio samples
- [recording-coordinator.spec.md](recording-coordinator.spec.md) - Orchestrates encoding

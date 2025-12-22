---
last-updated: 2025-12-22
status: active
---

# Technical Guidance: Real-time Audio Noise Suppression

## Data Flow Diagrams

### High-Level Pipeline Integration

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         AUDIO CAPTURE PIPELINE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────┐    ┌──────────┐    ┌─────────────────┐    ┌──────────────┐   │
│  │Microphone│───▶│   cpal   │───▶│Format Conversion│───▶│DTLN Denoiser │   │
│  │          │    │(capture) │    │  (to f32 mono)  │    │  (NEW)       │   │
│  └──────────┘    └──────────┘    └─────────────────┘    └──────┬───────┘   │
│                                                                 │           │
│                                                                 ▼           │
│  ┌──────────────┐    ┌─────────┐    ┌────────────┐    ┌──────────────┐     │
│  │Transcription │◀───│   VAD   │◀───│AudioBuffer │◀───│Denoised Audio│     │
│  │  (Parakeet)  │    │(Silero) │    │(ring buffer│    │              │     │
│  └──────────────┘    └─────────┘    └────────────┘    └──────────────┘     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### DTLN Denoiser Internal Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          DTLN DENOISER (per frame)                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Input: 512 samples (f32, 16kHz)                                            │
│  ┌─────────────────┐                                                        │
│  │  Frame Buffer   │ ← Accumulates samples, shifts by 128                   │
│  │  (512 samples)  │                                                        │
│  └────────┬────────┘                                                        │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────┐                                                        │
│  │   Hann Window   │ ← Reduces spectral leakage                             │
│  └────────┬────────┘                                                        │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────┐      ┌──────────────┐                                  │
│  │    FFT (257)    │─────▶│   Magnitude  │                                  │
│  │   (rustfft)     │      │    + Phase   │                                  │
│  └─────────────────┘      └──────┬───────┘                                  │
│                                  │                                           │
│                    ┌─────────────┴─────────────┐                            │
│                    │                           │                            │
│                    ▼                           ▼                            │
│           ┌────────────────┐          ┌────────────────┐                    │
│           │   Model 1      │          │  Phase (kept)  │                    │
│           │  (ONNX LSTM)   │          │                │                    │
│           │ → Noise Mask   │          └────────┬───────┘                    │
│           └────────┬───────┘                   │                            │
│                    │                           │                            │
│                    ▼                           │                            │
│           ┌────────────────┐                   │                            │
│           │ Apply Mask to  │◀──────────────────┘                            │
│           │   Magnitude    │                                                │
│           └────────┬───────┘                                                │
│                    │                                                         │
│                    ▼                                                         │
│           ┌────────────────┐                                                │
│           │     IFFT       │                                                │
│           │ (reconstruct)  │                                                │
│           └────────┬───────┘                                                │
│                    │                                                         │
│                    ▼                                                         │
│           ┌────────────────┐                                                │
│           │   Model 2      │                                                │
│           │  (ONNX LSTM)   │                                                │
│           │ → Refinement   │                                                │
│           └────────┬───────┘                                                │
│                    │                                                         │
│                    ▼                                                         │
│           ┌────────────────┐                                                │
│           │  Overlap-Add   │ ← Combines frames smoothly                     │
│           │    Buffer      │                                                │
│           └────────┬───────┘                                                │
│                    │                                                         │
│                    ▼                                                         │
│  Output: 128 samples (shifted output per frame)                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### LSTM State Management

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         STATE PERSISTENCE ACROSS FRAMES                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Frame N                           Frame N+1                                │
│  ┌─────────────────┐              ┌─────────────────┐                       │
│  │    Model 1      │              │    Model 1      │                       │
│  │  ┌───────────┐  │              │  ┌───────────┐  │                       │
│  │  │ hidden_1  │──┼──────────────┼─▶│ hidden_1  │  │                       │
│  │  │ cell_1    │──┼──────────────┼─▶│ cell_1    │  │                       │
│  │  └───────────┘  │              │  └───────────┘  │                       │
│  └─────────────────┘              └─────────────────┘                       │
│                                                                             │
│  ┌─────────────────┐              ┌─────────────────┐                       │
│  │    Model 2      │              │    Model 2      │                       │
│  │  ┌───────────┐  │              │  ┌───────────┐  │                       │
│  │  │ hidden_2  │──┼──────────────┼─▶│ hidden_2  │  │                       │
│  │  │ cell_2    │──┼──────────────┼─▶│ cell_2    │  │                       │
│  │  └───────────┘  │              │  └───────────┘  │                       │
│  └─────────────────┘              └─────────────────┘                       │
│                                                                             │
│  States are MODEL OUTPUTS that become INPUTS for the next frame             │
│  Initialize to zeros at start, reset on new recording session              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Graceful Degradation Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         INITIALIZATION WITH FALLBACK                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                    ┌─────────────────────┐                                  │
│                    │  Load ONNX Models   │                                  │
│                    └──────────┬──────────┘                                  │
│                               │                                             │
│                    ┌──────────▼──────────┐                                  │
│                    │     Success?        │                                  │
│                    └──────────┬──────────┘                                  │
│                               │                                             │
│              ┌────────────────┴────────────────┐                            │
│              │                                 │                            │
│              ▼                                 ▼                            │
│     ┌────────────────┐              ┌────────────────────┐                  │
│     │   Ok(models)   │              │   Err(error)       │                  │
│     └───────┬────────┘              └─────────┬──────────┘                  │
│             │                                 │                             │
│             ▼                                 ▼                             │
│     ┌────────────────┐              ┌────────────────────┐                  │
│     │ denoiser =     │              │ warn!("Failed...")  │                  │
│     │ Some(Denoiser) │              │ denoiser = None    │                  │
│     └───────┬────────┘              └─────────┬──────────┘                  │
│             │                                 │                             │
│             └────────────┬────────────────────┘                            │
│                          ▼                                                  │
│             ┌────────────────────────┐                                      │
│             │  Audio capture works   │                                      │
│             │  (with or without      │                                      │
│             │   noise suppression)   │                                      │
│             └────────────────────────┘                                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Architecture Overview

### Integration Point
The denoiser will be inserted into the existing audio pipeline between format conversion and the AudioBuffer:

```
Microphone → cpal → format conversion → DTLN Denoiser → AudioBuffer → VAD → transcription
```

### Processing Parameters
| Parameter | Value | Notes |
|-----------|-------|-------|
| Sample Rate | 16kHz | Matches existing pipeline |
| Frame Size | 512 samples | 32ms at 16kHz |
| Frame Shift | 128 samples | 8ms, 75% overlap |
| Latency | 32ms | One frame delay |
| Model Format | ONNX | model_1.onnx, model_2.onnx |

## Testing Strategy

> **Philosophy**: Test behavior, not implementation. See `docs/TESTING.md` for full guidelines.

### What to Test (Behavior-Focused)

| Behavior | Test Description | Why It Matters |
|----------|------------------|----------------|
| Complete denoising flow | Feed noisy audio → get cleaner audio | Users care about reduced noise |
| Speech preservation | Sine wave / speech sample → frequencies preserved | Users need clear transcription |
| Graceful degradation | Missing models → audio still works | Users shouldn't see errors |
| State continuity | Multiple frames → smooth output | Users hear continuous audio |

### What NOT to Test

- LSTM state initialization (implementation detail)
- FFT bin counts (framework handles this)
- Model input/output tensor shapes (obvious if it works)
- Frame buffer shift logic in isolation (tested via behavior)

### Test Structure

```rust
// GOOD: Complete workflow test
#[test]
fn test_denoising_reduces_noise_and_preserves_speech() {
    // Setup: Load actual models, create denoiser
    let denoiser = DtlnDenoiser::new().expect("models should load");

    // Create test audio: speech + noise
    let noisy_audio = generate_speech_with_noise(16000, 1.0); // 1 second

    // Action: Process through denoiser
    let denoised = denoiser.process(&noisy_audio);

    // Verify behavior:
    // 1. Output exists and has expected length (accounting for latency)
    assert!(denoised.len() > 0);

    // 2. SNR improved (simple energy comparison)
    let noise_energy_before = calculate_noise_energy(&noisy_audio);
    let noise_energy_after = calculate_noise_energy(&denoised);
    assert!(noise_energy_after < noise_energy_before);
}

// GOOD: Error handling test
#[test]
fn test_audio_capture_continues_without_models() {
    // Simulate missing models
    std::env::set_var("DTLN_MODEL_PATH", "/nonexistent");

    // Action: Try to initialize denoiser
    let result = DtlnDenoiser::new();

    // Verify: Returns error, doesn't panic
    assert!(result.is_err());

    // Verify: CpalBackend can still capture audio
    let backend = CpalBackend::new(); // Should not panic
    assert!(backend.is_ok());
}
```

### TCR Commands for This Feature

```bash
# During spec implementation (quick tests)
bun tcr.ts check "cd src-tauri && cargo test denoiser"

# Full backend tests
bun tcr.ts check "cd src-tauri && cargo test"

# Feature review (with coverage)
bun tcr.ts check "cd src-tauri && cargo +nightly llvm-cov --fail-under-lines 60 --fail-under-functions 60 --ignore-filename-regex '_test\.rs$'"
```

### Coverage Target: 60%

Focus test effort on:
1. **Main success path**: Noisy audio → denoised audio
2. **Primary error path**: Model loading failure → graceful degradation
3. **Critical edge case**: Empty/silent audio handling

Skip testing:
- Internal buffer management
- FFT/IFFT correctness (trust rustfft)
- ONNX runtime internals (trust tract-onnx)

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Use DTLN via ONNX | Native 16kHz support, MIT license, proven quality (DNS Challenge), lightweight (<1M params) | 2025-12-22 |
| Use tract-onnx or ort crate | Pure Rust ONNX inference, tract is audio-focused (Sonos/Snips heritage) | 2025-12-22 |
| Always-on (no toggle) | Simplifies UX, noise suppression should "just work" | 2025-12-22 |
| Graceful degradation on failure | Continue without denoising if model fails to load | 2025-12-22 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-22 | DTLN operates at 16kHz native - matches heycat's pipeline exactly | No resampling needed, simplifies integration |
| 2025-12-22 | DTLN uses two ONNX models with LSTM states | Need to track state between frames |
| 2025-12-22 | tract-onnx (from Sonos) designed for audio/voice inference | Better fit than generic ort crate |
| 2025-12-22 | 512-sample frames with 128-sample shift (75% overlap) | Need overlap-add buffer for smooth output |
| 2025-12-22 | Models available at github.com/breizhn/DTLN/pretrained_model | model_1.onnx, model_2.onnx to bundle |

## Open Questions

- [x] Which ONNX runtime to use? → tract-onnx (primary), ort as fallback
- [x] How to handle model loading failure? → Graceful degradation, log error, continue without denoising
- [ ] Best strategy for bundling ONNX models (include_bytes! vs resources directory)?
- [ ] Should denoiser run in audio callback thread or separate processing thread?

## Files to Modify

### New Files
- `src-tauri/src/audio/denoiser/mod.rs` - Module exports
- `src-tauri/src/audio/denoiser/dtln.rs` - DTLN denoiser implementation
- `src-tauri/src/audio/denoiser/tests.rs` - Behavior tests

### Modified Files
- `src-tauri/Cargo.toml` - Add dependencies
  - `tract-onnx` or `ort` for ONNX inference
  - `rustfft` for FFT operations

- `src-tauri/src/audio/mod.rs` - Export denoiser module

- `src-tauri/src/audio/cpal_backend.rs` - Integrate denoiser
  - Initialize denoiser when starting capture
  - Process audio through denoiser before buffer

### Resource Files
- `src-tauri/resources/dtln/model_1.onnx` - DTLN stage 1 model
- `src-tauri/resources/dtln/model_2.onnx` - DTLN stage 2 model

## References

### DTLN
- [DTLN GitHub (breizhn)](https://github.com/breizhn/DTLN) - Original implementation, ONNX models
- [DTLN Paper](https://arxiv.org/abs/2006.04037) - Technical details on dual-signal approach

### ONNX Inference
- [tract (Sonos)](https://github.com/sonos/tract) - Audio-focused ONNX inference
- [ort (pykeio)](https://github.com/pykeio/ort) - Alternative ONNX Runtime wrapper

### Audio Processing
- [rustfft](https://crates.io/crates/rustfft) - FFT operations in Rust
- [DTLN real_time_processing_onnx.py](https://github.com/breizhn/DTLN/blob/master/real_time_processing_onnx.py) - Reference implementation

---
status: pending
created: 2025-12-22
completed: null
dependencies: ["dtln-model-setup"]
---

# Spec: Core DTLN denoiser implementation

## Description

Implement the core DtlnDenoiser struct that processes audio frames through the DTLN two-stage pipeline. This includes frame buffering, FFT/IFFT operations, ONNX inference for both models, and overlap-add for smooth output. The denoiser must maintain LSTM states between frames for temporal continuity.

## Acceptance Criteria

- [ ] `DtlnDenoiser` struct created with loaded models, frame buffer, and LSTM states
- [ ] `new()` constructor initializes denoiser from loaded ONNX models
- [ ] `process(&mut self, samples: &[f32]) -> Vec<f32>` method implemented
- [ ] Frame buffering: accumulates 512 samples, shifts by 128 (75% overlap)
- [ ] FFT extracts magnitude and phase from input frame
- [ ] Model 1 inference produces magnitude mask
- [ ] Masked magnitude + original phase reconstructed via IFFT
- [ ] Model 2 refines time-domain signal
- [ ] Overlap-add produces continuous output stream
- [ ] LSTM states persist across `process()` calls
- [ ] `reset()` method clears buffers and states for new audio stream

## Test Cases

- [ ] Test: Process silent audio returns silent output
- [ ] Test: Process sine wave preserves frequency content
- [ ] Test: Multiple consecutive calls maintain temporal continuity
- [ ] Test: Reset clears state for new stream
- [ ] Test: Output latency is approximately 32ms (512 samples at 16kHz)
- [ ] Test: Process noisy speech sample and verify noise reduction (SNR improvement)

## Dependencies

- `dtln-model-setup` - Provides loaded ONNX models

## Preconditions

- ONNX models loaded successfully from dtln-model-setup spec
- Audio input is 16kHz mono f32 samples

## Implementation Notes

**Files to create/modify:**
- `src-tauri/src/audio/denoiser/dtln.rs` - DtlnDenoiser implementation

**DTLN Processing Pipeline:**
```
Input frame (512 samples)
  → Apply Hann window
  → FFT (rustfft)
  → Extract magnitude, preserve phase
  → Model 1: magnitude → masked magnitude
  → Reconstruct complex: masked_mag * exp(i*phase)
  → IFFT
  → Model 2: time-domain refinement
  → Overlap-add to output buffer
  → Output: 128 new samples per frame
```

**LSTM State Management:**
- Model 1 has LSTM state (hidden + cell)
- Model 2 has LSTM state (hidden + cell)
- States are inputs to the model and updated from outputs
- Initialize to zeros, update after each frame

**Reference Implementation:**
- See: https://github.com/breizhn/DTLN/blob/master/real_time_processing_onnx.py

## Related Specs

- [dtln-model-setup.spec.md](./dtln-model-setup.spec.md) - Provides models
- [pipeline-integration.spec.md](./pipeline-integration.spec.md) - Consumes this denoiser

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs` (via pipeline-integration)
- Connects to: dtln-model-setup (uses models), pipeline-integration (used by)

## Integration Test

- Test location: `src-tauri/src/audio/denoiser/tests.rs`
- Verification: [ ] Integration test passes

---
status: in-review
created: 2025-12-22
completed: null
dependencies: []
review_round: 1
---

# Spec: ONNX model loading and dependencies

## Description

Set up the ONNX inference infrastructure for DTLN noise suppression. This includes adding required dependencies to Cargo.toml, bundling the DTLN ONNX model files, and creating the model loading function that will be used by the denoiser.

## Acceptance Criteria

- [ ] `tract-onnx` (or `ort`) crate added to Cargo.toml
- [ ] `rustfft` crate added to Cargo.toml for FFT operations
- [ ] DTLN ONNX models (model_1.onnx, model_2.onnx) downloaded and placed in resources directory
- [ ] Model loading function created that loads both ONNX models
- [ ] Loading function returns Result type for graceful error handling
- [ ] Models are embedded via `include_bytes!` or loaded from resources path
- [ ] Unit test verifies models can be loaded successfully

## Test Cases

- [ ] Test: Models load successfully when files exist
- [ ] Test: Loading returns appropriate error when model files missing
- [ ] Test: Models have expected input/output shapes after loading

## Dependencies

None - this is the foundational spec.

## Preconditions

- DTLN ONNX models available from https://github.com/breizhn/DTLN/tree/master/pretrained_model

## Implementation Notes

**Files to create/modify:**
- `src-tauri/Cargo.toml` - Add dependencies
- `src-tauri/resources/dtln/model_1.onnx` - Stage 1 ONNX model
- `src-tauri/resources/dtln/model_2.onnx` - Stage 2 ONNX model
- `src-tauri/src/audio/denoiser/mod.rs` - Model loading functions

**Crate options:**
- Primary: `tract-onnx` - designed for audio inference (Sonos heritage)
- Fallback: `ort` - ONNX Runtime wrapper if tract doesn't work

**Model details:**
- model_1.onnx: Magnitude masking (frequency domain)
- model_2.onnx: Time-domain refinement
- Both have LSTM states that need external handling

## Related Specs

- [dtln-denoiser.spec.md](./dtln-denoiser.spec.md) - Uses loaded models
- [pipeline-integration.spec.md](./pipeline-integration.spec.md) - Final integration

## Integration Points

- Production call site: `src-tauri/src/audio/denoiser/mod.rs` (called during denoiser init)
- Connects to: dtln-denoiser spec (provides models for inference)

## Integration Test

- Test location: `src-tauri/src/audio/denoiser/tests.rs`
- Verification: [ ] Integration test passes

---
status: pending
created: 2025-12-22
completed: null
dependencies: ["dtln-denoiser"]
---

# Spec: Integrate denoiser into audio capture pipeline

## Description

Integrate the DtlnDenoiser into the CpalBackend audio capture pipeline so that all captured audio is processed through noise suppression before reaching the AudioBuffer. This includes initializing the denoiser on startup and implementing graceful degradation if the denoiser fails to load.

## Acceptance Criteria

- [ ] CpalBackend holds an optional DtlnDenoiser instance
- [ ] Denoiser is initialized when audio capture starts
- [ ] Audio samples pass through denoiser after format conversion, before buffering
- [ ] If denoiser initialization fails, audio capture continues without denoising
- [ ] Error is logged when denoiser fails to load (for debugging)
- [ ] Denoiser is reset when starting a new recording/listening session
- [ ] No user-visible errors if denoiser is unavailable
- [ ] Audio latency increase is acceptable (~32ms)

## Test Cases

- [ ] Test: Audio capture works normally when denoiser loads successfully
- [ ] Test: Audio capture continues when ONNX models are missing (graceful degradation)
- [ ] Test: Error is logged when denoiser fails to initialize
- [ ] Test: Denoiser is reset between recording sessions
- [ ] Test: Manual recording with background noise shows audible improvement

## Dependencies

- `dtln-denoiser` - Provides DtlnDenoiser implementation

## Preconditions

- DtlnDenoiser implementation complete and tested
- CpalBackend audio capture working (existing functionality)

## Implementation Notes

**Files to modify:**
- `src-tauri/src/audio/cpal_backend.rs` - Add denoiser integration
- `src-tauri/src/audio/mod.rs` - Export denoiser module

**Integration approach:**
```rust
// In CpalBackend
struct CpalBackend {
    // ... existing fields
    denoiser: Option<DtlnDenoiser>,
}

// In audio callback or post-processing
fn process_samples(&mut self, samples: &[f32]) -> Vec<f32> {
    if let Some(ref mut denoiser) = self.denoiser {
        denoiser.process(samples)
    } else {
        samples.to_vec()
    }
}
```

**Graceful degradation:**
```rust
let denoiser = match DtlnDenoiser::new() {
    Ok(d) => {
        info!("DTLN denoiser initialized successfully");
        Some(d)
    }
    Err(e) => {
        warn!("Failed to initialize denoiser, continuing without noise suppression: {}", e);
        None
    }
};
```

**Logging:**
- Log at INFO level when denoiser initializes successfully
- Log at WARN level when denoiser fails (with error details)
- Use existing crate::info! and crate::warn! macros

## Related Specs

- [dtln-model-setup.spec.md](./dtln-model-setup.spec.md) - Foundation
- [dtln-denoiser.spec.md](./dtln-denoiser.spec.md) - Provides denoiser

## Integration Points

- Production call site: `src-tauri/src/audio/cpal_backend.rs` (CpalBackend::new and audio callback)
- Connects to: dtln-denoiser (uses DtlnDenoiser), AudioBuffer (sends denoised samples)

## Integration Test

- Test location: Manual testing with microphone + background noise
- Verification: [ ] Integration test passes
- Note: Full integration requires manual testing with real audio hardware

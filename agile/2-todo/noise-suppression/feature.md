---
discovery_phase: complete
---

# Feature: Real-time Audio Noise Suppression

**Created:** 2025-12-22
**Owner:** Michael
**Discovery Phase:** not_started

## Description

Add real-time noise suppression to the audio capture pipeline to improve voice recognition accuracy. Currently, heycat captures raw microphone audio like QuickTime, including all background noise (fans, keyboard, ambient sounds). This feature will integrate DTLN (Dual-signal Transformation LSTM Network) noise suppression via ONNX models to filter out background noise before the audio reaches VAD and transcription, similar to how Microsoft Teams handles audio in calls.

## BDD Scenarios

### User Persona
Voice command users who interact with heycat using wake words ("Hey Cat") and spoken commands in various environments. These users may be in home offices with background noise (AC, fans, mechanical keyboards), living rooms with ambient sounds, or other non-studio environments. They expect clear, accurate voice recognition without needing a professional recording setup.

### Problem Statement
When users speak wake words or commands to heycat, background noise is captured along with their voice. This noise can:
1. Interfere with wake word detection accuracy
2. Reduce transcription quality
3. Create a poor user experience compared to apps like Microsoft Teams that suppress noise automatically

The solution is to add always-on noise suppression that filters background noise while preserving speech clarity, using the DTLN deep learning model which operates natively at 16kHz (matching heycat's existing audio pipeline).

```gherkin
Feature: Real-time Audio Noise Suppression

  # Happy Paths
  Scenario: Background noise is suppressed during wake word detection
    Given heycat is listening for the wake word
    And there is background noise (fan, AC, keyboard)
    When I say "Hey Cat"
    Then the wake word is detected accurately
    And background noise does not trigger false positives

  Scenario: Speech clarity is preserved after noise suppression
    Given heycat is recording a voice command
    And there is moderate background noise
    When I speak a command clearly
    Then the transcription accurately captures my speech
    And the noise suppression does not distort my voice

  Scenario: Noise suppression initializes automatically
    Given the heycat application starts
    When audio capture begins
    Then the DTLN denoiser is loaded and active
    And audio is processed through the denoiser before VAD

  # Edge Cases
  Scenario: Graceful degradation if denoiser fails to load
    Given the ONNX model files are missing or corrupted
    When audio capture starts
    Then audio capture continues without noise suppression
    And an error is logged for debugging

  Scenario: Handling extreme noise levels
    Given there is very loud background noise
    When I speak a command
    Then the denoiser attenuates the noise as much as possible
    And speech that is audible above the noise is preserved
```

### Out of Scope
- Echo cancellation (speaker feedback removal) - potential future enhancement
- User toggle to enable/disable noise suppression - always on by default
- Custom model training or fine-tuning
- Multiple noise suppression algorithm choices
- Real-time visualization of noise reduction levels
- Noise suppression for playback/output audio (only for microphone input)

### Assumptions
- DTLN ONNX models (~2-4MB total) will be bundled with the application
- The `tract-onnx` or `ort` crate can correctly handle LSTM stateful inference in Rust
- 32ms processing latency is acceptable for wake word and voice command use cases
- The existing audio pipeline's 16kHz sample rate is suitable (DTLN is 16kHz native)
- Users' hardware can run ONNX inference in real-time (tested to work on Raspberry Pi 3 B+)
- FFT/IFFT operations can be performed efficiently using `rustfft` crate

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] DTLN ONNX models bundled with the application
- [ ] DtlnDenoiser processes audio at 16kHz with 32ms latency
- [ ] Noise suppression integrated into audio capture pipeline
- [ ] Graceful degradation if denoiser fails to load
- [ ] Wake word detection works accurately with background noise
- [ ] Speech clarity preserved after noise suppression

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

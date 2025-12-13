# Feature: Replace Whisper with Parakeet v3 + Streaming

**Created:** 2025-12-13
**Owner:** Claude
**Discovery Phase:** complete

## Description

Replace the current Whisper-based transcription engine with NVIDIA's Parakeet model, supporting both batch transcription (TDT v3 - multilingual) and real-time streaming transcription (EOU). This modernizes the transcription layer with a faster, more accurate model that supports 25 European languages with auto-detection, while adding the capability for live transcription during recording.

## BDD Scenarios

### User Persona

A power user who frequently uses voice-to-text for capturing thoughts, dictating notes, or inputting text hands-free. They may speak multiple languages and want to see their speech transcribed in real-time rather than waiting until after recording stops.

### Problem Statement

The current Whisper-based transcription only supports batch mode (transcription after recording) and is English-focused. Users want:
1. Real-time feedback while speaking (streaming transcription)
2. Better multilingual support (25 languages with auto-detection)
3. Faster transcription performance

```gherkin
Feature: Parakeet Transcription Engine

  Background:
    Given the user has downloaded the Parakeet TDT model
    And the model is loaded successfully

  Scenario: Batch transcription after recording
    Given the user is using batch transcription mode
    When the user starts a recording
    And the user speaks "Hello, this is a test"
    And the user stops the recording
    Then the system should transcribe the audio using ParakeetTDT
    And the transcribed text should contain "Hello, this is a test"
    And a transcription_completed event should be emitted

  Scenario: Real-time streaming transcription
    Given the user has downloaded the Parakeet EOU model
    And the user is using streaming transcription mode
    When the user starts a recording
    And the user speaks continuously
    Then the system should emit transcription_partial events
    And the UI should display real-time transcribed text
    And when the user stops recording, a final transcription_completed event should be emitted

  Scenario: Multilingual transcription
    Given the user is using batch transcription mode
    When the user records speech in German
    Then the system should auto-detect the language
    And the transcribed text should be accurate German text

  Scenario: Model download for TDT
    Given the user does not have the TDT model downloaded
    When the user initiates a TDT model download
    Then the system should download all required ONNX files
    And show download progress for each file
    And store the files in the parakeet-tdt model directory

  Scenario: Model download for EOU (streaming)
    Given the user does not have the EOU model downloaded
    When the user initiates an EOU model download
    Then the system should download all required ONNX files
    And store the files in the parakeet-eou model directory

  Scenario: Transcription mode switching
    Given both TDT and EOU models are downloaded
    When the user switches from batch to streaming mode in settings
    Then the system should use the EOU model for subsequent recordings
    And emit transcription_partial events during recording
```

### Out of Scope

- Speaker diarization (identifying different speakers)
- CUDA/GPU acceleration (using CPU execution for stability)
- Real-time language switching mid-sentence
- Whisper fallback option (complete replacement, not parallel support)
- Custom vocabulary or fine-tuning

### Assumptions

- parakeet-rs library v0.2 is stable and production-ready
- ONNX models are available on HuggingFace (v2 confirmed, v3 may need verification)
- CPU execution provides acceptable performance on Apple Silicon
- Users accept downloading two separate models for batch vs streaming modes

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Whisper dependency removed, parakeet-rs added
- [ ] TDT model can be downloaded and used for batch transcription
- [ ] EOU model can be downloaded and used for streaming transcription
- [ ] Streaming mode shows real-time text during recording
- [ ] Multilingual audio transcribes correctly with auto-detection
- [ ] Frontend settings allow model downloads and mode selection
- [ ] All existing tests pass or are updated

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

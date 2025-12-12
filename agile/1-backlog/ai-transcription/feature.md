# Feature: AI Transcription

**Created:** 2025-12-12
**Owner:** Michael
**Discovery Phase:** complete

## Description

Enable automatic transcription of voice recordings using local Whisper models. When a user stops recording, the audio is transcribed and the text is copied to clipboard. This is the foundation for future AI chat and voice command features.

## BDD Scenarios

### User Persona

**Power User** - Someone who frequently or constantly needs to transcribe audio recordings. They want to use voice instead of keyboard/mouse to speed up workflows.

### Problem Statement

Users are frustrated navigating multiple applications and inputting data using keyboard and mouse. They want voice control to increase speed and reduce friction in their workflows.

### Gherkin Scenarios

```gherkin
Feature: AI Transcription

  As a power user who frequently uses voice input
  I want my recordings to be transcribed and copied to clipboard
  So that I can quickly capture voice input as text without manual typing

  Background:
    Given the Whisper Large v3 Turbo model is downloaded
    And the model is stored in the app models directory

  Scenario: Successful transcription after recording
    Given I have completed a voice recording
    When the recording stops
    Then transcription should start automatically
    And a loading indicator should be displayed
    And I should not be able to start a new recording until transcription completes
    When transcription completes successfully
    Then the transcribed text should be copied to clipboard
    And a brief success notification should be shown
    And I should be able to start a new recording

  Scenario: Download model via UI button
    Given the Whisper model is not downloaded
    When I view the model download UI
    Then I should see a button to download the Large v3 Turbo model
    When I click the download button
    Then the model should start downloading
    And the button should show a "downloading..." state
    When the download completes
    Then the button should indicate the model is available
    And I should be able to start recording

  Scenario: Attempt to record without model
    Given the Whisper model is not downloaded
    When I try to start a recording
    Then recording should be blocked
    And I should see a notification that a model is required

  Scenario: Transcription fails
    Given I have completed a voice recording
    And the Whisper model is available
    When transcription fails (corrupted audio, Whisper error, etc.)
    Then an error notification should be displayed
    And the clipboard should not be modified
    And I should be able to start a new recording
```

### Out of Scope

- Language selection (use auto-detect only)
- Multiple model support (Large v3 Turbo only for MVP)
- Download progress UI (just "downloading..." state)
- Model file management (no delete/re-download)
- AI chat integration (separate feature: `ai-chat`)
- Voice command execution (separate feature: `voice-commands`)

### Assumptions

1. The app already has hotkey-triggered audio recording working (from `global-hotkey-recording` feature)
2. Recordings are saved as WAV files in `{app_data_dir}/recordings/`
3. whisper.cpp Rust bindings are available (e.g., `whisper-rs` crate)
4. The frontend can show notifications (existing pattern to follow)
5. The download can be implemented using reqwest or similar HTTP client

### Technical Details

- **Model**: `ggml-large-v3-turbo.bin` (1.5 GB)
- **Source**: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`
- **Storage**: `{app_data_dir}/models/` (alongside existing `recordings/` folder)
- **Reference**: VoiceInk project at `/Users/michaelhindley/Documents/git/VoiceInk`

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Whisper Large v3 Turbo model can be downloaded via UI
- [ ] Recording is blocked if model not available
- [ ] Transcription starts automatically when recording stops
- [ ] Transcription copies text to clipboard on success
- [ ] Loading indicator shown during transcription
- [ ] Error notification shown on failure
- [ ] Success notification shown on completion

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

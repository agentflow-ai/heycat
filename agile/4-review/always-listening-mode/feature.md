---
discovery_phase: complete
---

# Feature: Always Listening Mode

**Created:** 2025-12-14
**Owner:** Michael Hindley
**Discovery Phase:** complete

## Description

Enable hands-free voice activation for heycat using a wake word ("Hey Cat"). When enabled, the app continuously listens for the wake word and automatically starts recording when detected. Supports both transcription mode (wake word → record → transcribe) and command mode (wake word → command → execute). This feature reduces friction for power users and improves accessibility for users who cannot easily use keyboard shortcuts or mouse clicks.

## BDD Scenarios

### User Persona
A power user or professional who uses heycat frequently for work tasks (meetings, content creation, documentation). They may also be a hands-busy user who needs voice control because their hands are occupied with other activities like cooking, working on a project, or multitasking.

### Problem Statement
Users face friction when activating recording - having to click or use a keyboard shortcut interrupts their flow and breaks concentration. Additionally, users with accessibility needs find manual activation difficult or impossible. This creates barriers to seamless voice capture when it's needed most.

```gherkin
Feature: Always Listening Mode

  Scenario: Happy path - Wake word triggers recording
    Given always-listening mode is enabled
    And the microphone is available
    When the user says the wake word "Hey Cat"
    Then the CatOverlay visual indicator shows recording has started
    And the app begins capturing audio
    When the user speaks their content
    And silence is detected for a few seconds
    Then recording automatically stops
    And the captured audio is transcribed

  Scenario: Happy path - Wake word triggers command
    Given always-listening mode is enabled
    And the microphone is available
    When the user says "Hey Cat" followed by a command phrase
    Then the CatOverlay visual indicator shows recording has started
    And the app captures the command
    And the command is interpreted and executed

  Scenario: Error case - False activation cancelled by user
    Given always-listening mode is enabled
    And the app incorrectly detected the wake word
    And recording has started
    When the user says "cancel" or "nevermind"
    Then recording is stopped
    And no transcription is saved

  Scenario: Error case - False activation auto-cancelled
    Given always-listening mode is enabled
    And the app incorrectly detected the wake word
    And recording has started
    When no speech is detected within the timeout period
    Then recording is automatically cancelled
    And no transcription is saved

  Scenario: Error case - Microphone unavailable
    Given always-listening mode is enabled
    When the microphone becomes unavailable or is in use by another application
    Then a status indicator shows always-listening is unavailable
    And wake word detection is paused
```

### Out of Scope
- Custom wake words (ability to change from default)
- Multi-language wake word detection
- Cloud-based wake word detection (all processing is local)
- Advanced command grammar (complex multi-step voice commands)
- Audio playback for feedback (visual-only in MVP, using existing CatOverlay)
- CPU/latency optimization (deferred post-MVP)
- Advanced VAD algorithms (using simple energy-based silence detection)

### Assumptions
- Wake word detection happens entirely on-device, no network required
- User has already granted microphone permissions to the app
- The existing Parakeet transcription model is used for wake word detection (small-window batching)
- Only one fixed wake word ("Hey Cat") will be supported in this initial implementation
- Visual feedback uses existing CatOverlay component (same as hotkey-triggered recording)

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Wake word "Hey Cat" triggers recording when listening mode is enabled
- [ ] Visual indicator (CatOverlay) shows when recording starts
- [ ] Recording auto-stops after configurable silence period
- [ ] User can cancel false activations with "cancel" or "nevermind"
- [ ] Settings persist across app restarts
- [ ] Listening mode integrates cleanly with existing hotkey recording

## Definition of Done

- [x] All specs completed
- [x] Technical guidance finalized
- [x] Code reviewed and approved
- [x] Tests written and passing
- [x] Documentation updated

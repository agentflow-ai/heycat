---
discovery_phase: complete
---

# Feature: Voice Commands

**Created:** 2025-12-12
**Owner:** Michael
**Discovery Phase:** not_started

## Description

Parse voice transcriptions as commands and execute actions. This enables hands-free control of applications and workflows - opening apps, clicking, typing, navigating, and automating multi-step workflows.

This feature builds on `ai-transcription` and represents the full vision of voice-controlled desktop interaction.

## BDD Scenarios

### User Persona

Power users and developers who want voice control during coding, research, or multi-app workflows to reduce context switching. Also serves productivity enthusiasts seeking voice shortcuts for repetitive tasks, and accessibility users who need hands-free control. Technical level ranges from intermediate to advanced - comfortable with customizing commands and understanding system permissions.

### Problem Statement

Current keyboard/mouse workflows create friction in three key areas:
1. **Context switching friction** - Moving hands between keyboard, mouse, and different applications breaks flow and slows down work
2. **Repetitive task fatigue** - Executing the same multi-step sequences (open app, navigate, click, type) is tedious and error-prone
3. **Multitasking limitations** - Cannot control computer while hands are occupied (eating, holding documents, on a call, etc.)

Users want to speak natural commands like "open Slack" or "switch to browser and search for X" without touching the keyboard or mouse.

```gherkin
Feature: Voice Commands

  Background:
    Given the app is running and listening for voice input
    And voice-to-text transcription is active

  # Happy Paths

  Scenario: Open application by voice
    Given the user has a command registered for "open slack"
    When the user says "open slack"
    Then Slack application launches and receives focus

  Scenario: Type text by voice
    Given the user has a command registered for "type"
    And a text editor is focused
    When the user says "type hello world"
    Then "hello world" is typed into the focused application

  Scenario: Execute multi-step workflow
    Given the user has a workflow command "search google for"
    When the user says "search google for rust tauri tutorial"
    Then the browser opens
    And navigates to Google
    And enters "rust tauri tutorial" in the search box

  Scenario: Execute system control command
    Given the user has a command registered for "volume up"
    When the user says "volume up"
    Then the system volume increases

  Scenario: Execute custom user-defined command
    Given the user has defined a custom command "deploy app" that runs a shell script
    When the user says "deploy app"
    Then the associated shell script executes

  # Default Fallback Behavior

  Scenario: Unrecognized phrase copies to clipboard
    Given the user speaks a phrase that doesn't match any command
    When the transcription "random unrecognized phrase" is received
    Then the text is copied to the clipboard
    And no error is shown to the user

  # Error Cases

  Scenario: App not found for open command
    Given the user has a parameterized command "open" that launches applications
    When the user says "open nonexistent-app"
    Then a notification indicates the app was not found

  Scenario: Ambiguous command match
    Given the user has commands "open slack" and "open slackbot"
    When the user says "open slack"
    And both commands match with similar confidence
    Then a disambiguation prompt appears
    And the user can select the intended command

  # Matching Behavior

  Scenario: Fuzzy match with minor variations
    Given the user has a command registered for "open slack"
    When the user says "open Slack" (different capitalization)
    Then the command matches successfully
    And Slack application launches

  # Multiple Commands

  Scenario: Multiple commands in one utterance execute sequentially
    Given the user has commands "open slack" and "type hello"
    When the user says "open slack and type hello"
    Then "open slack" executes first
    And waits for completion
    Then "type hello" executes second
```

### Out of Scope

- **LLM/AI reasoning** - No AI interpretation of user intent; only pattern/fuzzy matching on predefined commands
- **Voice synthesis feedback** - No text-to-speech responses back to the user
- **Learning/adaptation** - No learning user patterns or adapting to command usage over time
- **Non-macOS implementations** - Core architecture will be cross-platform compatible, but only macOS concrete implementations will be built initially (Windows/Linux deferred)

### Assumptions

- **Transcription feature works** - The ai-transcription feature is complete and provides reliable text output to the command parser
- **Commands are pre-configured** - Users will configure commands through a settings UI or config file before use
- **Sequential execution for multiple commands** - If a user speaks multiple commands in one utterance, they execute in blocking/sequential order based on match order

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Commands can be registered with trigger phrases and associated actions
- [ ] Transcribed text is matched against registered commands using fuzzy matching
- [ ] Matched commands execute their associated actions (open app, type text, run workflow, system control)
- [ ] Unmatched text falls through to clipboard copy (default behavior)
- [ ] Ambiguous matches prompt user for disambiguation
- [ ] Multiple commands in one utterance execute sequentially

## Definition of Done

- [x] All specs completed
- [x] Technical guidance finalized
- [x] Code reviewed and approved
- [x] Tests written and passing
- [x] Documentation updated

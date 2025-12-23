---
discovery_phase: complete
---

# Feature: Window Context Detection for Context-Sensitive Commands

**Created:** 2025-12-23
**Owner:** Michael
**Discovery Phase:** not_started

## Description

Enable heycat to detect the currently active window and apply context-sensitive voice commands and dictionary entries. Users can define "window contexts" that match applications by name and/or window title patterns, with configurable merge/replace behavior for how context-specific entries interact with global ones. This provides a foundation for app-aware voice workflows.

## BDD Scenarios

### User Persona
A general productivity user (non-technical) who uses heycat for voice-driven workflows across multiple applications. They want voice commands to behave differently depending on which application is currently active, without needing to manually switch modes or remember complex command variations.

### Problem Statement
Users face three key challenges with the current global-only command approach:
1. **Command conflicts**: The same voice trigger may need to do different things in different apps (e.g., "save" might mean different actions in a text editor vs. a design tool)
2. **Generic commands**: Global commands are too broad and don't leverage app-specific functionality
3. **Context switching overhead**: Users must mentally track which commands work where, increasing cognitive load and reducing voice input efficiency

Solving this enables future features like app-specific dictation modes, specialized vocabulary per application, and smarter context-aware workflows.

```gherkin
Feature: Window Context Detection for Context-Sensitive Commands

  # === HAPPY PATHS ===

  Scenario: Create a window context from Settings
    Given I am on the Settings page
    When I navigate to the "Window Contexts" section
    And I click "New Context"
    And I enter "Slack" as the app name
    And I set override mode to "Merge"
    And I save the context
    Then a new window context "Slack" is created
    And it appears in the contexts list

  Scenario: Assign commands to a window context
    Given I have a window context "VS Code" created
    And I have global commands "save" and "undo" defined
    When I edit the "VS Code" context
    And I assign commands "format code" and "run tests" to this context
    Then the context shows 2 assigned commands
    And these commands are only active when VS Code is focused

  Scenario: Voice command uses context-specific command
    Given I have a window context "Slack" with command "send message"
    And I have a global command "send message" that does something different
    And the context is set to "Replace" mode
    When Slack is the active window
    And I speak "send message"
    Then the Slack-specific "send message" command executes
    And the global command is not triggered

  Scenario: Context merges with global commands
    Given I have a window context "Chrome" with command "bookmark page"
    And I have global commands "scroll down" and "go back"
    And the context is set to "Merge" mode
    When Chrome is the active window
    And I speak "scroll down"
    Then the global "scroll down" command executes
    And "bookmark page" is also available

  Scenario: Title pattern matches specific window
    Given I have a window context "Chrome - Gmail" with title pattern ".*Gmail.*"
    And I have a window context "Chrome" matching app name only
    When Chrome is focused with title "Inbox - Gmail - Google Chrome"
    Then the "Chrome - Gmail" context is matched (more specific)
    And its commands are used

  Scenario: Bulk assign commands to context
    Given I am editing window context "Figma"
    When I click "Assign Commands"
    And I select multiple commands from the list
    And I confirm the selection
    Then all selected commands are assigned to "Figma"

  Scenario: Context priority resolves overlapping matches
    Given I have context "Chrome" with priority 1
    And I have context "Chrome - Docs" with title pattern ".*Google Docs.*" and priority 2
    When Chrome is focused with title "Untitled - Google Docs"
    Then context "Chrome - Docs" is matched (higher priority)

  # === ERROR CASES ===

  Scenario: No matching context falls back to global
    Given I have a window context "Slack" configured
    And I have global commands defined
    When "Finder" is the active window (no context defined)
    And I speak a command
    Then global commands are used
    And no error is shown

  Scenario: Window detection fails gracefully
    Given window detection encounters a macOS API error
    When I start recording
    Then the app falls back to global commands
    And a warning is logged (not shown to user)
    And recording continues normally

  Scenario: Invalid regex pattern shows validation error
    Given I am creating a new window context
    When I enter an invalid regex pattern "[unclosed"
    And I try to save
    Then a validation error is shown
    And the context is not saved
    And I can correct the pattern

  Scenario: Ambiguous context uses highest priority
    Given I have two contexts that both match current window
    When a voice command is triggered
    Then the context with highest priority is used
    And the app does not prompt for disambiguation
```

### Out of Scope
- **Windows/Linux support**: This feature is macOS-only; cross-platform window detection is deferred
- **Auto-learning contexts**: No AI/ML to automatically suggest contexts based on usage patterns
- **Cross-device sync**: Window contexts are stored locally only, no cloud synchronization
- **Per-document contexts**: Match by app name and window title only, not by file/document content

### Assumptions
- **Accessibility permissions granted**: User has already granted macOS accessibility permissions (required for window detection APIs)
- **Single active context**: Only one window context can be active at a time (the highest-priority match)
- **Immediate detection**: Window focus changes are detected within ~250ms via background polling

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Window contexts can be created, edited, and deleted from Settings UI
- [ ] Active window is continuously monitored and matched against defined contexts
- [ ] Context-specific commands are used when a matching context is active
- [ ] Merge/Replace mode works correctly for command resolution
- [ ] Title pattern matching with regex is supported
- [ ] Priority-based resolution handles overlapping contexts
- [ ] Graceful fallback to global commands when no context matches or detection fails

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

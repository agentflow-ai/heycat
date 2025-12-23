---
discovery_phase: complete
---

# Feature: Block Cancel Key Propagation

**Created:** 2025-12-23
**Owner:** Michael
**Discovery Phase:** not_started

## Description

When users cancel a hotkey recording using double-escape, the Escape key events currently propagate to other applications. This causes unintended side effects like closing dialogs or exiting modes in terminals/editors. This feature will consume the Escape key events during recording so they don't reach other applications.

## BDD Scenarios

### User Persona
Any user who records hotkeys while other applications are focused. This includes users working in terminals, IDEs, browsers, or any application where Escape has special meaning.

### Problem Statement
When cancelling a recording with double-escape, the Escape key presses also reach other applications, causing unintended actions (e.g., closing dialogs, exiting modes in terminals/editors). Users expect the cancel action to be non-destructive to their workflow.

```gherkin
Feature: Block Cancel Key Propagation

  Scenario: Happy path - Cancel recording without affecting other apps
    Given the user is actively recording audio
    And another application is focused (e.g., terminal, IDE)
    When the user presses Escape twice within 300ms
    Then the recording is cancelled
    And the Escape key events are NOT sent to the focused application

  Scenario: Escape passes through when not recording
    Given the user is NOT actively recording
    And another application is focused
    When the user presses Escape
    Then the Escape key event IS sent to the focused application normally

  Scenario: Error case - Key blocking fails
    Given the user is actively recording audio
    And key event blocking cannot be established (e.g., permissions issue)
    When the user presses Escape twice to cancel
    Then the recording is still cancelled
    And the user is notified that key blocking failed
    And the Escape key events may reach other applications
```

### Out of Scope
- Windows/Linux support (this feature is macOS-only using CGEventTap)
- Blocking keys other than Escape during recording
- Blocking Escape key when not actively recording

### Assumptions
- Accessibility permissions are already granted (required for CGEventTap to function)
- CGEventTap's DefaultTap mode works as expected for consuming/blocking events

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] CGEventTap operates in DefaultTap mode allowing event consumption
- [ ] Escape key events are blocked during active recording
- [ ] Escape key events pass through normally when not recording
- [ ] User is notified if key blocking cannot be established
- [ ] Recording functionality works regardless of blocking capability

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

---
discovery_phase: complete
---

# Feature: Ui Redesign

**Created:** 2025-12-17
**Owner:** Michael
**Discovery Phase:** not_started

## Description

Complete UI redesign for HeyCat desktop voice assistant app. Implements a new design system based on the HeyCat mascot (orange cat with teal accents), featuring a sidebar + main content layout, command palette, and comprehensive component library. See `ui.md` for full design specifications.

## BDD Scenarios

### User Persona
Desktop users ranging from general consumers to power users who want a voice assistant for recording, transcription, and voice commands. Power users expect keyboard shortcuts and efficient navigation. All users need clear visual feedback for app states (idle, listening, recording, processing).

### Problem Statement
The current UI needs a complete overhaul addressing:
- **Dated appearance** - doesn't match modern desktop app standards
- **Poor usability** - users struggle to find features and understand app state
- **Missing feature UI** - many features lack proper interfaces
This redesign establishes a proper UI architecture foundation before adding more features.

```gherkin
Feature: HeyCat UI Redesign

  # Recording Flow
  Scenario: Happy path - User records and gets transcription
    Given the app is in idle state
    And a microphone is available
    When the user clicks "Start Recording" or presses ⌘⇧R
    Then the status pill changes to red "Recording" with pulse animation
    And the footer shows recording duration timer
    When the user stops recording
    Then the status changes to amber "Processing" with spinner
    And a toast notification appears with transcription result

  Scenario: Happy path - User cancels recording
    Given the app is recording
    When the user presses Esc twice
    Then the recording is cancelled without saving
    And the app returns to idle state

  # Navigation & Layout
  Scenario: Happy path - User navigates via sidebar
    Given the app is open
    When the user clicks a sidebar navigation item
    Then the main content area updates to show that page
    And the active sidebar item is highlighted with orange/cream background

  Scenario: Happy path - User uses command palette
    Given the app is open
    When the user presses ⌘K
    Then the command palette overlay appears
    And focus is in the search input
    When the user types to filter and selects a command
    Then the command executes and palette closes

  # Settings & Configuration
  Scenario: Happy path - User changes audio device
    Given the user is on Settings > Audio tab
    When the user selects a different input device from dropdown
    Then the audio level meter shows input from the new device
    And the preference is saved

  Scenario: Happy path - User downloads transcription model
    Given the model is not installed
    When the user clicks "Download Model"
    Then a progress bar shows download progress
    And when complete, the status shows "Ready"

  # Voice Commands
  Scenario: Happy path - User creates voice command
    Given the user is on the Commands page
    When the user clicks "+ New Command"
    Then a modal appears with trigger phrase and action fields
    When the user fills the form and saves
    Then the command appears in the list with toggle enabled

  # Error Cases
  Scenario: Error - Microphone not available
    Given no microphone is connected or permission denied
    When the user tries to start recording
    Then an error modal appears explaining the issue
    And provides "Open System Preferences" and "Try Again" buttons

  Scenario: Error - Model download fails
    Given the user is downloading the model
    When the download fails due to network error
    Then an inline error message appears
    And provides "Retry" button and manual download link

  Scenario: Error - Recording fails to start
    Given the app is in idle state
    When recording fails to initialize
    Then an error toast notification appears
    And the app remains in idle state with guidance
```

### Out of Scope
- Backend/Rust changes (frontend-only redesign)
- New feature functionality (only UI for existing features)

### Assumptions
- Existing Tauri commands work correctly and provide required data
- Design specifications in `ui.md` are approved and serve as source of truth
- Project will use Tailwind CSS, Radix UI, Framer Motion, and Lucide icons as specified

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria in individual spec files. See `ui.md` for complete design specs.

- [ ] Design system established with HeyCat brand colors, typography, and tokens
- [ ] New layout implemented (header + sidebar + content + footer)
- [ ] All 4 pages functional (Dashboard, Recordings, Commands, Settings)
- [ ] Command palette (⌘K) working with navigation and actions
- [ ] Status pill shows all app states (Idle, Listening, Recording, Processing)
- [ ] Toast notifications for transcription results and errors
- [ ] Dev toggle allows switching between old/new UI during development
- [ ] All existing functionality preserved (recording, transcription, commands, settings)
- [ ] Legacy CSS and components removed (~2,400 lines)

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

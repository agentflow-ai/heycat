---
discovery_phase: complete
---

# Feature: View Recordings

**Created:** 2025-12-01
**Owner:** Michael
**Discovery Phase:** not_started

## Description

A history view accessible from a sidebar menu that displays all user recordings. Users can browse recordings (showing filename, duration, date), filter by date range and duration, expand entries to see additional metadata, and open recordings in an external player. Error states are handled gracefully with inline indicators.

## BDD Scenarios

### User Persona
General user with average technical proficiency who creates recordings in heycat and needs to access them later.

### Problem Statement
Users can record audio but have no way to view or access their recordings within the app. Currently they must manually navigate to the app data folder to find recordings. This is a core part of the recording lifecycle - without viewing capability, the recording feature is incomplete.

```gherkin
Feature: View Recordings

  Scenario: View recordings list
    Given the user has made one or more recordings
    And the app is open
    When the user clicks the "History" tab in the sidebar menu
    Then they see a list of all recordings
    And each entry shows filename, duration, and date

  Scenario: View recording details
    Given the user is viewing the recordings list
    When the user clicks on a recording entry
    Then the entry expands to show additional metadata
    And the user can open the recording from the expanded view

  Scenario: Filter recordings by date
    Given the user is viewing the recordings list
    When the user applies a date range filter
    Then only recordings within that date range are displayed

  Scenario: Filter recordings by duration
    Given the user is viewing the recordings list
    When the user applies a duration filter
    Then only recordings matching the duration criteria are displayed

  Scenario: No recordings exist
    Given the user has no recordings
    When the user clicks the "History" tab in the sidebar menu
    Then they see a message indicating no recordings exist

  Scenario: Filter returns no results
    Given the user is viewing the recordings list
    When the user applies filters that match no recordings
    Then they see a message indicating the current filters have no results

  Scenario: Recording data is corrupted or missing
    Given a recording file is corrupted or missing
    When the user views the recordings list
    Then that recording still appears in the list with an error indicator
    And the error details are logged to the frontend console
    And any OS-level file errors are logged to the Tauri backend console

  Scenario: Recording metadata is incomplete
    Given a recording has incomplete or invalid metadata
    When the user views the recordings list
    Then the available data is shown
    And missing fields display an inline error message
```

### Out of Scope
- Deleting recordings
- Exporting recordings
- Editing recording metadata
- Playing recordings inline (will open in external player)
- Search by filename/text
- Bulk operations (selecting multiple recordings)

### Assumptions
- Recordings are stored in the Tauri app data directory (location controlled by backend)
- Metadata is derived from file system metadata (creation date, file size, etc.) - no separate metadata storage
- Sidebar menu does not exist and needs to be created (simple, temporary implementation)
- Primary target is macOS, but code should be written with Windows compatibility in mind (cannot test Windows currently)
- Recording feature already saves files with extractable duration information

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Sidebar menu with History tab exists
- [ ] Recordings list displays filename, duration, and date
- [ ] Clicking a recording expands to show full metadata
- [ ] User can open recording from expanded view (external player)
- [ ] Filter by date range works
- [ ] Filter by duration works
- [ ] Empty state shown when no recordings exist
- [ ] Empty state shown when filters match nothing
- [ ] Corrupted/missing recordings show error indicator
- [ ] Errors logged to frontend and backend consoles

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

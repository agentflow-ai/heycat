---
discovery_phase: complete
---

# Feature: Dictionary Improvements

**Created:** 2025-12-22
**Owner:** Michael
**Discovery Phase:** not_started

## Description

Add per-entry configuration options to dictionary entries, allowing users to customize expansion behavior. Each entry will have an expandable settings panel with:

1. **Suffix field** - A freeform text field (max 5 characters) to append after the expansion (e.g., ".", "!", "?")
2. **Auto-enter toggle** - When enabled, simulates an enter keypress after the expansion is typed

This removes the need for users to manually add punctuation or press enter after expansions, making the dictionary feature more efficient.

## BDD Scenarios

### User Persona
All users of heycat who use dictionary expansion. These are general users who want efficient text expansion without manual post-processing.

### Problem Statement
When dictionary entries are expanded, users sometimes want punctuation included and sometimes don't. Additionally, some expansions should automatically trigger an enter key press. Currently, users must manually add punctuation or press enter after each expansion, which defeats the purpose of quick text expansion. Users need per-entry configuration to control this behavior.

```gherkin
Feature: Dictionary Entry Expansion Settings

  Scenario: Configure punctuation suffix on an existing entry
    Given I am on the Dictionary page with an existing entry "brb" → "be right back"
    When I click the settings icon on the entry
    And I enter "." in the suffix text field
    And I save the entry
    Then the entry is updated with the suffix setting

  Scenario: Configure auto-enter on an existing entry
    Given I am on the Dictionary page with an existing entry "sig" → "Best regards, Michael"
    When I click the settings icon on the entry
    And I enable the "Press enter after expansion" toggle
    And I save the entry
    Then the entry is updated with the auto-enter setting

  Scenario: Create new entry with punctuation suffix and auto-enter
    Given I am on the Dictionary page
    When I add a new entry with trigger "ty" and expansion "Thank you"
    And I click the settings icon on the new entry form
    And I enter "!" in the suffix text field
    And I enable the "Press enter after expansion" toggle
    And I save the entry
    Then the entry is created with both suffix and auto-enter settings

  Scenario: Expansion applies punctuation suffix
    Given a dictionary entry "brb" → "be right back" with suffix "."
    When the transcription service expands "brb"
    Then the output is "be right back."

  Scenario: Expansion triggers auto-enter
    Given a dictionary entry "sig" → "Best regards, Michael" with auto-enter enabled
    When the transcription service expands "sig"
    Then the output includes the expansion text
    And an enter keypress is simulated

  Scenario: Expansion with both suffix and auto-enter
    Given a dictionary entry "done" → "Task completed" with suffix "." and auto-enter enabled
    When the transcription service expands "done"
    Then the output is "Task completed."
    And an enter keypress is simulated

  Scenario: Error - Invalid punctuation suffix (too long)
    Given I am on the Dictionary page editing an entry
    When I click the settings icon on the entry
    And I enter a suffix longer than 5 characters
    Then a validation error is displayed
    And the entry is not saved
```

### Out of Scope
- Import/export dictionary entries
- Bulk editing of settings across multiple entries
- Advanced expansion options (capitalize first letter, spacing)
- Conditional expansions (different behavior based on context)

### Assumptions
- The existing dictionary UI and storage will be extended (not replaced)
- Suffix and auto-enter settings default to empty/off for existing entries (backward compatible)
- Keyboard simulation for auto-enter needs to be implemented as part of this feature

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Dictionary entries support optional suffix field (max 5 chars)
- [ ] Dictionary entries support optional auto-enter toggle
- [ ] UI shows expandable settings panel per entry
- [ ] Expander applies suffix when configured
- [ ] Expander triggers enter keypress when auto-enter enabled
- [ ] Existing entries remain backward compatible (no suffix, no auto-enter)

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

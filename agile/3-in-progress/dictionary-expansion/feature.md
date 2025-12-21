---
discovery_phase: complete
---

# Feature: Dictionary Expansion

**Created:** 2025-12-21
**Owner:** Michael
**Discovery Phase:** not_started

## Description

Add a user-managed dictionary system that allows users to define custom text expansions applied to transcription output. Users can add trigger words that automatically expand to full phrases (e.g., "brb" → "be right back"), correct specialized vocabulary, and fix commonly mis-transcribed names and acronyms. The dictionary is managed through a dedicated UI page and expansions are applied post-transcription with case-insensitive, whole-word matching.

## BDD Scenarios

### User Persona
Both technical and non-technical users who use heycat for audio transcription. They range from developers working with technical content to general users capturing meeting notes or other spoken content.

### Problem Statement
Users experience inaccurate transcriptions due to:
- **Specialized vocabulary**: Domain-specific terms (medical, legal, technical jargon) are not recognized correctly
- **Names and proper nouns**: People names, company names, and product names are transcribed incorrectly
- **Acronyms and abbreviations**: Industry acronyms aren't recognized or expanded properly
- **Keyword expansion**: Users want the ability to expand short keywords into full strings (e.g., "brb" → "be right back")

This is important because competing transcription tools already offer custom dictionary/vocabulary functionality.

```gherkin
Feature: Dictionary Expansion

  # Happy Path - Manual Entry
  Scenario: User adds a dictionary entry that auto-applies to transcriptions
    Given I am on the Dictionary page
    When I add a new entry with trigger "brb" and expansion "be right back"
    Then the entry is saved to my dictionary
    And future transcriptions containing "brb" are expanded to "be right back"

  # Happy Path - Real-time Suggestion
  Scenario: User saves a suggested correction during transcription
    Given I am viewing a transcription result
    And the transcription contains an unrecognized term "Anthropic"
    When the system suggests a correction for the term
    And I confirm the suggestion to add it to my dictionary
    Then "Anthropic" is saved to my dictionary for future transcriptions

  # Happy Path - Case-insensitive Matching
  Scenario: Dictionary entries match regardless of case
    Given I have a dictionary entry with trigger "api" expanding to "Application Programming Interface"
    When a transcription contains "API" or "Api" or "api"
    Then all variations are expanded to "Application Programming Interface"

  # Happy Path - Whole Word Matching
  Scenario: Dictionary entries only match whole words
    Given I have a dictionary entry with trigger "cat" expanding to "category"
    When a transcription contains "concatenate"
    Then "concatenate" is not modified
    But "cat" alone would be expanded to "category"

  # Error Case - Invalid Entry Format
  Scenario: User enters an invalid dictionary entry
    Given I am on the Dictionary page
    When I try to add an entry with an empty trigger
    Then I see an error message explaining the trigger cannot be empty
    And the entry is not saved

  # Error Case - Duplicate Entry
  Scenario: User tries to add a duplicate dictionary entry
    Given I am on the Dictionary page
    And I already have an entry with trigger "brb"
    When I try to add another entry with trigger "brb"
    Then I see an error message that this trigger already exists
    And I am offered the option to update the existing entry
```

### Out of Scope
- Cloud sync of dictionary across devices
- Shared/team dictionaries for organizations
- ML-based auto-learning of new terms from user corrections
- Bulk import/export of dictionary entries

### Assumptions
- Dictionary is stored locally on the user's machine
- Expansion happens post-transcription (after Parakeet processes audio), not during speech recognition
- Matching is text-based on the transcription output, not phonetic/audio-based

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Dictionary page accessible from main navigation
- [ ] Users can add, edit, and delete dictionary entries
- [ ] Expansions applied to transcription output automatically
- [ ] Case-insensitive, whole-word matching works correctly
- [ ] Validation prevents invalid/duplicate entries

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

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

- [x] All specs completed
- [x] Technical guidance finalized
- [x] Code reviewed and approved
- [x] Tests written and passing
- [x] Documentation updated

## Feature Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Spec Integration Matrix

| Spec | Declares Integration With | Verified Connection | Status |
|------|--------------------------|---------------------|--------|
| data-model-update | DictionaryStore, DictionaryExpander, Tauri commands | Yes - Fields used in store.rs:24-29, expander.rs:66-76, dictionary.ts:13-15 | PASS |
| backend-storage-update | DictionaryStore, TranscriptionService, Event Bridge | Yes - Commands at dictionary.rs:68-152, events emitted, hook calls invoke | PASS |
| expander-suffix-support | DictionaryStore, TranscriptionService | Yes - service.rs:316-342 uses ExpansionResult, suffix appended at expander.rs:66-69 | PASS |
| keyboard-simulation | DictionaryExpander, TranscriptionService | Yes - service.rs:362-376 calls KeyboardSimulator when should_press_enter is true | PASS |
| settings-panel-ui | useDictionary hook, DictionaryEntry type | Yes - Dictionary.tsx:106-111, 483-489 passes suffix/autoEnter to mutations | PASS |
| suffix-validation | Settings panel UI, save functionality | Yes - Dictionary.tsx:98-105 validateSuffix, lines 196, 292 disable save on error | PASS |

### BDD Scenario Verification

| Scenario | Specs Involved | End-to-End Tested | Status |
|----------|----------------|-------------------|--------|
| Configure punctuation suffix on an existing entry | settings-panel-ui, backend-storage-update | Yes - Dictionary.test.tsx:466-473 | PASS |
| Configure auto-enter on an existing entry | settings-panel-ui, backend-storage-update | Yes - Dictionary.test.tsx:466-473 | PASS |
| Create new entry with punctuation suffix and auto-enter | settings-panel-ui, backend-storage-update | Yes - Dictionary.test.tsx:458-473 | PASS |
| Expansion applies punctuation suffix | expander-suffix-support | Yes - expander_test.rs:116-124 | PASS |
| Expansion triggers auto-enter | expander-suffix-support, keyboard-simulation | Yes - expander_test.rs:137-146, service.rs:362-376 | PASS |
| Expansion with both suffix and auto-enter | expander-suffix-support, keyboard-simulation | Yes - expander_test.rs:148-160 tests both, service.rs handles full flow | PASS |
| Error - Invalid punctuation suffix (too long) | suffix-validation | Yes - Dictionary.test.tsx:593-652 | PASS |

### Integration Health

**Orphaned Components:**
- None identified

**Mocked Dependencies in Production Paths:**
- None identified - All specs are wired to production code paths

**Integration Test Coverage:**
- 6 of 6 integration points have explicit tests
- Backend: 27 dictionary tests including store, expander, and keyboard modules
- Frontend: 10 Dictionary page tests including settings panel and validation

### Smoke Test Results

N/A - No smoke test configured

### Feature Cohesion

**Strengths:**
- Clean data flow from UI through hook to Tauri commands to backend storage and expander
- Proper separation of concerns: data model, storage, expansion logic, keyboard simulation, UI
- Consistent use of serde defaults ensures backward compatibility with existing dictionary.json files
- Event Bridge properly invalidates caches on dictionary_updated events, keeping UI in sync
- Comprehensive test coverage at both Rust (backend) and TypeScript (frontend) layers
- Graceful error handling in keyboard simulation - failures are logged but don't crash the app
- The ExpansionResult struct cleanly encapsulates both expanded text and auto-enter flag

**Concerns:**
- None identified

### Verdict

**APPROVED_FOR_DONE** - All 6 specs are completed and properly integrated. The feature implements a complete end-to-end flow:

1. **UI Layer**: SettingsPanel component allows configuring suffix and auto-enter per entry with proper validation (max 5 chars)
2. **Hook Layer**: useDictionary passes suffix/autoEnter fields to Tauri commands
3. **Command Layer**: add/update commands accept and persist new fields
4. **Storage Layer**: DictionaryEntry struct with serde defaults ensures backward compatibility
5. **Expander Layer**: ExpansionResult returns expanded text with suffix appended and should_press_enter flag
6. **Keyboard Layer**: KeyboardSimulator simulates Enter keypress after paste when auto-enter is enabled

All 7 BDD scenarios from the feature description are covered by tests. The integration is verified through the complete data flow from UI action to backend persistence to expansion behavior.

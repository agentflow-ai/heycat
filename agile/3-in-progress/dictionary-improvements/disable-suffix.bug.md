---
status: pending
severity: major
origin: manual
created: 2025-12-22
completed: null
parent_feature: "dictionary-improvements"
parent_spec: null
---

# Bug: Cannot disable suffix - default transcription suffix still applied to dictionary expansions

**Created:** 2025-12-22
**Severity:** Major

## Problem Description

When a dictionary entry matches during transcription, the expansion still receives the default transcription suffix (punctuation like `.`, `?`, `!` based on voice/context). There's no way to disable the suffix entirely for a dictionary entry.

**Expected:** User should be able to configure a dictionary entry to have NO suffix, overriding the default transcription behavior.

**Actual:** Even with an empty suffix field, the transcription service still appends its default punctuation to dictionary expansions.

## Steps to Reproduce

1. Create a dictionary entry (e.g., trigger: "brb", expansion: "be right back")
2. Leave the suffix field empty (or don't configure one)
3. Use voice transcription that triggers the expansion
4. Observe that the expansion gets the default transcription punctuation (e.g., "be right back.")

## Root Cause

[To investigate: The expander likely appends suffix when present, but the transcription service still applies its own suffix logic to the final output]

## Fix Approach

[To determine: May need a "disable suffix" flag that explicitly tells the transcription service to skip default punctuation for this expansion]

## Acceptance Criteria

- [ ] User can configure a dictionary entry to have no suffix applied
- [ ] When "disable suffix" is set, default transcription punctuation is NOT appended
- [ ] Existing entries with explicit suffix still work correctly
- [ ] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Entry with "disable suffix" flag | No punctuation appended after expansion | [ ] |
| Entry with explicit suffix | Suffix appended, no default punctuation | [ ] |
| Entry with no suffix config (legacy) | Existing behavior preserved | [ ] |

## Integration Points

- Dictionary data model (may need new field)
- Expander logic (suffix handling)
- Transcription service (final output assembly)
- Settings panel UI (new toggle?)

## Integration Test

Manual: Create entry with "disable suffix", trigger via voice, verify no punctuation appears

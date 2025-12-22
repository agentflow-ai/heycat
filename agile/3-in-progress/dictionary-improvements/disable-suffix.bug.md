---
status: completed
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

**Original issue:** The expander appends suffix when present, but the transcription service also applies punctuation based on voice/context. When a user wants no punctuation, they need an explicit flag to suppress all trailing punctuation.

**Follow-up issue (discovered 2025-12-22):** The `disable_suffix` feature was implemented but didn't work because the Rust `DictionaryEntry` struct used snake_case serde serialization (`disable_suffix`) while the frontend expected camelCase (`disableSuffix`). This caused the frontend to always read `undefined` for the `disableSuffix` field when loading entries, making the toggle appear OFF even when the value was stored as `true` in the backend.

## Fix Approach

**Original fix:** Added a `disable_suffix` boolean field to `DictionaryEntry` that, when true:
1. Suppresses any explicit suffix from being appended
2. Strips trailing punctuation after the trigger match in the transcription text

**Follow-up fix:** Added `#[serde(rename_all = "camelCase")]` to the `DictionaryEntry` struct to serialize/deserialize using camelCase, matching frontend expectations. Added `alias` attributes for backward compatibility with existing JSON files that used snake_case.

## Acceptance Criteria

- [x] User can configure a dictionary entry to have no suffix applied
- [x] When "disable suffix" is set, default transcription punctuation is NOT appended
- [x] Existing entries with explicit suffix still work correctly
- [x] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Entry with "disable suffix" flag | No punctuation appended after expansion | [x] |
| Entry with explicit suffix | Suffix appended, no default punctuation | [x] |
| Entry with no suffix config (legacy) | Existing behavior preserved | [x] |

## Integration Points

- Dictionary data model (may need new field)
- Expander logic (suffix handling)
- Transcription service (final output assembly)
- Settings panel UI (new toggle?)

## Integration Test

Manual: Create entry with "disable suffix", trigger via voice, verify no punctuation appears

---

## Review

**Reviewer:** Claude Opus 4.5
**Date:** 2025-12-22
**Verdict:** APPROVED

### Pre-Review Gates

#### 1. Build Warning Check
```
No new warnings related to this fix. Existing warnings are for the unfinished noise-suppression feature.
```
PASS

#### 2. Command Registration Check
```
Unregistered: check_parakeet_model_status, download_model (unrelated to this fix - noise-suppression feature)
```
PASS (no new commands in this fix)

#### 3. Event Subscription Check
No new events introduced by this fix.
PASS

### Manual Review

#### 1. Is the code wired up end-to-end?
- [x] The `#[serde(rename_all = "camelCase")]` attribute on `DictionaryEntry` is used when serializing responses from Tauri commands
- [x] `list_dictionary_entries` returns `Vec<DictionaryEntry>` which now serializes with camelCase
- [x] Frontend `DictionaryEntry` interface expects `disableSuffix` (camelCase) - matches
- [x] Frontend sends `disable_suffix` (snake_case) in invoke parameters - backend accepts this via Tauri command parameters

PASS

#### 2. What would break if this code was deleted?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `#[serde(rename_all = "camelCase")]` | attribute | DictionaryEntry serialization via list/add commands | YES |
| `alias = "disable_suffix"` | attribute | Backward compat for existing JSON files | YES |
| `alias = "auto_enter"` | attribute | Backward compat for existing JSON files | YES |

PASS - All changes are production-reachable

#### 3. Where does the data flow?

```
[UI] Dictionary.tsx toggles disableSuffix
     |
     v
[Hook] useDictionary.ts:34 invoke("add_dictionary_entry", {..., disable_suffix})
     |
     v
[Command] commands/dictionary.rs:70 add_dictionary_entry(disable_suffix: Option<bool>)
     |
     v
[Store] dictionary/store.rs:166 DictionaryEntry { disable_suffix: true }
     |
     v (on list)
[Command] commands/dictionary.rs:48 list_dictionary_entries() -> Vec<DictionaryEntry>
     |
     v (serialization with rename_all = "camelCase")
[JSON] {"disableSuffix": true, ...}
     |
     v
[Frontend] DictionaryEntry.disableSuffix = true (UI reads correct value)
```

PASS - Complete data flow verified

#### 4. Are there any deferrals?
```bash
grep -rn "TODO\|FIXME\|XXX\|HACK\|handled separately\|will be implemented\|for now" src-tauri/src/dictionary/
```
No matches found.

PASS

#### 5. Test Coverage

- `test_expand_disable_suffix_case_insensitive` - Tests the specific bug scenario (Clear? -> /clear)
- `test_expand_with_disable_suffix_strips_trailing_punctuation` - Covers all punctuation types
- `test_backward_compatible_deserialization` - Ensures old JSON files still load

All 28 dictionary tests pass.

PASS

### Summary

The fix correctly addresses the serde serialization mismatch:
1. `#[serde(rename_all = "camelCase")]` ensures JSON responses use camelCase field names
2. `alias` attributes maintain backward compatibility with existing snake_case JSON files
3. The expander logic was already correct - the issue was purely serialization
4. Tests comprehensively cover the disable_suffix functionality including case-insensitive matching

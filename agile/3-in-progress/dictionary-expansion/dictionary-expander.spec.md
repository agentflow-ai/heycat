---
status: pending
created: 2025-12-21
completed: null
dependencies: ["dictionary-store"]
---

# Spec: Dictionary Expander (Backend)

## Description

Create the `DictionaryExpander` that applies dictionary expansions to transcription text. Implements case-insensitive, whole-word matching using regex to replace trigger words with their expansions.

See: `## Transcription + Expansion Pipeline Detail` in technical-guidance.md for the expansion logic diagram.

## Acceptance Criteria

- [ ] `DictionaryExpander` struct that takes a list of `DictionaryEntry`
- [ ] `expand(text: &str) -> String` method applies all expansions
- [ ] Case-insensitive matching (e.g., "BRB", "brb", "Brb" all match "brb" trigger)
- [ ] Whole-word matching only (e.g., "api" doesn't match inside "capitalize")
- [ ] Multiple expansions applied in single pass
- [ ] Original text returned unchanged if no matches

## Test Cases

- [ ] Expansion matches case-insensitively with whole-word boundaries ("brb"/"BRB"/"Brb" all expand, "api" not matched in "capitalize")
- [ ] Multiple entries expand in single pass ("brb" and "api" both replaced)
- [ ] Punctuation-adjacent triggers expand correctly ("brb." and "brb,")
- [ ] No triggers in text: original returned unchanged

## Dependencies

- dictionary-store.spec.md (provides `DictionaryEntry` struct)

## Preconditions

- `regex` crate available in Cargo.toml

## Implementation Notes

**Files to create:**
- `src-tauri/src/dictionary/expander.rs` - DictionaryExpander implementation

**Regex pattern for whole-word, case-insensitive:**
```rust
// For trigger "brb":
let pattern = format!(r"(?i)\b{}\b", regex::escape(trigger));
```

**Algorithm:**
1. For each entry, compile case-insensitive word-boundary regex
2. Apply all replacements to input text
3. Return expanded text

**Performance note:** Pre-compile regex patterns if expander is reused.

## Related Specs

- dictionary-store.spec.md (depends on entry struct)
- pipeline-integration.spec.md (calls this expander)

## Integration Points

- Production call site: `src-tauri/src/transcription/service.rs` (in process_recording)
- Connects to: DictionaryStore (to get entries)

## Integration Test

- Test location: `src-tauri/src/dictionary/expander.rs` (unit tests)
- Verification: [ ] Unit tests pass

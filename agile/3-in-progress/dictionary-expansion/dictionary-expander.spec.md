---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: ["dictionary-store"]
review_round: 1
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

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `DictionaryExpander` struct that takes a list of `DictionaryEntry` | PASS | `src-tauri/src/dictionary/expander.rs:15-17` - struct with `patterns: Vec<CompiledPattern>`, constructor at line 22 takes `&[DictionaryEntry]` |
| `expand(text: &str) -> String` method applies all expansions | PASS | `src-tauri/src/dictionary/expander.rs:50-58` - iterates through patterns and applies replacements |
| Case-insensitive matching | PASS | `src-tauri/src/dictionary/expander.rs:27` - uses `(?i)` flag in regex pattern |
| Whole-word matching only | PASS | `src-tauri/src/dictionary/expander.rs:27` - uses `\b` word boundaries in regex pattern |
| Multiple expansions applied in single pass | PASS | `src-tauri/src/dictionary/expander.rs:53-55` - iterates through all patterns applying each |
| Original text returned unchanged if no matches | PASS | `src-tauri/src/dictionary/expander.rs:51-57` - regex `replace_all` returns original when no matches |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Case-insensitive whole-word matching ("brb"/"BRB"/"Brb" all expand, "api" not matched in "capitalize") | PASS | `src-tauri/src/dictionary/expander_test.rs:12-30` |
| Multiple entries expand in single pass ("brb" and "api" both replaced) | PASS | `src-tauri/src/dictionary/expander_test.rs:32-45` |
| Punctuation-adjacent triggers expand correctly ("brb." and "brb,") | PASS | `src-tauri/src/dictionary/expander_test.rs:47-57` |
| No triggers in text: original returned unchanged | PASS | `src-tauri/src/dictionary/expander_test.rs:59-70` |

### Code Quality

**Strengths:**
- Pre-compiles regex patterns in constructor for efficient reuse (performance note addressed)
- Uses `regex::escape()` to safely handle special characters in triggers
- Graceful error handling with logging when regex compilation fails
- Tests are behavior-focused and cover all acceptance criteria
- Clean separation between compilation and expansion phases

**Concerns:**
- Dead code warnings (`struct DictionaryExpander is never constructed`) - ACCEPTABLE for this spec. This is a foundational internal module as documented in `mod.rs` comment. The `#[allow(unused_imports)]` is correctly applied. The `pipeline-integration.spec.md` is the dependent spec that will wire this into production.

### Pre-Review Gate Results

```
Build warnings (new code):
warning: struct `CompiledPattern` is never constructed
warning: struct `DictionaryExpander` is never constructed
warning: associated items `new` and `expand` are never used
```

These warnings are EXPECTED and ACCEPTABLE because:
1. The spec explicitly lists `pipeline-integration.spec.md` as a dependent spec that "calls this expander"
2. The `mod.rs` file documents: "This is a foundational internal module consumed by tauri-commands.spec.md"
3. The dependency chain in the issue shows this is an internal module with consumers coming in subsequent specs

### Integration Point Verification

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| DictionaryExpander | struct | To be wired by `pipeline-integration.spec.md` | DEFERRED - by design |
| DictionaryExpander::new | fn | To be wired by `pipeline-integration.spec.md` | DEFERRED - by design |
| DictionaryExpander::expand | fn | To be wired by `pipeline-integration.spec.md` | DEFERRED - by design |

The spec explicitly documents that production wiring happens in `pipeline-integration.spec.md` (line 67: "Integration Points - Production call site: `src-tauri/src/transcription/service.rs`"). This is an internal foundational module pattern.

### Verdict

**APPROVED** - The DictionaryExpander implementation correctly satisfies all acceptance criteria and test cases. The dead code warnings are expected for this foundational module spec - production wiring is explicitly deferred to `pipeline-integration.spec.md` as documented in the spec's "Integration Points" and "Related Specs" sections. Tests are behavior-focused and comprehensive per the testing philosophy. Code quality is high with proper regex escaping and error handling.

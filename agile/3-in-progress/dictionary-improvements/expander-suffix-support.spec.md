---
status: pending
created: 2025-12-22
completed: null
dependencies: ["backend-storage-update"]
---

# Spec: Update DictionaryExpander to append suffix when expanding

## Description

Modify DictionaryExpander to:
1. Return an `ExpansionResult` struct instead of a plain `String`
2. Append the entry's suffix to the expansion text when present
3. Track whether any expanded entry has `auto_enter: true` for the keyboard simulation spec

## Acceptance Criteria

- [ ] `expand()` returns `ExpansionResult` struct with `expanded_text` and `should_press_enter` fields
- [ ] When entry has suffix, expansion includes suffix (e.g., "brb" with suffix "." → "be right back.")
- [ ] When entry has no suffix, expansion is unchanged
- [ ] When any expanded entry has `auto_enter: true`, result has `should_press_enter: true`
- [ ] Multiple expansions in same text all apply their suffixes correctly

## Test Cases

- [ ] Expand "brb" with suffix "." → "be right back."
- [ ] Expand "brb" without suffix → "be right back"
- [ ] Expand "brb" with auto_enter=true → should_press_enter is true
- [ ] Expand text with multiple triggers, one has auto_enter → should_press_enter is true
- [ ] Expand text with no matches → should_press_enter is false

## Dependencies

- `backend-storage-update` - DictionaryStore must provide entries with suffix/auto_enter fields

## Preconditions

- DictionaryEntry struct has suffix and auto_enter fields
- DictionaryStore provides entries with these fields populated

## Implementation Notes

### Data Flow Position
```
DictionaryStore (provides entries)
       ↓
DictionaryExpander.expand() ← This spec
       ↓
ExpansionResult { expanded_text, should_press_enter }
       ↓
TranscriptionService (uses result)
```

### New ExpansionResult Struct (`src-tauri/src/dictionary/expander.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpansionResult {
    pub expanded_text: String,
    pub should_press_enter: bool,
}
```

### Updated expand() Method

```rust
impl DictionaryExpander {
    pub fn expand(&self, text: &str) -> ExpansionResult {
        let mut result = text.to_string();
        let mut should_press_enter = false;

        for pattern in &self.patterns {
            if pattern.regex.is_match(&result) {
                // Build replacement with suffix
                let replacement = match &pattern.entry.suffix {
                    Some(suffix) => format!("{}{}", pattern.entry.expansion, suffix),
                    None => pattern.entry.expansion.clone(),
                };

                result = pattern.regex.replace_all(&result, replacement.as_str()).to_string();

                // Track auto_enter
                if pattern.entry.auto_enter {
                    should_press_enter = true;
                }
            }
        }

        ExpansionResult {
            expanded_text: result,
            should_press_enter,
        }
    }
}
```

### Pattern Struct Update

The internal `CompiledPattern` struct needs to store the full entry:

```rust
struct CompiledPattern {
    regex: Regex,
    entry: DictionaryEntry,  // Changed from just storing expansion string
}
```

### Testing Strategy

**Backend (Rust):**
```rust
// src-tauri/src/dictionary/expander_test.rs
#[test]
fn test_expand_with_suffix() {
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: Some(".".to_string()),
        auto_enter: false,
    }];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("I'll brb");

    assert_eq!(result.expanded_text, "I'll be right back.");
    assert_eq!(result.should_press_enter, false);
}

#[test]
fn test_expand_without_suffix() {
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: None,
        auto_enter: false,
    }];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("brb");

    assert_eq!(result.expanded_text, "be right back");
    assert_eq!(result.should_press_enter, false);
}

#[test]
fn test_expand_with_auto_enter() {
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "sig".to_string(),
        expansion: "Best regards, Michael".to_string(),
        suffix: None,
        auto_enter: true,
    }];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("sig");

    assert_eq!(result.expanded_text, "Best regards, Michael");
    assert_eq!(result.should_press_enter, true);
}

#[test]
fn test_expand_multiple_entries_one_auto_enter() {
    let entries = vec![
        DictionaryEntry {
            id: "1".to_string(),
            trigger: "brb".to_string(),
            expansion: "be right back".to_string(),
            suffix: None,
            auto_enter: false,
        },
        DictionaryEntry {
            id: "2".to_string(),
            trigger: "sig".to_string(),
            expansion: "Best regards".to_string(),
            suffix: None,
            auto_enter: true,
        },
    ];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("brb sig");

    assert_eq!(result.expanded_text, "be right back Best regards");
    assert_eq!(result.should_press_enter, true);  // sig has auto_enter
}

#[test]
fn test_expand_no_match_returns_false() {
    let entries = vec![DictionaryEntry {
        id: "1".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: None,
        auto_enter: true,
    }];

    let expander = DictionaryExpander::new(entries);
    let result = expander.expand("hello world");

    assert_eq!(result.expanded_text, "hello world");
    assert_eq!(result.should_press_enter, false);  // No match, no auto_enter
}
```

## Related Specs

- [data-model-update.spec.md](./data-model-update.spec.md) - Provides DictionaryEntry with new fields
- [backend-storage-update.spec.md](./backend-storage-update.spec.md) - DictionaryStore provides entries
- [keyboard-simulation.spec.md](./keyboard-simulation.spec.md) - Uses should_press_enter result

## Integration Points

- Production call site: `src-tauri/src/transcription/service.rs` - RecordingTranscriptionService calls expand()
- Connects to: DictionaryStore (source of entries), TranscriptionService (consumer)

## Integration Test

- Test location: `src-tauri/src/dictionary/expander_test.rs`
- Verification: [ ] Integration test passes

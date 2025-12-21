---
status: pending
created: 2025-12-21
completed: null
dependencies: ["dictionary-expander"]
---

# Spec: Transcription Pipeline Integration (Backend)

## Description

Integrate the `DictionaryExpander` into the `RecordingTranscriptionService` pipeline. Dictionary expansion is applied after Parakeet transcription and before command matching, ensuring expanded text is used for both commands and clipboard.

See: `## Transcription + Expansion Pipeline Detail` in technical-guidance.md.

## Acceptance Criteria

- [ ] `RecordingTranscriptionService` accepts optional `DictionaryExpander` via builder
- [ ] Expansion applied after transcription result, before command matching
- [ ] Expanded text used for command matching (not original)
- [ ] Expanded text copied to clipboard (not original)
- [ ] `transcription_completed` event contains expanded text
- [ ] Graceful fallback: no expander = no expansion (original text used)

## Test Cases

- [ ] Full pipeline with dictionary: transcribed text expanded in clipboard and `transcription_completed` event
- [ ] Expanded text passed to command matcher (not original)

## Dependencies

- dictionary-expander.spec.md (provides DictionaryExpander)
- dictionary-store.spec.md (provides entries to expander)

## Preconditions

- DictionaryExpander implemented and tested
- DictionaryStore can load entries

## Implementation Notes

**Files to modify:**
- `src-tauri/src/transcription/service.rs` - Add expander integration

**Integration point in process_recording (around line 276-301):**
```rust
// After: let text = ... (transcription result)
// Before: let command_handled = Self::try_command_matching(...)

// Apply dictionary expansion
let expanded_text = if let Some(expander) = &self.dictionary_expander {
    expander.expand(&text)
} else {
    text.clone()
};

// Use expanded_text for command matching and clipboard
```

**Builder pattern addition:**
```rust
pub fn with_dictionary_expander(mut self, expander: Arc<DictionaryExpander>) -> Self {
    self.dictionary_expander = Some(expander);
    self
}
```

## Related Specs

- dictionary-expander.spec.md (provides the expander)
- dictionary-store.spec.md (source of entries)

## Integration Points

- Production call site: `src-tauri/src/transcription/service.rs:276-301`
- Connects to: DictionaryExpander, DictionaryStore, command matching, clipboard

## Integration Test

- Test location: Manual testing with dictionary entries
- Verification: [ ] Transcribed text with trigger words shows expansions in clipboard

---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - command-registry
---

# Spec: Fuzzy Matcher

## Description

Match transcribed text against registered command triggers using exact and fuzzy matching. Handles input normalization, parameterized commands, and ambiguous match detection.

## Acceptance Criteria

- [ ] Normalize input: lowercase, trim whitespace
- [ ] Exact match takes priority over fuzzy match
- [ ] Fuzzy matching using Levenshtein distance with configurable threshold
- [ ] Extract parameters from parameterized commands (e.g., "type {text}")
- [ ] Return `MatchResult`: Exact, Fuzzy(score), Ambiguous(candidates), NoMatch
- [ ] Ambiguous detection when multiple commands match within threshold

## Test Cases

- [ ] Exact match "open slack" returns Exact result
- [ ] Fuzzy match "opn slack" returns Fuzzy result with high score
- [ ] "OPEN SLACK" normalizes to match "open slack"
- [ ] "type hello world" extracts "hello world" as parameter
- [ ] Very different text "xyz abc" returns NoMatch
- [ ] Two similar commands trigger Ambiguous result

## Dependencies

- command-registry (provides command list to match against)

## Preconditions

- Command registry initialized with at least one command

## Implementation Notes

- Location: `src-tauri/src/voice_commands/matcher.rs`
- Use `strsim` crate for Levenshtein distance
- Default threshold: 0.8 similarity score
- Parameterized pattern: `{param_name}` captures remaining text

## Related Specs

- command-registry.spec.md (data source)
- transcription-integration.spec.md (caller)
- disambiguation-ui.spec.md (handles Ambiguous results)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs` (post-transcription)
- Connects to: command-registry, transcription-integration

## Integration Test

- Test location: `src-tauri/src/voice_commands/matcher_test.rs`
- Verification: [ ] Integration test passes

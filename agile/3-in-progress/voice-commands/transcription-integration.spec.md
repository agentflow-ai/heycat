---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - command-registry
  - fuzzy-matcher
  - action-executor
---

# Spec: Transcription Integration

## Description

Wire command matching into the existing transcription pipeline. Intercept transcribed text, attempt command matching, execute if matched, or fall back to clipboard copy.

## Acceptance Criteria

- [ ] Intercept transcription at `hotkey/integration.rs` after text received
- [ ] Call fuzzy-matcher with transcribed text
- [ ] On match: execute command via action-executor
- [ ] On no match: copy to clipboard (existing behavior)
- [ ] Emit `command_matched` event when command identified
- [ ] Emit `command_executed` or `command_failed` after execution
- [ ] Emit `transcription_completed` for clipboard fallback

## Test Cases

- [ ] Transcription "open slack" matches and launches Slack
- [ ] Transcription "random text" copies to clipboard
- [ ] Matched command emits command_matched event
- [ ] Successful execution emits command_executed event
- [ ] Failed execution emits command_failed event
- [ ] Fallback path emits transcription_completed event

## Dependencies

- command-registry (command definitions)
- fuzzy-matcher (matching logic)
- action-executor (command execution)

## Preconditions

- ai-transcription feature complete
- Voice commands module initialized

## Implementation Notes

- Location: Modify `src-tauri/src/hotkey/integration.rs:249-264`
- Add voice_commands module to Tauri state
- Flow: transcription -> try_match -> execute or clipboard

## Related Specs

- command-registry.spec.md
- fuzzy-matcher.spec.md
- action-executor.spec.md
- disambiguation-ui.spec.md (handles ambiguous matches)

## Integration Points

- Production call site: `src-tauri/src/hotkey/integration.rs:spawn_transcription()`
- Connects to: all voice_commands modules, existing transcription flow

## Integration Test

- Test location: `src-tauri/src/hotkey/integration_test.rs`
- Verification: [ ] Integration test passes

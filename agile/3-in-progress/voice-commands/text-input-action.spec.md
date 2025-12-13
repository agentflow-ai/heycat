---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - action-executor
---

# Spec: Text Input Action

## Description

Type text into the currently focused application using macOS keyboard simulation. Implements the Action trait for TypeText action type using CGEvent APIs.

## Acceptance Criteria

- [ ] Implement `Action` trait for `TextInputAction`
- [ ] Type text using CGEvent keyboard simulation
- [ ] Check Accessibility permission before execution
- [ ] Return permission error if Accessibility not granted
- [ ] Handle special characters and unicode
- [ ] Support configurable typing delay between characters

## Test Cases

- [ ] Type "hello world" into focused text field
- [ ] Type text with special characters (!@#$%)
- [ ] Type unicode characters (emojis, accented letters)
- [ ] Missing Accessibility permission returns PermissionDenied error
- [ ] Empty text parameter returns success (no-op)

## Dependencies

- action-executor (provides Action trait and dispatch)

## Preconditions

- macOS environment
- Accessibility permission granted
- Text input field focused

## Implementation Notes

- Location: `src-tauri/src/voice_commands/actions/text_input.rs`
- Use `core-foundation` and `core-graphics` crates
- CGEvent keyboard events: `CGEvent::new_keyboard_event`
- Check permission via `AXIsProcessTrusted()`
- Reference: VoiceInk `CursorPaster.swift` for CGEvent pattern

## Related Specs

- action-executor.spec.md (trait definition)
- transcription-integration.spec.md (end-to-end flow)

## Integration Points

- Production call site: `src-tauri/src/voice_commands/executor.rs` (dispatch)
- Connects to: action-executor, macOS Accessibility APIs

## Integration Test

- Test location: `src-tauri/src/voice_commands/actions/text_input_test.rs`
- Verification: [ ] Integration test passes

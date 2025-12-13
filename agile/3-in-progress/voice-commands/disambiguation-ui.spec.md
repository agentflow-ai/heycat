---
status: pending
created: 2025-12-13
completed: null
dependencies:
  - fuzzy-matcher
  - transcription-integration
---

# Spec: Disambiguation UI

## Description

Frontend component to handle ambiguous command matches. Displays candidate commands for user selection when multiple commands match with similar confidence.

## Acceptance Criteria

- [ ] React component receives ambiguous match candidates via event
- [ ] Display list of matching commands with trigger phrases
- [ ] User can select intended command via click or keyboard
- [ ] Selection triggers command execution
- [ ] Cancel/timeout returns to idle state without execution
- [ ] Component auto-dismisses after selection or timeout

## Test Cases

- [ ] Display 2 candidate commands when ambiguous event received
- [ ] Click on candidate triggers execution
- [ ] Keyboard navigation (up/down/enter) works
- [ ] Escape key dismisses without execution
- [ ] 5-second timeout auto-dismisses
- [ ] Component not visible when no ambiguous match

## Dependencies

- fuzzy-matcher (produces Ambiguous result)
- transcription-integration (emits ambiguous event)

## Preconditions

- Frontend React app running
- Event listener for command_ambiguous event

## Implementation Notes

- Location: `src/components/DisambiguationPanel.tsx`
- Listen for `command_ambiguous` Tauri event
- Call `execute_command` Tauri command on selection
- Use existing UI patterns from the app

## Related Specs

- fuzzy-matcher.spec.md (produces ambiguous result)
- transcription-integration.spec.md (emits event)

## Integration Points

- Production call site: `src/App.tsx` (event listener, component mount)
- Connects to: Tauri event system, action-executor via command

## Integration Test

- Test location: `src/components/DisambiguationPanel.test.tsx`
- Verification: [ ] Integration test passes

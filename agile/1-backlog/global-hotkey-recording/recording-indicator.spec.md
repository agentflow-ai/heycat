---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - recording-state-hook
---

# Spec: Recording Indicator Component

## Description

Implement a React component that displays recording status with visual feedback. Shows idle/recording states with appropriate styling and animations.

## Acceptance Criteria

- [ ] Display "Idle" state (gray indicator, no animation)
- [ ] Display "Recording" state (red indicator, pulsing animation)
- [ ] Display error message if recording fails
- [ ] Accessible (ARIA labels for screen readers)
- [ ] Dark mode support via CSS media query

## Test Cases

- [ ] Renders idle state correctly when `isRecording: false`
- [ ] Renders recording state with red indicator when `isRecording: true`
- [ ] Shows error message when error prop is set
- [ ] Animation keyframes applied during recording
- [ ] ARIA live region announces state changes
- [ ] Dark mode colors apply correctly

## Dependencies

- [recording-state-hook.spec.md](recording-state-hook.spec.md) - Provides recording state

## Preconditions

- useRecording hook implemented
- CSS animation support in target browsers

## Implementation Notes

- Create new files: `src/components/RecordingIndicator.tsx`, `src/components/RecordingIndicator.css`
- Use `useRecording` hook for state
- CSS keyframes for pulse: `@keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }`
- ARIA: `role="status"` and `aria-live="polite"`

## Related Specs

- [recording-state-hook.spec.md](recording-state-hook.spec.md) - State provider
- [app-integration.spec.md](app-integration.spec.md) - Integration into app

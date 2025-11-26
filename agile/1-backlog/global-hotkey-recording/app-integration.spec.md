---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - recording-indicator
  - hotkey-integration
---

# Spec: App Integration

## Description

Integrate the RecordingIndicator component into the main App.tsx. Ensures the indicator is visible, positioned appropriately, and syncs with global hotkey triggers.

## Acceptance Criteria

- [ ] RecordingIndicator component added to App.tsx
- [ ] Component visible in UI with appropriate placement (e.g., header/corner)
- [ ] Hotkey (Cmd+Shift+R) toggles indicator state via backend events
- [ ] State persists correctly across component re-renders
- [ ] No console errors or warnings

## Test Cases

- [ ] Component renders in app without errors
- [ ] State syncs when backend emits recording events
- [ ] Multiple rapid state changes handled correctly
- [ ] Component remains visible after other UI interactions
- [ ] No memory leaks from event listeners

## Dependencies

- [recording-indicator.spec.md](recording-indicator.spec.md) - Component to integrate
- [hotkey-integration.spec.md](hotkey-integration.spec.md) - Hotkey triggers events

## Preconditions

- All Layer 2 specs completed (backend ready)
- RecordingIndicator component implemented

## Implementation Notes

- Import and add `<RecordingIndicator />` in `src/App.tsx`
- Position using flexbox or absolute positioning
- Consider z-index for overlay scenarios
- Test with `bun run tauri dev` for full integration

## Related Specs

- [recording-indicator.spec.md](recording-indicator.spec.md) - The component
- [hotkey-integration.spec.md](hotkey-integration.spec.md) - Event source

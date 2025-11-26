---
status: pending
created: 2025-11-26
completed: null
dependencies:
  - tauri-ipc-commands
  - event-emission
---

# Spec: Recording State Hook

## Description

Implement a custom React hook `useRecording` that manages recording state on the frontend. Provides methods to start/stop recording and listens to backend events for state updates.

## Acceptance Criteria

- [ ] Hook returns: `{ isRecording, error, startRecording, stopRecording, lastRecording }`
- [ ] `startRecording()` calls `invoke("start_recording")`
- [ ] `stopRecording()` calls `invoke("stop_recording")`
- [ ] Listen to `recording_started` and `recording_stopped` events
- [ ] Clean up event listeners on unmount

## Test Cases

- [ ] Hook initializes with `isRecording: false` and no error
- [ ] `startRecording()` updates `isRecording` to true on success
- [ ] `stopRecording()` updates `isRecording` to false and sets `lastRecording`
- [ ] Event listener updates state when backend emits events
- [ ] Cleanup removes listeners on component unmount
- [ ] Error state set when invoke fails

## Dependencies

- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Backend commands
- [event-emission.spec.md](event-emission.spec.md) - Backend events

## Preconditions

- IPC commands and events implemented in backend
- `@tauri-apps/api` available in frontend

## Implementation Notes

- Create new file: `src/hooks/useRecording.ts`
- Use `useEffect` for event listener setup/cleanup
- Import `invoke` from `@tauri-apps/api/core`
- Import `listen` from `@tauri-apps/api/event`
- Mark Tauri calls with `/* v8 ignore next */` for coverage

## Related Specs

- [recording-indicator.spec.md](recording-indicator.spec.md) - Consumes this hook
- [tauri-ipc-commands.spec.md](tauri-ipc-commands.spec.md) - Backend commands

# Feature: Global Hotkey Microphone Recording

**Created:** 2025-11-26
**Owner:** Michael

## Description

Implement microphone recording functionality that allows users to capture audio via a global hotkey. The recording should work system-wide (even when the app is not focused) and provide both file output and in-memory storage for further processing (e.g., transcription).

### Requirements

- **Audio Format:** WAV (uncompressed) for maximum quality and transcription accuracy
- **Global Hotkey:** Cmd+Shift+R on macOS, Ctrl+Shift+R on Windows/Linux
- **Trigger Mode:** Toggle - press once to start recording, press again to stop
- **User Feedback:** Simple UI element in the frontend showing recording state
- **Output:**
  - Save recordings to file (WAV format)
  - Keep audio data in memory for immediate processing (transcription pipeline)

### Technical Context

This is a Tauri v2 application. Implementation will require:
- **Backend (Rust):** Audio capture using platform APIs (cpal/rodio), global hotkey registration (tauri-plugin-global-shortcut)
- **Frontend (React):** Recording indicator component, state management for recording status
- **IPC:** Commands for start/stop recording, events for state changes

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Global hotkey (Cmd/Ctrl+Shift+R) triggers recording toggle from anywhere in the system
- [ ] Audio is captured from the default microphone in WAV format
- [ ] Recording state is visible in the UI (recording indicator)
- [ ] Completed recordings are saved to disk
- [ ] Audio data is available in memory for transcription processing
- [ ] Recording works when app window is not focused

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

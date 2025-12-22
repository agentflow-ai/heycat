---
status: pending
severity: minor
origin: manual
created: 2025-12-22
completed: null
parent_feature: null
parent_spec: null
---

# Bug: Multiple Paste On Close

**Created:** 2025-12-22
**Owner:** Claude
**Severity:** Minor

## Problem Description

When closing the app after using it (recording/transcribing), dozens of paste operations fire, pasting transcription text or clipboard content to whatever application has focus.

## Steps to Reproduce

1. Open the app and record/transcribe something
2. Wait for transcription to complete (app is idle)
3. Close the app
4. Observe dozens of paste operations in the focused application

## Root Cause

Async transcription tasks spawned via `tauri::async_runtime::spawn()` continue running after window destruction. These tasks call `simulate_paste()` which creates Cmd+V keystrokes via CoreGraphics. No coordination exists to stop paste operations during shutdown.

**Paste locations:**
- `src-tauri/src/transcription/service.rs:356` - in `process_recording()` async task
- `src-tauri/src/hotkey/integration.rs:277` - in `copy_and_paste()`

## Fix Approach

Add a global shutdown flag (`AtomicBool`) that prevents paste during shutdown:
1. Create `shutdown.rs` module with `signal_shutdown()` and `is_shutting_down()`
2. Signal shutdown on `WindowEvent::Destroyed` before any cleanup
3. Guard all `simulate_paste()` calls to check the flag

## Acceptance Criteria

- [ ] Bug no longer reproducible
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Close app after transcription | No paste operations fire | [ ] |
| Normal transcription while app open | Paste works normally | [ ] |

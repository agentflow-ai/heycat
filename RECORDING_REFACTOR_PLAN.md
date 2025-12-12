# Microphone Recording Feature - Refactor Plan

This document outlines the 7-phase refactoring plan implemented to address code quality issues identified during code review of the microphone recording feature.

## Overview

The refactoring addressed:
- Sample rate mismatch between device and encoding
- Commands not triggering audio capture (only hotkey did)
- Audio thread never being properly shutdown
- Excessive use of `.expect()` causing potential panics
- Code duplication between hotkey and command paths
- No buffer size limits (unlimited memory growth)
- Duplicate state sync between frontend and backend

---

## Phase 1: Sample Rate Propagation

**Problem:** Sample rate was hardcoded to 44100 Hz, but audio devices may use different rates (e.g., 48000 Hz), causing audio quality issues.

**Solution:**
- Modified `AudioCaptureBackend::start()` to return `Result<u32, AudioCaptureError>` with actual sample rate
- Added `ActiveRecording` struct to `RecordingManager` to track sample rate per recording
- Added `set_sample_rate()` and `get_sample_rate()` methods
- Updated hotkey and command paths to use actual device sample rate for WAV encoding

**Files Changed:**
- `src-tauri/src/audio/mod.rs`
- `src-tauri/src/audio/cpal_backend.rs`
- `src-tauri/src/audio/thread.rs`
- `src-tauri/src/recording/state.rs`
- `src-tauri/src/hotkey/integration.rs`
- `src-tauri/src/commands/logic.rs`

---

## Phase 2: Commands Trigger Audio Capture

**Problem:** Tauri commands (`start_recording`, `stop_recording`) only changed state but didn't actually capture audio. Only the hotkey path triggered real audio capture.

**Solution:**
- Wrapped `AudioThreadHandle` in `Arc` for sharing between hotkey and commands
- Added `AudioThreadHandle` as Tauri-managed state
- Updated command implementations to call `audio_thread.start()` and `audio_thread.stop()`
- Both paths now have full API parity

**Files Changed:**
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/commands/logic.rs`
- `src-tauri/src/hotkey/integration.rs`

---

## Phase 3: Implement Drop for AudioThreadHandle

**Problem:** When the app closed, the audio thread was orphaned (JoinHandle dropped without joining), potentially causing resource leaks.

**Solution:**
- Changed `_thread: JoinHandle<()>` to `thread: Option<JoinHandle<()>>`
- Implemented `Drop` for `AudioThreadHandle`:
  - Sends `Shutdown` command to audio thread
  - Joins the thread to wait for clean exit

**Files Changed:**
- `src-tauri/src/audio/thread.rs`

---

## Phase 4: Replace .expect() with Proper Error Handling

**Problem:** Excessive use of `.expect()` in production code could cause panics on edge cases like lock poisoning or state machine bugs.

**Solution:**
- Replaced all `.expect()` calls with proper error handling
- Lock poisoning now emits error event + returns gracefully
- State transition failures emit error events
- Buffer errors attempt recovery by transitioning to Idle
- Created coverage-excluded error handler functions for untestable paths

**Files Changed:**
- `src-tauri/src/hotkey/integration.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/recording/state.rs`

---

## Phase 5: Unify Hotkey and Command Paths

**Problem:** Hotkey and command paths had separate implementations for start/stop recording logic, leading to code duplication and potential inconsistencies.

**Solution:**
- Made `commands::logic` module public
- Updated `HotkeyIntegration::handle_toggle()` to call `start_recording_impl()` and `stop_recording_impl()`
- Removed duplicate code (~100 lines): `try_start_audio_capture()`, `encode_samples_to_wav()`, error handlers
- Hotkey now just adds debouncing and event emission on top of command implementations

**Files Changed:**
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/hotkey/integration.rs`

---

## Phase 6: Add Buffer Size Limit and User-Friendly Errors

**Problem:**
1. No buffer size limit - recording for hours could consume gigabytes of memory
2. Error messages were technical and not helpful to users

**Solution:**
- Added `MAX_BUFFER_SAMPLES` constant (~10 minutes at 48kHz, ~115MB)
- Updated cpal callbacks to check buffer size and stop adding samples when full
- Replaced technical error messages with user-friendly ones:
  - "Already recording" → "A recording is already in progress. Stop the current recording first."
  - "Audio capture failed" → "Could not access the microphone. Please check that your microphone is connected and permissions are granted."
  - etc.

**Files Changed:**
- `src-tauri/src/audio/mod.rs`
- `src-tauri/src/audio/cpal_backend.rs`
- `src-tauri/src/commands/logic.rs`

---

## Phase 7: Simplify Frontend State Sync

**Problem:** Frontend updated state both from command responses AND from events, causing duplicate updates and potential race conditions with hotkey-triggered recordings.

**Solution:**
- Frontend now only updates state from events (single source of truth)
- Tauri commands emit events on success (`recording_started`, `recording_stopped`)
- Hotkey and command paths both emit the same events
- Tests updated to simulate events after commands

**Files Changed:**
- `src/hooks/useRecording.ts`
- `src/hooks/useRecording.test.ts`
- `src-tauri/src/commands/mod.rs`
- `src/components/RecordingIndicator.tsx` (coverage exclusion)

---

## Summary

| Phase | Commits | Key Improvement |
|-------|---------|-----------------|
| 1 | 9ee2339 | Correct sample rate throughout pipeline |
| 2 | dc8e818 | Commands actually capture audio |
| 3 | d9bb475 | Clean thread shutdown on app exit |
| 4 | 94cf4d6 | No more panics, graceful error handling |
| 5 | 419d687 | Single implementation for both paths |
| 6 | d7aeca7 | Memory safety + better UX |
| 7 | de702b6 | Consistent event-driven state sync |

All phases maintained test coverage with TCR (Test-Commit-Refactor) workflow.

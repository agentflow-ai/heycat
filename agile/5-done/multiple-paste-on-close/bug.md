---
status: completed
severity: minor
origin: manual
created: 2025-12-22
completed: 2025-12-24
parent_feature: null
parent_spec: null
---

# Bug: Multiple Paste On Close

**Created:** 2025-12-22
**Completed:** 2025-12-24
**Owner:** Claude
**Severity:** Minor

## Problem Description

When closing the app after using it (recording/transcribing), dozens of paste operations fire, pasting transcription text or clipboard content to whatever application has focus.

## Steps to Reproduce

1. Open the app and record/transcribe something
2. Wait for transcription to complete (app is idle)
3. Close the app (Ctrl+C in terminal)
4. Observe paste operations in the terminal

## Root Cause

**Initial hypothesis (WRONG):** Async transcription tasks continuing after window destruction.

**Actual root cause:** Calling `std::process::exit(0)` from a signal handler (ctrlc callback) is NOT async-signal-safe. This causes undefined behavior on macOS - the process doesn't exit cleanly, and macOS's CGEvent system generates spurious keyboard events.

**Diagnostic proof:** Added `[PASTE-TRACE]` logging to all paste paths. During shutdown:
- NO trace logs from Rust paste code appeared
- Paste events occurred AFTER `exit(0)` was called
- Terminal left in broken state (arrow keys show escape sequences)

## Fix Approach

Use Tauri's graceful exit mechanism instead of `std::process::exit()`:
1. Store AppHandle globally via `shutdown::register_app_handle()`
2. Call `shutdown::request_app_exit(0)` which uses `AppHandle::exit(0)` for clean shutdown
3. Keep shutdown guards on `simulate_paste()` as defense-in-depth
4. Centralize keyboard synthesis in `keyboard/synth.rs`

## Definition of Done

- [x] All specs completed
- [x] Technical guidance finalized
- [x] Code reviewed and approved
- [x] Tests written and passing
- [x] Root cause documented

## Acceptance Criteria

- [x] Bug no longer reproducible
- [x] Root cause addressed (not just symptoms)
- [x] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Close app after transcription | No paste operations fire | [x] |
| Normal transcription while app open | Paste works normally | [x] |

## Bug Review

**Reviewed:** 2025-12-24
**Reviewer:** Claude

### Root Cause Verification

| Aspect | Status | Evidence |
|--------|--------|----------|
| Root cause identified in guidance | PASS | `technical-guidance.md` - `std::process::exit(0)` from signal handler is not async-signal-safe, causing undefined behavior |
| Fix addresses root cause (not symptoms) | PASS | Using `AppHandle::exit(0)` via `request_app_exit()` for graceful shutdown instead of `std::process::exit(0)` |
| Related code paths checked | PASS | All paste paths have shutdown guards + centralized synthesis in `keyboard/synth.rs` |

### Regression Test Audit

| Test | Status | Location |
|------|--------|----------|
| Shutdown flag state transitions | PASS | `src-tauri/src/shutdown.rs` - Unit test verifies flag transitions |
| Integration-level paste prevention | PASS | Manual testing confirms no paste on Ctrl+C after recording |

### Bug Fix Cohesion

**Strengths:**
- Root cause properly diagnosed via diagnostic logging
- Graceful exit using Tauri's intended pattern (`AppHandle::exit()`)
- Defense in depth: shutdown guards remain as safety net
- Centralized keyboard synthesis prevents code duplication
- Clean terminal state after exit

**Concerns:**
- None identified.

### Verdict

**APPROVED_FOR_DONE** - Root cause properly identified (`std::process::exit(0)` from signal handler is unsafe) and fixed with graceful Tauri exit. Manual testing confirms the bug is fixed.

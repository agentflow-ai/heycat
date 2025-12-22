---
status: pending
severity: critical
origin: manual
created: 2025-12-22
completed: null
parent_feature: null
parent_spec: null
---

# Bug: Hotkey Audio Speedup

**Created:** 2025-12-22
**Owner:** Claude
**Severity:** Critical

## Problem Description

Hotkey recordings progressively speed up (chipmunk audio) with each subsequent recording. First recording is normal, each subsequent one is faster. Memory also increases.

**Root Cause:** cpal stream callback race condition - when stream is dropped in CpalBackend::stop(), the OS audio thread callback may still be running, causing state corruption between recordings.

**Evidence:**
- First recording works = fresh state
- Progressive degradation = state accumulating
- Memory increase = resources not fully cleaned up
- CpalBackend is reused across recordings (created once in audio_thread_main)

**Fix:** Add explicit barrier after stream drop to ensure callback completes before cleanup, plus early-exit check in callback when signaled to stop.

**Files:** src-tauri/src/audio/cpal_backend.rs (primary), src-tauri/src/audio/thread.rs

## Steps to Reproduce

1. [First step]
2. [Second step]
3. [Expected result vs actual result]

## Root Cause

[To be filled in during investigation]

## Fix Approach

[How the bug will be fixed]

## Acceptance Criteria

- [ ] Bug no longer reproducible
- [ ] Root cause addressed (not just symptoms)
- [ ] Tests added to prevent regression

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| [Test case description] | [Expected outcome] | [ ] |

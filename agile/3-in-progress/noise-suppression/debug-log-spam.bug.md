---
status: completed
severity: critical
origin: manual
created: 2025-12-22
completed: null
parent_feature: noise-suppression
parent_spec: null
---

# Bug: Disable verbose denoiser diagnostics

**Created:** 2025-12-22
**Severity:** Major

## Problem Description

The `tract_core` crate emits thousands of DEBUG-level log messages during ONNX model optimization. This causes severe performance degradation - hotkey response takes several seconds due to logging overhead.

Example logs:
```
[tract_core::optim][DEBUG] applying patch #100: codegen/0 >> codegen #211 "model_2/lstm_6/MatMul_add.split-over-1.256..384" EinSumMatMul >> Einsum to OptMatMul
[tract_core::optim][DEBUG] applying patch #101: codegen/0 >> codegen #242 "conv1d_3" EinSumMatMul >> Einsum to OptMatMul
```

## Steps to Reproduce

1. Start the application with noise suppression enabled
2. Press the hotkey to trigger recording
3. Observe multi-second delay before recording starts
4. Check logs - thousands of `tract_core::optim` DEBUG messages

## Root Cause

The logging configuration does not filter out DEBUG-level logs from the `tract_core` dependency. During model loading/optimization, tract emits extensive debug output that floods the logging system.

## Fix Approach

Configure the logging system to filter `tract_core` logs to WARN or ERROR level only, while preserving debug logging for application code.

## Acceptance Criteria

- [ ] No `tract_core` DEBUG logs appear during normal operation
- [ ] Hotkey responds immediately (no multi-second delay)
- [ ] Application debug logging still works for heycat code
- [ ] Model loading/inference still functions correctly

## Test Cases

| Test Case | Expected Result | Status |
|-----------|-----------------|--------|
| Press hotkey with noise suppression enabled | Immediate response (<500ms) | [ ] |

## Integration Points

Logging configuration (likely in main.rs or lib.rs) - need to add filter for `tract_core` module.

## Integration Test

N/A - manual verification by checking logs and hotkey response time.

## Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| No `tract_core` DEBUG logs appear during normal operation | PASS | `lib.rs:60` - `.level_for("tract_core", tauri_plugin_log::log::LevelFilter::Warn)` filters tract_core to WARN level |
| Hotkey responds immediately (no multi-second delay) | DEFERRED | Requires manual testing - logging filter is correctly configured to eliminate spam |
| Application debug logging still works for heycat code | PASS | `lib.rs:53-57` - Base level is DEBUG for debug builds, Info for release; tract filters are module-specific overrides |
| Model loading/inference still functions correctly | PASS | Log filtering only affects log output, not model execution; tract library functionality unchanged |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Press hotkey with noise suppression enabled | DEFERRED | Manual verification required - no automated test for this |

### Code Quality

**Strengths:**
- Comprehensive filtering: Implementation filters ALL tract-related modules (`tract_core`, `tract_onnx`, `tract_hir`, `tract_linalg`) not just `tract_core`, preventing any related debug spam
- Clear documentation: Comment at lines 58-59 explains why the filter exists ("Suppress verbose DEBUG logs from tract ONNX inference library")
- Minimal impact: Uses `level_for()` module-specific filtering which only affects tract crates while preserving debug logging for application code
- Correct integration point: Filter added in the logging plugin builder during app setup

**Concerns:**
- None identified

### Verdict

**APPROVED** - The logging configuration correctly filters tract library DEBUG output to WARN level while preserving application debug logging. All tract-related modules are filtered comprehensively. The implementation follows the Tauri logging plugin patterns and is properly documented with comments explaining the purpose.

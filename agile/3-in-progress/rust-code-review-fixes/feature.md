# Feature: Rust Code Review Fixes

**Created:** 2025-12-21
**Owner:** Claude
**Discovery Phase:** complete

## Description

Address issues found during senior Rust code review. These are code quality improvements identified during a comprehensive review of the `src-tauri/` codebase focusing on correctness, error handling, and code clarity.

## Review Summary

**Overall Assessment:** Good - Production-ready with minor improvements recommended

The codebase demonstrates solid Rust practices with good architectural separation between Tauri framework code and testable business logic. No critical bugs or security vulnerabilities were found.

## Issues to Address

### High Priority (Code Quality)
1. **thiserror adoption** - Three error types still use manual Display/Error impls
2. **Silent error swallowing** - TranscribingGuard Drop ignores lock failures
3. **Missing error variant** - ActionErrorCode missing UnsupportedPlatform (breaks non-macOS builds)

### Medium Priority (Defensive Coding)
4. **Integer overflow clarity** - AudioFingerprint should use saturating_sub
5. **Magic number** - VAD partial chunk size should be a named constant

### Positive Observations (No Action Needed)
- Well-designed architecture with trait-based abstractions
- Excellent error handling patterns at Tauri boundaries
- Good threading with Arc<Mutex<T>> and lock-free ring buffers
- Security-conscious input validation in app_launcher.rs

## BDD Scenarios

```gherkin
Feature: Rust Code Review Fixes

  Scenario: Error types use thiserror consistently
    Given the codebase has custom error types
    When a developer reads the error definitions
    Then all error types use thiserror derive macro
    And no manual Display/Error implementations exist

  Scenario: Lock failures are observable
    Given the TranscribingGuard manages state transitions
    When a lock is poisoned during drop
    Then a warning is logged for debugging
    And no panic occurs

  Scenario: Code compiles on all platforms
    Given the code uses platform-specific features
    When compiled on non-macOS platforms
    Then all error variants exist
    And compilation succeeds
```

## Acceptance Criteria (High-Level)

- [ ] All error types use `#[derive(thiserror::Error)]`
- [ ] Lock failures in Drop impls are logged
- [ ] Code compiles on non-macOS platforms
- [ ] Magic numbers extracted to constants
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Definition of Done

- [ ] All specs completed
- [ ] Code reviewed and approved
- [ ] Tests written and passing

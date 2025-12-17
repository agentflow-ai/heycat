# Feature: Tauri Code Review Improvements

**Created:** 2025-12-17
**Owner:** Claude
**Discovery Phase:** complete

## Description

Implement improvements identified from comprehensive Tauri code review. These are low-severity enhancements that improve code consistency, build optimization, and type safety without changing functionality.

**Review Summary:**
- Overall Rating: 8.7/10
- Plugin Adoption: 9/10
- Command Patterns: 9/10
- Security: 9/10

## BDD Scenarios

### User Persona
Developer maintaining the heycat Tauri application who wants to ensure best practices are followed consistently throughout the codebase.

### Problem Statement
The code review identified minor improvements that would enhance code quality: missing release build optimizations, an inconsistent event name constant, and a missing TypeScript type parameter. These are small changes with low risk that improve maintainability.

```gherkin
Feature: Tauri Code Review Improvements

  Scenario: Release build optimization
    Given the Cargo.toml has no [profile.release] section
    When a release build is created with `cargo build --release`
    Then the binary should be optimized for size
    And LTO should reduce binary size

  Scenario: Consistent event naming
    Given the audio-level event uses a string literal
    When the developer searches for event names
    Then all event names should be found in events.rs
    And the audio-level constant should match the emission site

  Scenario: Type-safe invoke calls
    Given the stop_recording invoke has no type parameter
    When TypeScript is compiled
    Then the invoke call should have explicit RecordingMetadata type
    And type checking should catch any mismatches
```

### Out of Scope
- Major refactoring (setup function extraction deferred)
- Adding tauri-plugin-window-state (separate feature)
- Changing any runtime behavior

### Assumptions
- Current tests continue to pass
- No functional changes, only code quality improvements
- Changes are backwards compatible

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] Release build optimizations added to Cargo.toml
- [ ] Audio-level event uses constant from events module
- [ ] TypeScript invoke calls have explicit type parameters

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing

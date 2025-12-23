---
discovery_phase: complete
---

# Feature: Worktree Support

**Created:** 2025-12-23
**Owner:** Michael
**Discovery Phase:** not_started

## Description

Add support for running heycat from git worktree directories. Currently there is no worktree support, which blocks developers who use worktrees for parallel feature development.

## BDD Scenarios

### User Persona

A developer who uses git worktrees to work on multiple branches simultaneously. They need to be able to run and develop heycat from any worktree directory, not just the main repository checkout.

### Problem Statement

heycat currently has no worktree support. Developers using git worktrees for parallel feature development cannot effectively work on heycat because the application doesn't recognize or properly function when run from a worktree directory.

```gherkin
Feature: Worktree Support

  Scenario: Happy path - Run dev server from worktree
    Given I am in a git worktree directory
    And the worktree was created with 'git worktree add'
    When I run the development server
    Then heycat starts and functions normally
    And uses worktree-specific configuration

  Scenario: Happy path - Build app from worktree
    Given I am in a git worktree directory
    When I build the application
    Then it compiles successfully
    And the built app runs correctly with worktree isolation

  Scenario: Worktree isolation - Config locations
    Given I am running heycat from a worktree
    When the app reads or writes configuration
    Then it uses a worktree-specific config location
    And does not collide with main repo or other worktrees

  Scenario: Worktree isolation - Installation locations
    Given I am running heycat from a worktree
    When the app installs or accesses local files
    Then it uses worktree-specific installation paths
    And files are isolated from other worktrees

  Scenario: Worktree isolation - Different default hotkey per worktree
    Given I am running heycat from worktree A
    And another instance is running from worktree B
    When I configure the recording hotkey in worktree A
    Then worktree A uses its own hotkey setting
    And worktree B maintains its separate hotkey setting

  Scenario: Automatic worktree detection
    Given I am in a git worktree directory
    When heycat starts
    Then it automatically detects the worktree context
    And applies worktree-specific isolation without manual configuration

  Scenario: Error case - Config collision detected
    Given I am running heycat from a worktree
    When a configuration collision is detected with another worktree
    Then a clear error message is displayed
    And resolution steps are provided to fix the collision
```

### Out of Scope

- Detached/standalone worktrees (not linked to a main repository)
- Cross-machine sync of worktree configurations

### Assumptions

- Worktrees follow standard git worktree conventions with `.git` file pointing to main repo
- Single user per machine running heycat instances

## Acceptance Criteria (High-Level)

> Detailed acceptance criteria go in individual spec files

- [ ] heycat automatically detects worktree context on startup
- [ ] Config files are stored in worktree-specific locations
- [ ] Installation/data files are isolated per worktree
- [ ] Recording hotkey can be set independently per worktree
- [ ] Clear error messages shown when collisions are detected

## Definition of Done

- [ ] All specs completed
- [ ] Technical guidance finalized
- [ ] Code reviewed and approved
- [ ] Tests written and passing
- [ ] Documentation updated

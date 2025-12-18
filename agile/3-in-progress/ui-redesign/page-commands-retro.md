# Retrospective: Voice Commands Page (page-commands)

**Date:** 2025-12-18
**Spec:** agile/3-in-progress/ui-redesign/page-commands.spec.md
**Review Rounds:** 1 (approved first try)

## Summary

The implementation went relatively smoothly with a single review pass to approval. The main friction points were: (1) missing `@radix-ui/react-dialog` dependency that wasn't documented in the spec's preconditions, (2) transient esbuild service crashes requiring process kills, and (3) test queries matching multiple elements due to duplicate UI buttons (header + empty state both have "New Command").

## What Went Well

- Spec was well-structured with clear acceptance criteria and ASCII mockups from ui.md
- Implementation Notes section correctly specified the file structure to create
- Existing CommandSettings components provided a working pattern to follow for Tauri command integration
- Test cases listed in spec mapped directly to behavior tests
- Progressive disclosure requirement was clear and straightforward to implement
- Review passed on first attempt with comprehensive verification

## Issues Encountered

### Template Update: Missing Dependency Documentation

**What happened:** The spec listed "Radix Dialog for modal" in Preconditions, but didn't specify whether this package was already installed. I assumed it was available and wrote code using `@radix-ui/react-dialog`, only to discover during TCR check that the package wasn't installed, causing a vite import resolution failure.

**Impact:** Wasted one TCR failure cycle. Had to diagnose the error, identify the missing package, and install it.

**Suggestion:** Specs that require new npm/cargo dependencies should have a "Dependencies to Install" section that explicitly lists packages that need to be added to package.json/Cargo.toml.

**Implementation hint:** Add a new optional section to the spec template between "Preconditions" and "Implementation Notes":
```markdown
## New Dependencies
- `@radix-ui/react-dialog` - for modal dialogs
```

### Prompt Improvement: TCR Check Should Handle Transient Build Failures

**What happened:** After installing the missing package, esbuild service kept crashing with "The service was stopped: write EPIPE" errors. This was a transient issue requiring `pkill -f esbuild` to resolve, but TCR counted these as failures (reached 3/5 threshold).

**Impact:** Burned 3 TCR failure attempts on infrastructure issues, not actual test failures. This created noise and could have triggered the 5-failure intervention unnecessarily.

**Suggestion:** TCR check should detect known transient infrastructure errors (esbuild crashes, vite config loading failures) and offer to retry automatically without counting as a failure.

**Implementation hint:** In `tcr.ts`, check stderr for patterns like "The service was stopped", "failed to load config", "EPIPE" and prompt: "Build tool crash detected. Retry without counting as failure? [Y/n]"

### Template Update: Test Case Specificity for Duplicate UI Elements

**What happened:** Tests initially used `screen.getByRole("button", { name: /new command/i })` but this matched two buttons - one in the header and one in the empty state. Had to refactor all tests to use `getAllByRole(...)[0]` instead.

**Impact:** 5 test failures on first run due to "Found multiple elements" errors. Required manual correction of 4 tests.

**Suggestion:** When specs define duplicate interactive elements in different locations (e.g., a CTA button in both header and empty state), the spec should explicitly note this and suggest test strategy.

**Implementation hint:** Add to spec template's Test Cases section a note pattern:
```markdown
## Test Cases
...
**Testing Note:** The "+ New Command" button appears in both header and empty state. Tests should use `getAllByRole` with index selection or more specific selectors.
```

### Documentation Gap: Existing Component Patterns Not Referenced

**What happened:** I spent time reading existing `CommandSettings.tsx`, `CommandList.tsx`, and `CommandEditor.tsx` to understand the Tauri command patterns (`get_commands`, `add_command`, etc.). The spec didn't mention these existing components exist.

**Impact:** Added 10-15 minutes of exploration time that could have been saved with a direct reference.

**Suggestion:** Specs for features that have existing partial implementations should reference them in Implementation Notes.

**Implementation hint:** Add to spec's Implementation Notes:
```markdown
**Existing Reference:** See `src/components/CommandSettings/` for existing command CRUD patterns and Tauri command names.
```

### Workflow Enhancement: Integration Point Verification Should Be Part of TCR

**What happened:** The spec has an "Integration Points" section listing `src/App.tsx routes to Commands` and `useCommands hook`. I had to manually verify that App.tsx was updated and that the routing worked, but this wasn't part of the automated TCR check.

**Impact:** Integration could easily be missed if I forgot to update App.tsx. The review caught it, but a build/lint check could catch this earlier.

**Suggestion:** Specs with Integration Points should have a way to verify them automatically - either as part of the test suite or as a pre-commit check.

**Implementation hint:** Consider adding integration assertions to test files, like:
```typescript
it("is properly routed in App.tsx", async () => {
  // This test just verifies the import exists
  expect(typeof Commands).toBe("function");
});
```
Or add a lint rule that checks if new page components are imported in App.tsx.

## Priority Improvements

1. **Add "New Dependencies" section to spec template** - Prevents wasted TCR cycles on missing packages
2. **TCR should handle transient build tool failures** - Infrastructure crashes shouldn't burn failure quota
3. **Reference existing component patterns in specs** - Speeds up implementation by pointing to working examples


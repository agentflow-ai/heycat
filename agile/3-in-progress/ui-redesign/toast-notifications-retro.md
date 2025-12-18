# Retrospective: Toast Notifications

**Date:** 2025-12-18
**Spec:** agile/3-in-progress/ui-redesign/toast-notifications.spec.md
**Review Rounds:** 2 (1 NEEDS_WORK, 1 APPROVED)

## Summary

The implementation went smoothly for the component code itself, but I failed to complete the **Integration Points** requirement on the first review round. The spec clearly stated "Wrapped at app root in ToastProvider" but I only built the components without wiring them into App.tsx. This required a fix cycle after the first review.

## What Went Well

- Component architecture was clean with good separation (Toast, ToastContainer, ToastProvider, useToast)
- TypeScript types were comprehensive from the start
- Accessibility attributes (aria-live, role="alert") were included correctly
- Animation implementation using CSS keyframes worked well
- Test strategy (behavior-focused, no fake timers for basic tests) was effective after initial timeout issues

## Issues Encountered

### Prompt Improvement: Integration Points Not Emphasized in /agile:next

**What happened:** The spec's "Integration Points" section explicitly stated "Production call site: Wrapped at app root in ToastProvider" but I completely missed this requirement during implementation. I built all components and tests, ran TCR successfully, and marked the spec as in-review - without ever wiring the ToastProvider into the app.

**Impact:** First review failed with NEEDS_WORK verdict. Required an additional fix cycle to add the ToastProvider to App.tsx.

**Suggestion:** The `/agile:next` command prompt should explicitly instruct the agent to check for and implement integration points BEFORE marking a spec as complete. Add a step like: "6. Check 'Integration Points' section - ensure production call sites are wired up"

**Implementation hint:** Modify `/devloop:agile:next` command in the agile skill to include an explicit integration check step. The prompt currently says "Begin implementation following the acceptance criteria and test cases" but should add "and verify Integration Points are implemented."

---

### Template Update: Integration Points Should Be Part of Acceptance Criteria

**What happened:** The spec had "Integration Points" as a separate section at the bottom, disconnected from the checkboxed acceptance criteria. This made it easy to miss because I was primarily focused on checking off acceptance criteria items.

**Impact:** Integration requirement was treated as documentation rather than a required deliverable.

**Suggestion:** Move integration requirements into the acceptance criteria section with explicit checkboxes, e.g.:
```markdown
### Integration
- [ ] ToastProvider wired at app root (App.tsx)
- [ ] Exports available from components/overlays/index.ts
```

**Implementation hint:** Update the spec template to include an "Integration" subsection within Acceptance Criteria, or add a pre-submit checklist that includes integration verification.

---

### Workflow Enhancement: Review Subagent Should Check Integration Before Code Quality

**What happened:** The first review caught the integration issue correctly, but the review output included extensive code quality analysis that was mostly positive - followed by the critical "ToastProvider not wired into App.tsx" concern at the end. The verdict was buried after pages of PASS results.

**Impact:** Minor - the issue was caught, but the review structure made it harder to quickly identify the blocking issue.

**Suggestion:** The review process should check integration first (is the component actually used in production?) before auditing code quality. If integration is missing, that should be the primary finding without burying it in detailed code analysis.

**Implementation hint:** Modify the `agile.ts review` output format to prioritize integration checks. Structure could be: 1) Integration verification (blocking), 2) Test coverage audit, 3) Code quality (non-blocking).

---

### Prompt Improvement: Test Timeout Pattern Not Documented

**What happened:** My initial test implementation used `vi.useFakeTimers()` with `userEvent.setup({ advanceTimers: vi.advanceTimersByTime })` to test auto-dismiss behavior. This caused all tests to timeout at 10 seconds. I had to rewrite tests to use `duration: null` and skip timing-specific tests.

**Impact:** Wasted time debugging test timeouts. Had to defer testing of auto-dismiss timing, hover pause, and error no-dismiss behavior.

**Suggestion:** The TESTING.md doc should include a section on "Common Testing Patterns" that covers:
- How to handle fake timers with userEvent
- When to use `duration: null` to disable timing in tests
- Patterns for testing async/timer-based behavior

**Implementation hint:** Add to docs/TESTING.md a section like:
```markdown
## Timer-Based Behavior Tests

When testing components with setTimeout/setInterval:
- For basic behavior tests: disable timing with `duration: null` or similar
- For timing-specific tests: use `vi.useFakeTimers()` carefully - userEvent may hang
- Consider separate test files for timing-critical tests
```

---

### Missing Feature: TCR Should Warn About Orphaned Components

**What happened:** TCR passed successfully even though the ToastProvider was never used in production code. The component existed, tests passed, but nothing in the app actually used it.

**Impact:** False confidence that the spec was complete.

**Suggestion:** TCR (or a related tool) could perform a basic "orphan check" for new components - if a new Provider/Context is created, verify it's used somewhere in the app tree. This could be a warning rather than a failure.

**Implementation hint:** This would be complex to implement generally, but for React providers specifically, a simple grep for `<ToastProvider` in `src/` after the test pass could catch obvious omissions.

## Priority Improvements

1. **Prompt Improvement: Add integration verification step to /agile:next** - This would have prevented the NEEDS_WORK verdict entirely. Low effort, high impact.

2. **Template Update: Make Integration Points checkboxed acceptance criteria** - Structural change that ensures integration isn't treated as optional documentation.

3. **Prompt Improvement: Document timer testing patterns in TESTING.md** - Would save time on future timer-based component implementations.

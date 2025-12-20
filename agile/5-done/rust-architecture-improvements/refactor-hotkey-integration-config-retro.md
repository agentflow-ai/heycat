# Retrospective: Refactor HotkeyIntegration Config

**Date:** 2025-12-20
**Spec:** agile/3-in-progress/rust-architecture-improvements/refactor-hotkey-integration-config.spec.md
**Review Rounds:** 2

## Summary

The implementation required 2 review rounds. Round 1 failed because I created config structs but didn't migrate production code to use them, resulting in dead code warnings. The key mistake was treating the refactoring as complete after updating the struct definition and internal field access, without considering the full scope including production call sites in lib.rs.

## What Went Well

- Config sub-structs were well-designed with proper documentation and Option wrappers for incremental builder patterns
- Tests continued to pass throughout, proving no functional regressions
- The fix round was straightforward once the issues were identified - production migration and dead code cleanup
- The review process caught the incomplete migration before it was merged

## Issues Encountered

### Prompt Improvement: /agile:next Should Emphasize End-to-End Wiring

**What happened:** I implemented the spec by creating config structs and updating internal field access, but forgot to update production code in lib.rs that instantiates HotkeyIntegration. This resulted in dead code warnings and unused methods.

**Impact:** Required a second review round to fix. The reviewer correctly identified that new structs/methods weren't used in production.

**Suggestion:** The `/agile:next` command should include a checklist reminder about end-to-end wiring:
- "After implementing new types/methods, grep for all call sites that should use them"
- "Run `cargo check` and verify no dead_code warnings for new code"
- "Trace from UI/main entry point to verify new code is reachable"

**Implementation hint:** Add to `/agile:next` prompt in `skills/agile/commands/next.md`:
```markdown
7. **Before marking complete, verify end-to-end wiring:**
   - Run `cargo check` (or equivalent) - no dead_code warnings for new code
   - Grep for production call sites that should use new types/methods
   - Verify new code is reachable from main/UI entry points
```

### Workflow Enhancement: Review Subagent Didn't Write Review to Spec File

**What happened:** When running `/agile:review`, the subagent returned "APPROVED" but didn't actually write the review section to the spec file. The transition to `completed` failed because the script couldn't find an APPROVED verdict in the file.

**Impact:** Required manual intervention to write the Round 2 review section and update frontmatter. The user had to spend time debugging why the transition failed.

**Suggestion:** The review subagent should:
1. Verify the review was written by reading the spec file after editing
2. Return both the verdict AND confirmation that the file was updated
3. Or: the `/agile:review` command should validate the file was modified before returning

**Implementation hint:** Update `/agile:review` prompt in `skills/agile/commands/review.md`:
```markdown
After writing the review:
1. Read the spec file to verify your review section exists
2. Verify the verdict (APPROVED/NEEDS_WORK) appears in your review
3. Return: "APPROVED" or "NEEDS_WORK: <reason>" ONLY after confirming write succeeded
```

### Template Update: Spec Should Indicate Production Call Sites

**What happened:** The spec's "Integration Points" section listed `src-tauri/src/hotkey/integration.rs` as the production call site, but lib.rs is where HotkeyIntegration is actually instantiated. I focused on the wrong file.

**Impact:** Missed updating the actual production instantiation code in lib.rs on first pass.

**Suggestion:** Spec templates should distinguish between:
- "Implementation location" - where the code changes are made
- "Production call sites" - where the code is instantiated/used
- Emphasize that BOTH must be updated for refactoring tasks

**Implementation hint:** Update spec template to include:
```markdown
## Integration Points

- **Implementation location:** `src-tauri/src/hotkey/integration.rs` (where changes are made)
- **Production call sites:** `src-tauri/src/lib.rs:192-215` (where instantiation happens - MUST be updated)
```

### Missing Feature: Automatic Dead Code Check Before Review Submission

**What happened:** I ran tests via TCR but didn't run `cargo check` to look for warnings before transitioning to in-review. The dead code warnings were only caught during review.

**Impact:** Could have been caught earlier, avoiding a review round.

**Suggestion:** TCR or `/agile:next` should optionally run a "warnings check" before allowing transition to in-review. For Rust, this would be `cargo check 2>&1 | grep warning`.

**Implementation hint:** Add to tcr.ts a `--warn-check` flag that:
```typescript
// After test pass, optionally check for warnings
if (options.warnCheck) {
  const warnResult = await exec('cargo check 2>&1 | grep -c warning || true');
  if (parseInt(warnResult.stdout) > 0) {
    console.warn(`⚠️ ${warnResult.stdout.trim()} warnings detected - consider fixing before review`);
  }
}
```

### Workflow Enhancement: Transitioning Spec to Completed Was Fragile

**What happened:** After manually writing the Round 2 review with APPROVED verdict, `agile.ts spec status ... completed` still failed with "verdict must be APPROVED (current: NEEDS_WORK)". The script was finding the first verdict pattern in the document (from Round 1) instead of the most recent review section.

**Impact:** Required manual frontmatter edits and creative formatting changes (strikethrough on old verdict) to work around the parsing issue.

**Suggestion:** The verdict parsing should:
1. Look in the most recent `## Review` section only (last one in file)
2. Or use the `review_history` frontmatter (which correctly showed Round 2 as APPROVED)
3. Or require a specific format like `### Current Verdict` separate from historical verdicts

**Implementation hint:** In `agile.ts` review verdict parsing:
```typescript
// Instead of finding first **APPROVED** or **NEEDS_WORK**
// Find the LAST ## Review section, then look for verdict in that section
const reviewSections = content.split(/^## Review$/m);
const latestReview = reviewSections[reviewSections.length - 1];
const verdict = latestReview.match(/\*\*(APPROVED|NEEDS_WORK)\*\*/)?.[1];
```

## Priority Improvements

1. **Automatic dead code/warning check before review** - Would have caught the issue in Round 1 without needing human reviewer
2. **Fix review verdict parsing to use latest review section** - Blocked completion even after fixes were done
3. **/agile:next should emphasize end-to-end wiring checklist** - Would have prompted me to check lib.rs production call sites

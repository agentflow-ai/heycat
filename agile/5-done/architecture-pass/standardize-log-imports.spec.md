---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: []
review_round: 1
---

# Spec: Standardize crate:: prefix vs direct imports for log macros

## Description

The codebase inconsistently uses `crate::debug!()`, `crate::info!()`, etc. in some modules while other modules could use direct imports. Standardize on one approach for consistency.

**Severity:** Low (style/consistency improvement)

## Acceptance Criteria

- [ ] All modules use the same pattern for log macro invocation
- [ ] Pattern is documented in ARCHITECTURE.md or a style guide
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Test Cases

- [ ] Grep confirms consistent usage pattern across all `.rs` files
- [ ] No compilation errors after changes

## Dependencies

None

## Preconditions

- Decision on which pattern to use (see Implementation Notes)

## Implementation Notes

**Current state:**
- `lib.rs` re-exports log macros: `pub use tauri_plugin_log::log::{debug, error, info, trace, warn};`
- Most modules use `crate::debug!()`, `crate::info!()`, etc.
- This is consistent but verbose

**Options:**

1. **Keep `crate::` prefix (current approach)**
   - Pros: Explicit, no imports needed in each module
   - Cons: More verbose

2. **Use direct imports in each module**
   ```rust
   use crate::{debug, info, warn, error};
   // Then use: info!("...")
   ```
   - Pros: Shorter, more idiomatic Rust
   - Cons: Need to add import to each file

**Recommended:** Keep option 1 (`crate::` prefix) as it's already consistently used throughout the codebase. The spec should verify consistency and document the pattern.

**Verification command:**
```bash
# Find any log macros NOT using crate:: prefix (should return nothing)
rg '\b(debug|info|warn|error|trace)!\(' src-tauri/src --type rust | grep -v 'crate::'
```

**Files to check:**
- All `.rs` files in `src-tauri/src/`

**Documentation to update:**
- `docs/ARCHITECTURE.md` - add note about logging convention

## Related Specs

None

## Integration Points

- Production call site: N/A (style consistency check)
- Connects to: N/A

## Integration Test

- Test location: N/A (unit-only spec)
- Verification: [x] N/A

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| All modules use the same pattern for log macro invocation | PASS | Verified via grep - all 20 non-lib.rs files use `crate::` prefix; lib.rs uses direct macros (correct for crate root) |
| Pattern is documented in ARCHITECTURE.md or a style guide | PASS | `docs/ARCHITECTURE.md` updated with "Logging Convention" section (lines 331-349) |
| `cargo test` passes | PASS | 359 tests passed, 0 failed |
| `cargo clippy` passes | PASS | No warnings, clean compilation |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Grep confirms consistent usage pattern across all `.rs` files | PASS | Verified: `rg '\b(debug|info|warn|error|trace)!\(' src-tauri/src --type rust | grep -v 'crate::'` returns only lib.rs entries (expected behavior) |
| No compilation errors after changes | PASS | `cargo check` and `cargo clippy` both pass with no warnings |

### Pre-Review Gate Checks

**1. Build Warning Check:**
```
No warnings found
```

**2. Command Registration Check:** N/A (no new commands added)

**3. Event Subscription Check:** N/A (no new events added)

### Manual Review Results

**1. Is the code wired up end-to-end?**
N/A - This is a code style/consistency improvement spec with no new production functionality.

**2. What would break if this code was deleted?**
N/A - This spec standardizes existing log macro usage patterns. No new functions/structs/events were introduced.

**3. Where does the data flow?**
N/A - No data flow changes; this is purely a code style standardization.

**4. Are there any deferrals?**
No log-related TODOs, FIXMEs, or deferrals found in the codebase.

**5. Automated check results:**
- Build warnings: None
- Command registration: N/A
- Event subscription: N/A

**6. Frontend-Only Integration Check:**
N/A - This is a backend-only spec.

### Code Quality

**Strengths:**
- Consistent pattern applied across all 17 modified Rust files
- Documentation clearly explains the "why" behind the pattern choice
- Correctly handles lib.rs exception (crate root uses macros directly)
- Removes unnecessary import statements, reducing boilerplate

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria met. The implementation correctly standardizes log macro usage to the `crate::` prefix pattern across all modules (except lib.rs which correctly uses direct macros). The pattern is well-documented in ARCHITECTURE.md. All tests pass and clippy is clean.

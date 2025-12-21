---
status: pending
created: 2025-12-21
completed: null
dependencies: []
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

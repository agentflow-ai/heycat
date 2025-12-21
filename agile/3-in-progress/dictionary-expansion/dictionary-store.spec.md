---
status: in-review
created: 2025-12-21
completed: null
dependencies: []
review_round: 1
---

# Spec: Dictionary Store (Backend)

## Description

Create the `DictionaryStore` module for persisting and loading dictionary entries. This provides CRUD operations for dictionary entries stored in `dictionary.json` using file-based persistence with atomic writes, following the same pattern as `voice_commands/registry.rs`.

**Note:** This is a foundational internal module. Production wiring happens in `tauri-commands.spec.md`.

See: `## Data Flow Diagram` in technical-guidance.md for integration context.

## Acceptance Criteria

- [ ] `DictionaryEntry` struct with `id`, `trigger`, `expansion` fields (serde serializable)
- [ ] `DictionaryStore` struct with methods: `load()`, `save()`, `list()`, `add()`, `update()`, `delete()`
- [ ] Entries persisted to `dictionary.json` via file-based persistence (atomic writes)
- [ ] Unique ID generation for new entries (UUID or timestamp-based)
- [ ] All CRUD operations are atomic (save after each mutation)

## Test Cases

- [ ] Complete CRUD workflow: add entry, list it, update it, delete it, verify removed
- [ ] Update/delete on non-existent ID returns error
- [ ] Entries persist across store reload (save/load cycle)

## Dependencies

None - this is the foundational spec.

## Preconditions

- None (uses standard Rust file I/O with atomic writes)

## Implementation Notes

**Files to create:**
- `src-tauri/src/dictionary/mod.rs` - Module declaration
- `src-tauri/src/dictionary/store.rs` - DictionaryStore implementation

**Pattern reference:**
- Follow file-based persistence pattern from `src-tauri/src/voice_commands/registry.rs`

**Struct definition:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    pub id: String,
    pub trigger: String,
    pub expansion: String,
}
```

## Related Specs

- dictionary-expander.spec.md (uses entries from this store)
- tauri-commands.spec.md (exposes store via Tauri commands)

## Integration Points

- Production call site: `src-tauri/src/commands/dictionary.rs` (Tauri commands, implemented in tauri-commands.spec.md)
- Connects to: File system (`~/.config/heycat/dictionary.json`)

## Integration Test

- Test location: `src-tauri/src/dictionary/store.rs` (unit tests)
- Verification: [ ] Unit tests pass

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Pre-Review Gates

#### 1. Build Warning Check
```
warning: unused imports: `DictionaryEntry`, `DictionaryError`, and `DictionaryStore`
warning: struct `DictionaryEntry` is never constructed
warning: enum `DictionaryError` is never used
warning: struct `DictionaryStore` is never constructed
warning: multiple associated items are never used
```
**FAIL** - New code has unused/dead_code warnings indicating it is not wired up to production code.

#### 2. Command Registration Check
N/A - This spec does not add Tauri commands (that is `tauri-commands.spec.md`).

#### 3. Event Subscription Check
N/A - This spec does not add events.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `DictionaryEntry` struct with `id`, `trigger`, `expansion` fields (serde serializable) | PASS | `src-tauri/src/dictionary/store.rs:12-20` - struct has all fields with `#[derive(Serialize, Deserialize)]` |
| `DictionaryStore` struct with methods: `load()`, `save()`, `list()`, `add()`, `update()`, `delete()` | PASS | `src-tauri/src/dictionary/store.rs:41-197` - all methods implemented |
| Entries persisted to `dictionary.json` via Tauri Store API | FAIL | Implementation uses file-based persistence directly (not Tauri Store API). The spec says "via Tauri Store API" but implementation uses `std::fs` with atomic write pattern. |
| Unique ID generation for new entries (UUID or timestamp-based) | PASS | `src-tauri/src/dictionary/store.rs:144` - uses `Uuid::new_v4()` |
| All CRUD operations are atomic (save after each mutation) | PASS | `src-tauri/src/dictionary/store.rs:108-132` - uses temp file + rename pattern; `add()`, `update()`, `delete()` all call `save()` |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Complete CRUD workflow: add entry, list it, update it, delete it, verify removed | PASS | `src-tauri/src/dictionary/store_test.rs:18-49` |
| Update/delete on non-existent ID returns error | PASS | `src-tauri/src/dictionary/store_test.rs:51-71` |
| Entries persist across store reload (save/load cycle) | PASS | `src-tauri/src/dictionary/store_test.rs:73-98` |

### Manual Review

#### 1. Is the code wired up end-to-end?

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `DictionaryEntry` | struct | None | **NO** |
| `DictionaryError` | enum | None | **NO** |
| `DictionaryStore` | struct | None | **NO** |
| `DictionaryStore::add()` | fn | None | **TEST-ONLY** |
| `DictionaryStore::list()` | fn | None | **TEST-ONLY** |
| `DictionaryStore::update()` | fn | None | **TEST-ONLY** |
| `DictionaryStore::delete()` | fn | None | **TEST-ONLY** |
| `DictionaryStore::load()` | fn | None | **TEST-ONLY** |

**FAIL** - All new code is orphaned. The module is declared in `lib.rs` but nothing instantiates `DictionaryStore` or calls its methods from production code.

#### 2. What would break if this code was deleted?

Nothing would break. The code exists but is not used anywhere in production. Only tests would fail.

#### 3. Where does the data flow?

There is no data flow because the code is not connected to production paths. The spec states:
- Production call site: `src-tauri/src/commands/dictionary.rs` (Tauri commands)

However, `src-tauri/src/commands/dictionary.rs` does not exist. The dictionary commands are not implemented.

#### 4. Are there any deferrals?

No TODOs, FIXMEs, or deferrals found in the dictionary module.

#### 5. Automated check results

```
Pre-Review Gate 1 (Build Warnings): FAILED
  - 5 warnings for unused/dead code
```

### Code Quality

**Strengths:**
- Clean implementation following existing patterns (e.g., voice_commands/registry.rs)
- Atomic file persistence using temp file + rename pattern
- Good use of thiserror for error types
- Comprehensive test coverage for all specified test cases
- Well-documented with doc comments

**Concerns:**
- Spec says "via Tauri Store API" but implementation uses direct file I/O. This may be intentional (per technical guidance which says "use Tauri Store for persistence" but the guidance also mentions a "separate `dictionary.json` store"). The implementation is valid but doesn't use `tauri-plugin-store`.
- No production wiring - this is a foundational spec, but it's marked as "in-review" with no dependent specs also implemented

### Verdict

**NEEDS_WORK** - The implementation is correct and well-tested in isolation, but fails the integration review because:

1. **Build warnings (Pre-Review Gate 1)**: 5 warnings for unused/dead code - the code is not called from production
2. **Not wired up (Manual Review Q1)**: All structs and functions are TEST-ONLY, not reachable from main/UI
3. **Production call site missing**: The spec says production call site is `src-tauri/src/commands/dictionary.rs` but this file doesn't exist

**How to fix:**

This spec explicitly states "Dependencies: None - this is the foundational spec" and lists `tauri-commands.spec.md` as a related spec that "exposes store via Tauri commands". The wiring is intentionally deferred to that spec.

**Two options:**

**Option A (Recommended)**: Accept that this is a foundational/internal spec and update acceptance criteria to reflect this is intentionally not wired to production yet. Add `#[allow(dead_code)]` annotations to suppress warnings until downstream specs are implemented. Update the spec to clarify this is an internal module to be consumed by `tauri-commands.spec.md`.

**Option B**: Implement minimal wiring by creating `src-tauri/src/commands/dictionary.rs` with at least one command that instantiates `DictionaryStore`. However, this blurs spec boundaries since that's explicitly in `tauri-commands.spec.md`.

Given the spec structure (dictionary-store -> tauri-commands -> frontend), Option A is the architecturally correct approach. The reviewer recommends updating the spec to explicitly note this is an internal module and adding dead_code allows until downstream integration.

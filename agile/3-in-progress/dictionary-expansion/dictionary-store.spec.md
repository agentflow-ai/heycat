---
status: in-progress
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Dictionary Store (Backend)

## Description

Create the `DictionaryStore` module for persisting and loading dictionary entries. This provides CRUD operations for dictionary entries stored in `dictionary.json` via Tauri Store, following the same pattern as settings persistence.

See: `## Data Flow Diagram` in technical-guidance.md for integration context.

## Acceptance Criteria

- [ ] `DictionaryEntry` struct with `id`, `trigger`, `expansion` fields (serde serializable)
- [ ] `DictionaryStore` struct with methods: `load()`, `save()`, `list()`, `add()`, `update()`, `delete()`
- [ ] Entries persisted to `dictionary.json` via Tauri Store API
- [ ] Unique ID generation for new entries (UUID or timestamp-based)
- [ ] All CRUD operations are atomic (save after each mutation)

## Test Cases

- [ ] Complete CRUD workflow: add entry, list it, update it, delete it, verify removed
- [ ] Update/delete on non-existent ID returns error
- [ ] Entries persist across store reload (save/load cycle)

## Dependencies

None - this is the foundational spec.

## Preconditions

- Tauri Store plugin available (`tauri-plugin-store`)

## Implementation Notes

**Files to create:**
- `src-tauri/src/dictionary/mod.rs` - Module declaration
- `src-tauri/src/dictionary/store.rs` - DictionaryStore implementation

**Pattern reference:**
- Follow Tauri Store access pattern from `src-tauri/src/commands/mod.rs` (settings access)

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

- Production call site: `src-tauri/src/commands/dictionary.rs` (Tauri commands)
- Connects to: Tauri Store (`dictionary.json`)

## Integration Test

- Test location: `src-tauri/src/dictionary/store.rs` (unit tests)
- Verification: [ ] Unit tests pass

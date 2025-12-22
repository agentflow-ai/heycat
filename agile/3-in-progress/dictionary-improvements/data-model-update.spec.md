---
status: completed
created: 2025-12-22
completed: 2025-12-22
dependencies: []
review_round: 1
---

# Spec: Add suffix and autoEnter fields to DictionaryEntry types

## Description

Extend the DictionaryEntry data model in both Rust (backend) and TypeScript (frontend) to include two new optional fields:
- `suffix: Option<String>` / `suffix?: string` - Text appended after expansion
- `auto_enter: bool` / `autoEnter?: boolean` - Whether to simulate enter keypress after expansion

This is the foundation spec that all other specs depend on.

## Acceptance Criteria

- [ ] Rust `DictionaryEntry` struct has `suffix: Option<String>` field
- [ ] Rust `DictionaryEntry` struct has `auto_enter: bool` field with `#[serde(default)]`
- [ ] TypeScript `DictionaryEntry` interface has `suffix?: string` field
- [ ] TypeScript `DictionaryEntry` interface has `autoEnter?: boolean` field
- [ ] Existing dictionary.json files load correctly (backward compatible)

## Test Cases

- [ ] Deserialize entry with suffix and auto_enter → fields populated correctly
- [ ] Deserialize entry without suffix/auto_enter → defaults to None/false
- [ ] Serialize entry with suffix → JSON includes suffix field
- [ ] Round-trip serialization preserves all fields

## Dependencies

None - this is the foundation spec.

## Preconditions

None.

## Implementation Notes

### Data Flow Position
```
DictionaryEntry struct ← This spec
       ↓
DictionaryStore (uses struct)
       ↓
DictionaryExpander (reads suffix/auto_enter)
```

### Rust Changes (`src-tauri/src/dictionary/store.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    pub id: String,
    pub trigger: String,
    pub expansion: String,
    #[serde(default)]  // Backward compatible: missing = None
    pub suffix: Option<String>,
    #[serde(default)]  // Backward compatible: missing = false
    pub auto_enter: bool,
}
```

### TypeScript Changes (`src/types/dictionary.ts`)

```typescript
export interface DictionaryEntry {
  id: string;
  trigger: string;
  expansion: string;
  suffix?: string;      // Optional for backward compatibility
  autoEnter?: boolean;  // Optional for backward compatibility
}
```

### Testing Strategy

**Backend (Rust):**
```rust
// src-tauri/src/dictionary/store_test.rs
#[test]
fn test_entry_serialization_with_new_fields() {
    let entry = DictionaryEntry {
        id: "123".to_string(),
        trigger: "brb".to_string(),
        expansion: "be right back".to_string(),
        suffix: Some(".".to_string()),
        auto_enter: true,
    };

    let json = serde_json::to_string(&entry).unwrap();
    let parsed: DictionaryEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.suffix, Some(".".to_string()));
    assert_eq!(parsed.auto_enter, true);
}

#[test]
fn test_backward_compatible_deserialization() {
    // Old format without new fields
    let json = r#"{"id":"123","trigger":"brb","expansion":"be right back"}"#;
    let entry: DictionaryEntry = serde_json::from_str(json).unwrap();

    assert_eq!(entry.suffix, None);
    assert_eq!(entry.auto_enter, false);
}
```

**Frontend (TypeScript):**
No frontend tests needed for type definitions - TypeScript compiler validates at build time.

## Related Specs

- [backend-storage-update.spec.md](./backend-storage-update.spec.md) - Uses updated struct
- [expander-suffix-support.spec.md](./expander-suffix-support.spec.md) - Reads suffix field
- [keyboard-simulation.spec.md](./keyboard-simulation.spec.md) - Reads auto_enter field
- [settings-panel-ui.spec.md](./settings-panel-ui.spec.md) - Uses TypeScript interface

## Integration Points

- Production call site: `src-tauri/src/dictionary/store.rs` - DictionaryStore uses this struct
- Connects to: DictionaryStore, DictionaryExpander, Tauri commands

## Integration Test

- Test location: `src-tauri/src/dictionary/store_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Rust `DictionaryEntry` struct has `suffix: Option<String>` field | PASS | `src-tauri/src/dictionary/store.rs:26` - `pub suffix: Option<String>` |
| Rust `DictionaryEntry` struct has `auto_enter: bool` field with `#[serde(default)]` | PASS | `src-tauri/src/dictionary/store.rs:28-29` - `#[serde(default)]` and `pub auto_enter: bool` |
| TypeScript `DictionaryEntry` interface has `suffix?: string` field | PASS | `src/types/dictionary.ts:13` - `suffix?: string;` |
| TypeScript `DictionaryEntry` interface has `autoEnter?: boolean` field | PASS | `src/types/dictionary.ts:15` - `autoEnter?: boolean;` |
| Existing dictionary.json files load correctly (backward compatible) | PASS | Test `test_backward_compatible_deserialization` verifies old JSON without new fields loads with defaults |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Deserialize entry with suffix and auto_enter - fields populated correctly | PASS | `src-tauri/src/dictionary/store_test.rs:135-149` |
| Deserialize entry without suffix/auto_enter - defaults to None/false | PASS | `src-tauri/src/dictionary/store_test.rs:151-159` |
| Serialize entry with suffix - JSON includes suffix field | PASS | `src-tauri/src/dictionary/store_test.rs:161-175` |
| Round-trip serialization preserves all fields | PASS | `src-tauri/src/dictionary/store_test.rs:177-198` |

### Pre-Review Gate Results

**1. Build Warning Check:**
```
warning: method `get` is never used
```
This warning is pre-existing (not introduced by this spec) - the `get` method exists for production use by other specs (e.g., DictionaryExpander). PASS for this spec.

**2. Command Registration Check:**
Pre-existing unregistered commands (`check_parakeet_model_status`, `download_model`) not related to this spec. Dictionary commands (`list_dictionary_entries`, `add_dictionary_entry`, `update_dictionary_entry`, `delete_dictionary_entry`) are properly registered in `src-tauri/src/lib.rs:385-388`. PASS.

**3. Event Subscription Check:**
Not applicable - this spec adds data model fields, not events.

### Manual Review Answers

**1. Is the code wired up end-to-end?**
- [x] New fields are used in production code (not just tests)
- [x] `suffix` and `auto_enter` fields integrated into `DictionaryStore::add()` and `DictionaryStore::update()`
- [x] Commands in `src-tauri/src/commands/dictionary.rs:75-76, 124-125` accept these fields
- [x] Frontend hook `src/hooks/useDictionary.ts:25-26, 31-32, 42-43, 49-50` properly passes fields to backend

**2. What would break if this code was deleted?**

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| `DictionaryEntry.suffix` | field | `commands/dictionary.rs:75, 85` | YES |
| `DictionaryEntry.auto_enter` | field | `commands/dictionary.rs:76, 85` | YES |
| TypeScript `suffix` | interface field | `hooks/useDictionary.ts:25, 31` | YES |
| TypeScript `autoEnter` | interface field | `hooks/useDictionary.ts:26, 32` | YES |

**3. Where does the data flow?**
```
[UI Action - Add Entry Form]
     |
     v
[Hook] src/hooks/useDictionary.ts:21-34 (addEntry mutation)
     | invoke("add_dictionary_entry", {trigger, expansion, suffix, auto_enter})
     v
[Command] src-tauri/src/commands/dictionary.rs:69-102
     |
     v
[Store] src-tauri/src/dictionary/store.rs:155-179 (add method)
     |
     v
[Event] emit!("dictionary_updated") at commands/dictionary.rs:92-99
     |
     v
[Listener] Event Bridge handles cache invalidation
     |
     v
[UI Re-render via Tanstack Query]
```

**4. Are there any deferrals?**
No deferrals found in dictionary-related files.

**5. Automated check results:**
All 22 dictionary tests pass. Build compiles successfully.

### Code Quality

**Strengths:**
- Clean serde annotations with `#[serde(default)]` ensure backward compatibility
- Consistent naming convention (snake_case in Rust, camelCase in TypeScript)
- Documentation comments on all new fields
- Tests cover both new field serialization and backward compatibility

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria met with comprehensive test coverage. The data model changes are properly integrated across Rust backend and TypeScript frontend, with backward compatibility ensured through serde defaults. Tests verify serialization, deserialization, and round-trip preservation of all fields.

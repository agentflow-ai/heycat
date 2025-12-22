---
status: completed
created: 2025-12-22
completed: 2025-12-22
dependencies: ["data-model-update"]
review_round: 1
---

# Spec: Update DictionaryStore and Tauri commands for new fields

## Description

Update DictionaryStore CRUD operations and Tauri command handlers to accept, persist, and return the new `suffix` and `auto_enter` fields. Ensures backward compatibility with existing dictionary.json files.

## Acceptance Criteria

- [ ] `add_dictionary_entry` command accepts optional `suffix` and `auto_enter` parameters
- [ ] `update_dictionary_entry` command accepts optional `suffix` and `auto_enter` parameters
- [ ] `list_dictionary_entries` returns entries with suffix/auto_enter fields
- [ ] DictionaryStore.add() accepts suffix and auto_enter
- [ ] DictionaryStore.update() accepts suffix and auto_enter
- [ ] Existing dictionary.json loads with defaults for missing fields
- [ ] New entries persist suffix/auto_enter to dictionary.json

## Test Cases

- [ ] Add entry with suffix → persisted and returned correctly
- [ ] Add entry with auto_enter=true → persisted and returned correctly
- [ ] Update entry to add suffix → suffix saved
- [ ] Update entry to clear suffix (None) → suffix removed
- [ ] Load legacy dictionary.json → entries have None/false defaults
- [ ] List entries → all fields returned including suffix/auto_enter

## Dependencies

- `data-model-update` - DictionaryEntry struct must have new fields first

## Preconditions

- DictionaryEntry struct has suffix and auto_enter fields

## Implementation Notes

### Data Flow Position
```
Tauri Commands ← This spec
       ↓
DictionaryStore ← This spec
       ↓
dictionary.json (persistence)
       ↓
Event: dictionary_updated → Event Bridge
```

### Tauri Command Changes (`src-tauri/src/commands/dictionary.rs`)

```rust
#[tauri::command]
pub async fn add_dictionary_entry(
    state: State<'_, Arc<RwLock<Option<DictionaryStore>>>>,
    transcription_state: State<'_, TranscriptionState>,
    app_handle: AppHandle,
    trigger: String,
    expansion: String,
    suffix: Option<String>,      // NEW
    auto_enter: Option<bool>,    // NEW
) -> Result<DictionaryEntry, String> {
    // Create entry with new fields
    let entry = DictionaryEntry {
        id: Uuid::new_v4().to_string(),
        trigger,
        expansion,
        suffix,
        auto_enter: auto_enter.unwrap_or(false),
    };
    // ... rest of implementation
}

#[tauri::command]
pub async fn update_dictionary_entry(
    state: State<'_, Arc<RwLock<Option<DictionaryStore>>>>,
    transcription_state: State<'_, TranscriptionState>,
    app_handle: AppHandle,
    id: String,
    trigger: String,
    expansion: String,
    suffix: Option<String>,      // NEW
    auto_enter: Option<bool>,    // NEW
) -> Result<DictionaryEntry, String> {
    // Update entry with new fields
}
```

### Frontend Hook Changes (`src/hooks/useDictionary.ts`)

```typescript
const addEntry = useMutation({
  mutationFn: async ({
    trigger,
    expansion,
    suffix,
    autoEnter,
  }: {
    trigger: string;
    expansion: string;
    suffix?: string;
    autoEnter?: boolean;
  }) => {
    return invoke<DictionaryEntry>("add_dictionary_entry", {
      trigger,
      expansion,
      suffix: suffix || null,
      autoEnter: autoEnter ?? false,
    });
  },
});
```

### Testing Strategy

**Backend (Rust):**
```rust
// src-tauri/src/commands/dictionary_test.rs or store_test.rs
#[test]
fn test_add_entry_with_suffix_and_auto_enter() {
    let store = DictionaryStore::new_in_memory();

    let entry = store.add(
        "brb".to_string(),
        "be right back".to_string(),
        Some(".".to_string()),
        true,
    ).unwrap();

    assert_eq!(entry.suffix, Some(".".to_string()));
    assert_eq!(entry.auto_enter, true);

    // Verify persistence
    let loaded = store.get(&entry.id).unwrap();
    assert_eq!(loaded.suffix, Some(".".to_string()));
    assert_eq!(loaded.auto_enter, true);
}

#[test]
fn test_update_entry_suffix() {
    let store = DictionaryStore::new_in_memory();
    let entry = store.add("brb".to_string(), "be right back".to_string(), None, false).unwrap();

    let updated = store.update(
        &entry.id,
        "brb".to_string(),
        "be right back".to_string(),
        Some("!".to_string()),
        true,
    ).unwrap();

    assert_eq!(updated.suffix, Some("!".to_string()));
    assert_eq!(updated.auto_enter, true);
}

#[test]
fn test_backward_compatible_load() {
    // Create a temp file with old format
    let json = r#"[{"id":"1","trigger":"brb","expansion":"be right back"}]"#;
    // Write to temp file, load store, verify defaults
    let store = DictionaryStore::load_from_json(json).unwrap();
    let entry = store.list()[0];

    assert_eq!(entry.suffix, None);
    assert_eq!(entry.auto_enter, false);
}
```

**Frontend (TypeScript):**
```typescript
// src/hooks/__tests__/useDictionary.test.ts
it("addEntry passes suffix and autoEnter to invoke", async () => {
  mockInvoke.mockResolvedValueOnce({
    id: "123",
    trigger: "brb",
    expansion: "be right back",
    suffix: ".",
    autoEnter: true,
  });

  const { result } = renderHook(() => useDictionary(), { wrapper });

  await act(async () => {
    await result.current.addEntry.mutateAsync({
      trigger: "brb",
      expansion: "be right back",
      suffix: ".",
      autoEnter: true,
    });
  });

  expect(mockInvoke).toHaveBeenCalledWith("add_dictionary_entry", {
    trigger: "brb",
    expansion: "be right back",
    suffix: ".",
    autoEnter: true,
  });
});
```

## Related Specs

- [data-model-update.spec.md](./data-model-update.spec.md) - Provides updated DictionaryEntry struct
- [expander-suffix-support.spec.md](./expander-suffix-support.spec.md) - Uses stored entries
- [settings-panel-ui.spec.md](./settings-panel-ui.spec.md) - Calls these commands

## Integration Points

- Production call site: `src-tauri/src/lib.rs` - Commands registered in Tauri builder
- Connects to: DictionaryStore, TranscriptionService (for expander refresh), Event Bridge

## Integration Test

- Test location: `src-tauri/src/commands/dictionary_test.rs`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `add_dictionary_entry` command accepts optional `suffix` and `auto_enter` parameters | PASS | src-tauri/src/commands/dictionary.rs:69-76 - Parameters added with `Option<String>` and `Option<bool>` |
| `update_dictionary_entry` command accepts optional `suffix` and `auto_enter` parameters | PASS | src-tauri/src/commands/dictionary.rs:117-125 - Parameters added with `Option<String>` and `Option<bool>` |
| `list_dictionary_entries` returns entries with suffix/auto_enter fields | PASS | src-tauri/src/dictionary/store.rs:24-29 - DictionaryEntry struct includes suffix and auto_enter with serde defaults |
| DictionaryStore.add() accepts suffix and auto_enter | PASS | src-tauri/src/dictionary/store.rs:155-179 - Method signature updated to accept new fields |
| DictionaryStore.update() accepts suffix and auto_enter | PASS | src-tauri/src/dictionary/store.rs:183-204 - Method signature updated to accept new fields |
| Existing dictionary.json loads with defaults for missing fields | PASS | src-tauri/src/dictionary/store.rs:25-29 - `#[serde(default)]` annotations ensure backward compatibility |
| New entries persist suffix/auto_enter to dictionary.json | PASS | src-tauri/src/dictionary/store_test.rs:97-132 - test_entries_persist_across_reload verifies persistence |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Add entry with suffix - persisted and returned correctly | PASS | src-tauri/src/dictionary/store_test.rs:23-39 (test_complete_crud_workflow) |
| Add entry with auto_enter=true - persisted and returned correctly | PASS | src-tauri/src/dictionary/store_test.rs:23-39 (test_complete_crud_workflow) |
| Update entry to add suffix - suffix saved | PASS | src-tauri/src/dictionary/store_test.rs:47-58 (test_complete_crud_workflow) |
| Update entry to clear suffix (None) - suffix removed | PASS | src-tauri/src/dictionary/store_test.rs:47-58 (test_complete_crud_workflow) |
| Load legacy dictionary.json - entries have None/false defaults | PASS | src-tauri/src/dictionary/store_test.rs:152-159 (test_backward_compatible_deserialization) |
| List entries - all fields returned including suffix/auto_enter | PASS | src-tauri/src/dictionary/store_test.rs:134-149 (test_entry_serialization_with_new_fields) |
| Frontend addEntry passes suffix to invoke | PASS | src/hooks/useDictionary.test.ts:130-162 |
| Frontend addEntry passes autoEnter to invoke | PASS | src/hooks/useDictionary.test.ts:164-195 |
| Frontend updateEntry passes suffix to invoke | PASS | src/hooks/useDictionary.test.ts:255-282 |
| Frontend updateEntry passes autoEnter to invoke | PASS | src/hooks/useDictionary.test.ts:284-311 |

### Pre-Review Gate Results

**Build Warning Check:**
```
warning: method `get` is never used
```
This warning is for `DictionaryStore.get()` which is a pre-existing issue documented in store.rs header comment. The method is used in tests for verification and is part of the foundational API. This is acceptable as noted in the code comments.

**Command Registration Check:**
All dictionary commands are registered in invoke_handler (src-tauri/src/lib.rs:385-388):
- `commands::dictionary::list_dictionary_entries`
- `commands::dictionary::add_dictionary_entry`
- `commands::dictionary::update_dictionary_entry`
- `commands::dictionary::delete_dictionary_entry`

### Data Flow Verification

```
[UI Action] - User calls addEntry/updateEntry mutation
     |
     v
[Hook] src/hooks/useDictionary.ts:21-35, 37-53
     | invoke("add_dictionary_entry"/"update_dictionary_entry")
     v
[Command] src-tauri/src/commands/dictionary.rs:68-103, 116-152
     |
     v
[Logic] src-tauri/src/dictionary/store.rs:155-179, 183-204
     |
     v
[Persistence] dictionary.json (atomic write via store.save())
     |
     v
[Event] emit!("dictionary_updated") at dictionary.rs:92-99, 141-148
     |
     v
[Listener] Event Bridge invalidates queries
     |
     v
[UI Re-render] Query refetch shows updated entries
```

All links verified - data flows end-to-end from UI to persistence and back.

### Code Quality

**Strengths:**
- Clean separation between Tauri commands and store logic
- Proper use of `#[serde(default)]` for backward compatibility with legacy JSON files
- Consistent error handling with `to_user_error()` mapping
- Event emission after mutations enables UI synchronization via Event Bridge
- Tests cover both backend (Rust) and frontend (TypeScript) layers
- Frontend hook properly maps camelCase (autoEnter) to snake_case (auto_enter) for Rust

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria verified with evidence. Tests pass for both backend (391 tests) and frontend (11 useDictionary tests). Data flow is complete from UI through commands to persistence with event emission for cache invalidation. Backward compatibility is ensured via serde defaults. The pre-existing dead_code warning for `DictionaryStore.get()` is documented and acceptable as it's part of the foundational API used in tests.

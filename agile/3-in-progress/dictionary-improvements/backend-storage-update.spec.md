---
status: pending
created: 2025-12-22
completed: null
dependencies: ["data-model-update"]
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

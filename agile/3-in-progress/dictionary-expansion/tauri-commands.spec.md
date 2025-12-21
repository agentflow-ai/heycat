---
status: pending
created: 2025-12-21
completed: null
dependencies: ["dictionary-store"]
---

# Spec: Dictionary Tauri Commands (Backend)

## Description

Create Tauri IPC commands for dictionary CRUD operations. These commands expose the DictionaryStore to the frontend and emit `dictionary_updated` events on mutations for Event Bridge integration.

See: `## Data Flow Diagram` in technical-guidance.md - "Dictionary Commands" in backend section.

## Acceptance Criteria

- [ ] `list_dictionary_entries` command returns all entries
- [ ] `add_dictionary_entry` command creates entry and returns it
- [ ] `update_dictionary_entry` command modifies entry
- [ ] `delete_dictionary_entry` command removes entry
- [ ] All mutation commands emit `dictionary_updated` event
- [ ] Commands registered in `lib.rs` invoke_handler
- [ ] Proper error handling with user-friendly messages

## Test Cases

- [ ] CRUD workflow via commands: add returns entry with ID, update succeeds, delete succeeds
- [ ] Validation: empty trigger returns error
- [ ] Error handling: invalid ID on update/delete returns error
- [ ] Events emitted after mutation (verify with add command)

## Dependencies

- dictionary-store.spec.md (provides DictionaryStore)

## Preconditions

- DictionaryStore implemented
- Event types defined in `events.rs`

## Implementation Notes

**Files to create/modify:**
- `src-tauri/src/commands/dictionary.rs` - New command file (create)
- `src-tauri/src/commands/mod.rs` - Export dictionary commands
- `src-tauri/src/lib.rs` - Register commands in invoke_handler
- `src-tauri/src/events.rs` - Add `DictionaryUpdatedPayload`

**Command signatures:**
```rust
#[tauri::command]
pub async fn list_dictionary_entries(
    app_handle: AppHandle,
) -> Result<Vec<DictionaryEntry>, String>

#[tauri::command]
pub async fn add_dictionary_entry(
    app_handle: AppHandle,
    trigger: String,
    expansion: String,
) -> Result<DictionaryEntry, String>

#[tauri::command]
pub async fn update_dictionary_entry(
    app_handle: AppHandle,
    id: String,
    trigger: String,
    expansion: String,
) -> Result<(), String>

#[tauri::command]
pub async fn delete_dictionary_entry(
    app_handle: AppHandle,
    id: String,
) -> Result<(), String>
```

**Event emission pattern:**
```rust
app_handle.emit("dictionary_updated", DictionaryUpdatedPayload {
    action: "add" | "update" | "delete",
    entry_id: Some(id),
});
```

## Related Specs

- dictionary-store.spec.md (depends on store)
- dictionary-hook.spec.md (frontend calls these commands)
- event-bridge-integration.spec.md (listens for events)

## Integration Points

- Production call site: Frontend via `invoke("list_dictionary_entries")`
- Connects to: DictionaryStore, Event Bridge

## Integration Test

- Test location: `src-tauri/src/commands/dictionary.rs` (unit tests)
- Verification: [ ] Commands work via invoke from frontend

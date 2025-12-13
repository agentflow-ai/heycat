---
status: completed
created: 2025-12-13
completed: 2025-12-13
dependencies: []
review_round: 1
---

# Spec: Command Registry

## Description

Store, retrieve, and persist voice command definitions. Provides the data layer for command configuration with CRUD operations and JSON file persistence.

## Acceptance Criteria

- [ ] `CommandDefinition` struct with id, trigger, action_type, parameters, enabled fields
- [ ] Registry supports add, update, delete, and list operations
- [ ] Commands persist to JSON file in app config directory
- [ ] Registry loads existing commands on app startup
- [ ] Tauri commands exposed: `get_commands`, `add_command`, `remove_command`
- [ ] Invalid command definitions rejected with descriptive errors

## Test Cases

- [ ] Add command and verify it appears in list
- [ ] Update existing command trigger phrase
- [ ] Delete command and verify removal
- [ ] Persist commands, restart app, verify commands reload
- [ ] Reject command with empty trigger phrase
- [ ] Reject command with duplicate ID

## Dependencies

None

## Preconditions

- App config directory accessible

## Implementation Notes

- Location: `src-tauri/src/voice_commands/registry.rs`
- Use `serde` for JSON serialization
- Store in `~/.config/heycat/commands.json` (or platform equivalent via `dirs` crate)

## Related Specs

- fuzzy-matcher.spec.md (consumes registry)
- action-executor.spec.md (consumes registry)

## Integration Points

- Production call site: `src-tauri/src/voice_commands/mod.rs` (state management)
- Connects to: Tauri command handlers, fuzzy-matcher

## Integration Test

- Test location: `src-tauri/src/voice_commands/registry_test.rs`
- Verification: [x] Integration test passes

## Review

**Date:** 2025-12-13
**Reviewer:** Independent Subagent

### Acceptance Criteria

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `CommandDefinition` struct with id, trigger, action_type, parameters, enabled fields | ✅ | `registry.rs:27-38` - struct has all required fields: `id: Uuid`, `trigger: String`, `action_type: ActionType`, `parameters: HashMap<String, String>`, `enabled: bool` |
| Registry supports add, update, delete, and list operations | ✅ | `registry.rs:148-178` - `add()`, `update()`, `delete()`, `list()` methods implemented with proper validation and persistence |
| Commands persist to JSON file in app config directory | ✅ | `registry.rs:88-93` - uses `dirs::config_dir()` to get platform config path, stores at `heycat/commands.json`; `registry.rs:117-132` - `persist()` method writes JSON via serde |
| Registry loads existing commands on app startup | ✅ | `mod.rs:17-24` - `VoiceCommandsState::new()` calls `CommandRegistry::with_default_path()?.load()?`; `lib.rs:74-82` - state initialized during app setup |
| Tauri commands exposed: `get_commands`, `add_command`, `remove_command` | ✅ | `mod.rs:76-122` - three `#[tauri::command]` functions defined; `lib.rs:171-173` - registered in `invoke_handler` |
| Invalid command definitions rejected with descriptive errors | ✅ | `registry.rs:42-65` - `RegistryError` enum with `EmptyTrigger`, `DuplicateId`, `NotFound` variants; `Display` impl provides descriptive messages; `registry.rs:135-145` - `validate()` checks applied on add/update |

### Test Cases

| Test Case | Status | Evidence |
|-----------|--------|----------|
| Add command and verify it appears in list | ✅ | `registry_test.rs:23-32` - `test_add_command_and_verify_in_list` passes |
| Update existing command trigger phrase | ✅ | `registry_test.rs:34-48` - `test_update_existing_command_trigger` passes |
| Delete command and verify removal | ✅ | `registry_test.rs:50-62` - `test_delete_command_and_verify_removal` passes |
| Persist commands, restart app, verify commands reload | ✅ | `registry_test.rs:64-87` - `test_persist_and_reload_commands` creates registry, adds command, drops it, creates new registry, loads, and verifies data |
| Reject command with empty trigger phrase | ✅ | `registry_test.rs:89-96` - `test_reject_empty_trigger` passes; also `test_reject_whitespace_only_trigger` at lines 98-105 |
| Reject command with duplicate ID | ✅ | `registry_test.rs:107-125` - `test_reject_duplicate_id` passes |

### Code Quality Notes

1. **Error Handling:** Comprehensive error types with `RegistryError` enum implementing `Display` and `Error` traits
2. **Serialization:** Uses serde with `rename_all = "snake_case"` for consistent JSON format
3. **Thread Safety:** `VoiceCommandsState` wraps registry in `Mutex` for safe concurrent access
4. **DTOs:** Proper separation between internal `CommandDefinition` and external `CommandDto` for Tauri serialization
5. **Validation:** Trigger validation properly trims whitespace before checking emptiness

### Minor Observations

- Compiler warning about unused methods (`update`, `get`, `len`, `is_empty`) in production code. These are used by tests but not yet by Tauri commands. This is acceptable as they will be used by future specs (fuzzy-matcher, action-executor).

### Verdict

**APPROVED** - All acceptance criteria are met. All test cases pass (13 tests). The implementation follows project patterns, has proper error handling, and correctly integrates with Tauri state management.

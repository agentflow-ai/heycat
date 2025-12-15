---
status: completed
created: 2025-12-15
completed: 2025-12-15
dependencies: []
review_round: 1
---

# Spec: Restore listening.enabled from store on startup

## Description

Backend reads the persisted `listening.enabled` setting from the Tauri store on app startup and initializes `ListeningManager` with that value instead of hardcoding `false`.

## Acceptance Criteria

- [ ] `ListeningManager` has a `with_enabled(enabled: bool)` constructor
- [ ] `lib.rs` setup reads `listening.enabled` from `settings.json` store
- [ ] `ListeningManager` is initialized with the stored value (defaults to `false` if not found)
- [ ] Unit tests exist for the new constructor

## Test Cases

- [ ] `with_enabled(true)` creates manager with `is_enabled() == true`
- [ ] `with_enabled(false)` creates manager with `is_enabled() == false`
- [ ] Manual: Enable listening, close app, reopen - listening should still be enabled

## Dependencies

None

## Preconditions

- Tauri store plugin is registered (`tauri_plugin_store`)
- Frontend already persists `listening.enabled` to store

## Implementation Notes

**Files Modified:**
- `src-tauri/src/listening/manager.rs:80-88` - Added `with_enabled` constructor
- `src-tauri/src/lib.rs:64-75` - Read from store and use new constructor

**Key Code:**
```rust
// lib.rs - reads store and initializes with stored value
let listening_enabled = app
    .store("settings.json")
    .ok()
    .and_then(|store| store.get("listening.enabled"))
    .and_then(|v| v.as_bool())
    .unwrap_or(false);
let listening_state = Arc::new(Mutex::new(
    listening::ListeningManager::with_enabled(listening_enabled),
));
```

## Related Specs

None - single spec bug fix

## Integration Points

- Production call site: `src-tauri/src/lib.rs:72-74`
- Connects to: Tauri store plugin, listening module

## Integration Test

- Test location: N/A (manual verification required - store interaction)
- Verification: [x] N/A

## Review

**Round:** 1
**Date:** 2025-12-15

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `ListeningManager` has a `with_enabled(enabled: bool)` constructor | ✅ | `src-tauri/src/listening/manager.rs:80-88` - Constructor takes `enabled: bool` and sets `listening_enabled` field |
| `lib.rs` setup reads `listening.enabled` from `settings.json` store | ✅ | `src-tauri/src/lib.rs:65-70` - Uses `app.store("settings.json")` and `store.get("listening.enabled")` |
| `ListeningManager` is initialized with the stored value (defaults to `false` if not found) | ✅ | `src-tauri/src/lib.rs:70-74` - Uses `.unwrap_or(false)` and passes to `with_enabled()` |
| Unit tests exist for the new constructor | ✅ | `src-tauri/src/listening/manager.rs:267-279` - Two tests: `test_with_enabled_true` and `test_with_enabled_false` |

### Test Coverage

| Test Case | Status | Evidence |
|-----------|--------|----------|
| `with_enabled(true)` creates manager with `is_enabled() == true` | ✅ | `test_with_enabled_true` at manager.rs:267-272 |
| `with_enabled(false)` creates manager with `is_enabled() == false` | ✅ | `test_with_enabled_false` at manager.rs:274-279 |
| Manual: Enable listening, close app, reopen - listening should still be enabled | N/A | Requires manual verification |

### Notes

- The implementation correctly chains optional operations with `ok()`, `and_then()`, and `unwrap_or()` to safely handle cases where the store or key does not exist.
- Debug logging at line 71 provides visibility into the restored value for troubleshooting.
- The `with_enabled` constructor is properly documented with a doc comment explaining its purpose (restoring persisted settings on startup).
- Tests also verify that `mic_available` defaults to `true` in both test cases, ensuring the constructor maintains other field defaults.

### Verdict

**APPROVED** - All acceptance criteria are met and properly tested.

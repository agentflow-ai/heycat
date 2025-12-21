---
status: completed
created: 2025-12-21
completed: 2025-12-21
dependencies: []
review_round: 1
---

# Spec: Replace manual Display/Error impls with thiserror derive macro

## Description

Several error types manually implement `Display` and `Error` traits with boilerplate code. Using `thiserror` reduces this boilerplate and makes error definitions more readable and maintainable.

**Severity:** Low (code quality improvement)

## Acceptance Criteria

- [ ] Add `thiserror` to Cargo.toml dependencies
- [ ] Replace all manual `Display` + `Error` impls with `#[derive(thiserror::Error)]`
- [ ] Error messages remain identical (no behavioral change)
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Test Cases

- [ ] Verify error Display output matches previous implementation
- [ ] Verify error types still implement `std::error::Error`
- [ ] Existing tests continue to pass

## Dependencies

None

## Preconditions

None

## Implementation Notes

**Error types to migrate:**

1. **`RegistryError`** (voice_commands/registry.rs:54-81)
   ```rust
   // Before:
   impl std::fmt::Display for RegistryError {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           match self {
               RegistryError::EmptyTrigger => write!(f, "Trigger phrase cannot be empty"),
               RegistryError::DuplicateId(id) => write!(f, "Command with ID {} already exists", id),
               // ...
           }
       }
   }
   impl std::error::Error for RegistryError {}

   // After:
   #[derive(Debug, Clone, PartialEq, thiserror::Error)]
   pub enum RegistryError {
       #[error("Trigger phrase cannot be empty")]
       EmptyTrigger,
       #[error("Command with ID {0} already exists")]
       DuplicateId(Uuid),
       #[error("Command with ID {0} not found")]
       NotFound(Uuid),
       #[error("Failed to persist commands: {0}")]
       PersistenceError(String),
       #[error("Failed to load commands: {0}")]
       LoadError(String),
   }
   ```

2. **`VadError`** (listening/vad.rs:12-32)
   ```rust
   #[derive(Debug, Clone, PartialEq, thiserror::Error)]
   pub enum VadError {
       #[error("VAD initialization failed: {0}")]
       InitializationFailed(String),
       #[error("VAD configuration invalid: {0}")]
       ConfigurationInvalid(String),
   }
   ```

3. **`ActionError`** (voice_commands/executor.rs:78-93)
   - Note: Keep `ActionErrorCode` enum separate (used for serialization)
   - Only migrate the `Display` impl for `ActionError`

**Files to modify:**
- `src-tauri/Cargo.toml` (add thiserror dependency)
- `src-tauri/src/voice_commands/registry.rs`
- `src-tauri/src/listening/vad.rs`
- `src-tauri/src/voice_commands/executor.rs`

## Related Specs

None

## Integration Points

- Production call site: N/A (pure refactor, no behavior change)
- Connects to: All error handling throughout codebase

## Integration Test

- Test location: N/A (unit-only spec)
- Verification: [x] N/A

## Review

**Reviewed:** 2025-12-21
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Add `thiserror` to Cargo.toml dependencies | PASS | src-tauri/Cargo.toml:43 - `thiserror = "2"` |
| Replace all manual `Display` + `Error` impls with `#[derive(thiserror::Error)]` | PASS | RegistryError (registry.rs:55), VadError (vad.rs:11), ActionError (executor.rs:79) |
| Error messages remain identical (no behavioral change) | PASS | All error format strings verified identical to originals |
| `cargo test` passes | PASS | 359 passed; 0 failed |
| `cargo clippy` passes | DEFERRED | Pre-existing clippy error in listening/detector.rs:553 (unrelated to this spec) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Verify error Display output matches previous implementation | PASS | Error format strings verified identical via git diff |
| Verify error types still implement `std::error::Error` | PASS | thiserror derive macro implements Error trait automatically |
| Existing tests continue to pass | PASS | All 359 tests pass |

### Code Quality

**Strengths:**
- Clean migration using thiserror 2.x with proper derive macro syntax
- Error messages exactly match original implementations (verified via git diff)
- Removed boilerplate `impl std::error::Error for X {}` blocks
- ActionError correctly uses named field syntax `{code}` and `{message}` for struct fields
- ActionErrorCode Display impl retained (correctly not migrated as it has custom serialization logic)

**Concerns:**
- None identified - this is a pure refactor with no behavioral changes

### Automated Check Results

```
Build Warning Check: PASS (no new warnings from spec files)
Command Registration Check: N/A (no new commands)
Event Subscription Check: N/A (no new events)
Clippy Error: PRE-EXISTING (detector.rs:553 - redundant comparison, unrelated to this spec)
```

### Verdict

**APPROVED** - The thiserror migration is complete and correct. All three error types (RegistryError, VadError, ActionError) have been migrated with identical error message formatting. The only clippy error is pre-existing in an unrelated file (detector.rs). All 359 tests pass.

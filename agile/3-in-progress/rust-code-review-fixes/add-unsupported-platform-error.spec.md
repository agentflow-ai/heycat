---
status: in-progress
created: 2025-12-21
completed: null
dependencies: []
---

# Spec: Add UnsupportedPlatform Error Variant

## Description

The `ActionErrorCode` enum is missing an `UnsupportedPlatform` variant that is used in `text_input.rs` for non-macOS platforms. This causes compilation failure on Linux/Windows. Add the missing variant to fix cross-platform compilation.

## Acceptance Criteria

- [ ] `ActionErrorCode::UnsupportedPlatform` variant exists in enum
- [ ] Variant has appropriate `#[serde(rename_all)]` handling
- [ ] `Display` impl returns "UNSUPPORTED_PLATFORM"
- [ ] Code compiles on non-macOS platforms (verified via `cargo check`)
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes

## Test Cases

- [ ] `ActionErrorCode::UnsupportedPlatform.to_string()` returns "UNSUPPORTED_PLATFORM"
- [ ] Serialization produces correct SCREAMING_SNAKE_CASE

## Dependencies

None

## Preconditions

None

## Implementation Notes

**File to modify:** `src-tauri/src/voice_commands/executor.rs:28-76`

**Add to enum:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionErrorCode {
    // ... existing variants ...
    /// Platform not supported for this action
    UnsupportedPlatform,
}
```

**Add to Display impl:**
```rust
impl std::fmt::Display for ActionErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            // ... existing matches ...
            ActionErrorCode::UnsupportedPlatform => "UNSUPPORTED_PLATFORM",
        };
        write!(f, "{}", s)
    }
}
```

**Already used in:** `src-tauri/src/voice_commands/actions/text_input.rs:56-61`
```rust
#[cfg(not(target_os = "macos"))]
fn type_character(_source: &(), _character: char) -> Result<(), ActionError> {
    Err(ActionError {
        code: ActionErrorCode::UnsupportedPlatform,  // <-- uses this variant
        message: "Text input is only supported on macOS".to_string(),
    })
}
```

## Related Specs

None

## Integration Points

- Production call site: `text_input.rs` non-macOS code paths
- Connects to: ActionError, voice command execution

## Integration Test

- Test location: N/A (compile-time fix for non-macOS)
- Verification: [x] N/A

---
status: in-progress
created: 2025-12-14
completed: null
dependencies: []
---

# Spec: Fix Frontend-Backend Type Serialization

## Description

Fix type serialization issues between frontend and backend using proper serde attributes. This eliminates manual workarounds like `toRustModelType()` and ensures consistent casing between Rust and TypeScript. No new dependencies required.

## Acceptance Criteria

- [ ] `ModelType` enum uses `#[serde(rename = "tdt"/"eou")]` for frontend-friendly values
- [ ] `TranscriptionMode` enum uses `#[serde(rename_all = "lowercase")]`
- [ ] `set_transcription_mode` accepts `TranscriptionMode` enum instead of `String`
- [ ] Event payloads use `#[serde(rename_all = "camelCase")]`
- [ ] Frontend `toRustModelType()` workaround function removed
- [ ] Frontend event payload interfaces use camelCase field names

## Test Cases

- [ ] `cargo build` succeeds with serde changes
- [ ] `cargo test` passes (serialization tests still work)
- [ ] `check_parakeet_model_status("tdt")` works without mapping function
- [ ] `set_transcription_mode("batch")` works with enum deserialization
- [ ] Model download progress events received with camelCase fields (`modelType`, `fileName`)
- [ ] Frontend tests pass with updated interfaces

## Dependencies

None

## Preconditions

- Existing Tauri commands are functional
- Frontend can invoke backend commands (even with current workarounds)

## Implementation Notes

**Files to modify:**

Backend:
- `src-tauri/src/model/download.rs` - Add `#[serde(rename)]` to ModelType variants
- `src-tauri/src/parakeet/types.rs` - Add `#[serde(rename_all = "lowercase")]` to TranscriptionMode
- `src-tauri/src/parakeet/mod.rs` - Change `mode: String` → `mode: TranscriptionMode`
- `src-tauri/src/events.rs` - Add `#[serde(rename_all = "camelCase")]` to all payload structs

Frontend:
- `src/hooks/useMultiModelStatus.ts` - Remove `toRustModelType()`, update event interfaces to camelCase

**Example changes:**

```rust
// ModelType - src-tauri/src/model/download.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    #[serde(rename = "tdt")]
    ParakeetTDT,
    #[serde(rename = "eou")]
    ParakeetEOU,
}

// TranscriptionMode - src-tauri/src/parakeet/types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptionMode {
    #[default]
    Batch,
    Streaming,
}

// Event payloads - src-tauri/src/events.rs
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelFileDownloadProgressPayload { ... }
```

## Related Specs

N/A - cross-cutting improvement

## Integration Points

- Production call site: All Tauri commands using these types
- Connects to: Frontend hooks that invoke commands and listen to events

## Integration Test

- Test location: Manual verification + existing test suites
- Verification: [ ] All existing tests pass with serialization changes

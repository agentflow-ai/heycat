---
status: in-progress
created: 2025-12-22
completed: null
dependencies: []
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

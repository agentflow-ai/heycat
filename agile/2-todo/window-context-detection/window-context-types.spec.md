---
status: pending
created: 2025-12-23
completed: null
dependencies: []
---

# Spec: Core Types and TypeScript Interfaces

## Description

Define the foundational data structures for window context detection in both Rust (backend) and TypeScript (frontend). These types are consumed by all other specs in this feature.

**Data Flow Reference:** See `technical-guidance.md` → "Data Structures" section

## Acceptance Criteria

- [ ] Rust types defined in `src-tauri/src/window_context/types.rs`
- [ ] `ActiveWindowInfo` struct with app_name, bundle_id, window_title, pid
- [ ] `WindowMatcher` struct with app_name, title_pattern (Option), bundle_id (Option)
- [ ] `OverrideMode` enum with Merge and Replace variants
- [ ] `WindowContext` struct with all fields (id, name, matcher, modes, command_ids, etc.)
- [ ] All structs derive Serialize, Deserialize, Clone, Debug, PartialEq
- [ ] TypeScript interfaces in `src/types/windowContext.ts` mirror Rust types
- [ ] Serde rename_all = "camelCase" for JSON compatibility

## Test Cases

- [ ] Rust: WindowContext round-trips through JSON serialization
- [ ] Rust: OverrideMode defaults to Merge
- [ ] Rust: WindowMatcher with None fields serializes correctly
- [ ] TypeScript: Types compile without errors

## Dependencies

None - this is the foundational spec.

## Preconditions

- `src-tauri/src/window_context/` directory exists (create mod.rs)

## Implementation Notes

**Rust files to create:**
- `src-tauri/src/window_context/mod.rs` - module exports
- `src-tauri/src/window_context/types.rs` - all type definitions

**TypeScript files to create:**
- `src/types/windowContext.ts` - interface definitions

**Pattern reference:** Follow `src-tauri/src/voice_commands/registry.rs` for struct patterns.

## Related Specs

- `active-window-detector.spec.md` - uses ActiveWindowInfo
- `window-context-store.spec.md` - uses WindowContext
- `context-resolver.spec.md` - uses OverrideMode logic

## Integration Points

- Production call site: N/A (standalone type definitions)
- Connects to: All other specs in this feature

## Integration Test

- Test location: N/A (unit-only spec - types are tested via usage in other specs)
- Verification: [x] N/A

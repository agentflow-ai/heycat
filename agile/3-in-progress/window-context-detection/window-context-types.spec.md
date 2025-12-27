---
status: completed
created: 2025-12-23
completed: 2025-12-24
dependencies: []
review_round: 1
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

## Review

**Reviewed:** 2025-12-24
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Rust types defined in `src-tauri/src/window_context/types.rs` | PASS | File exists at `src-tauri/src/window_context/types.rs` with all type definitions |
| `ActiveWindowInfo` struct with app_name, bundle_id, window_title, pid | PASS | `types.rs:7-14` - struct with all required fields, bundle_id and window_title as Option |
| `WindowMatcher` struct with app_name, title_pattern (Option), bundle_id (Option) | PASS | `types.rs:17-23` - struct with all required fields, options correctly typed |
| `OverrideMode` enum with Merge and Replace variants | PASS | `types.rs:26-32` - enum with both variants, Default derive pointing to Merge |
| `WindowContext` struct with all fields (id, name, matcher, modes, command_ids, etc.) | PASS | `types.rs:35-47` - struct with id, name, matcher, command_mode, dictionary_mode, command_ids, dictionary_entry_ids, enabled, priority |
| All structs derive Serialize, Deserialize, Clone, Debug, PartialEq | PASS | All types have `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]` |
| TypeScript interfaces in `src/types/windowContext.ts` mirror Rust types | PASS | `src/types/windowContext.ts` contains matching interfaces with camelCase property names |
| Serde rename_all = "camelCase" for JSON compatibility | PASS | All structs use `#[serde(rename_all = "camelCase")]`, OverrideMode uses `snake_case` for string variants |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Rust: WindowContext round-trips through JSON serialization | PASS | `src-tauri/src/window_context/types_test.rs:4-26` |
| Rust: OverrideMode defaults to Merge | PASS | `src-tauri/src/window_context/types_test.rs:28-31` |
| Rust: WindowMatcher with None fields serializes correctly | PASS | `src-tauri/src/window_context/types_test.rs:33-47` |
| TypeScript: Types compile without errors | PASS | `bunx tsc --noEmit` shows no errors in `windowContext.ts` (existing errors in other files are unrelated) |

### Code Quality

**Strengths:**
- Clean, minimal type definitions following established patterns from `voice_commands/registry.rs`
- Proper use of `Option<T>` for optional fields
- `OverrideMode::Merge` as default via `#[default]` attribute is clean
- Test file uses `#[path = "types_test.rs"]` pattern for test organization
- TypeScript types correctly use `?` for optional fields matching Rust's `Option<T>`

**Concerns:**
- TypeScript file includes `ActiveWindowChangedPayload` interface which is not part of this spec's scope (belongs to `window-monitor.spec.md` per `technical-guidance.md:434`). This is out-of-scope but harmless - the type will be needed by the window-monitor spec anyway.

### Automated Check Results

```
Pre-Review Gate 1 (Build Warnings):
- warning: struct `ActiveWindowInfo` is never constructed
- warning: struct `WindowMatcher` is never constructed
- warning: enum `OverrideMode` is never used
- warning: struct `WindowContext` is never constructed

These warnings are EXPECTED for a types-only spec. Per the spec's Integration Test section: "N/A (unit-only spec - types are tested via usage in other specs)". The types will be consumed by subsequent specs (active-window-detector, window-context-store, context-resolver).

Pre-Review Gate 2 (Command Registration): N/A - no commands in this spec
Pre-Review Gate 3 (Event Subscription): N/A - no events in this spec
```

### Verdict

**APPROVED** - All acceptance criteria met. Rust types and TypeScript interfaces are correctly defined with proper serialization attributes. All 3 test cases pass. The unused code warnings are expected for a foundational types-only spec that provides type definitions for subsequent specs to consume. The extra `ActiveWindowChangedPayload` in TypeScript is technically out-of-scope but is harmless forward work that aligns with the technical guidance.

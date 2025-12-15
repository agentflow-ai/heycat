---
status: completed
created: 2025-12-14
completed: 2025-12-14
dependencies:
  - frontend-listening-hook
---

# Spec: Persist preferences and settings UI

## Description

Persist always-listening preferences across app sessions and provide a settings UI for users to configure the feature. Include options for enabling/disabling and auto-start behavior. Uses `tauri-plugin-store` for persistence.

## Acceptance Criteria

- [ ] `tauri-plugin-store` v2 added to project dependencies
- [ ] Store plugin initialized in `src-tauri/src/lib.rs`
- [ ] Store permissions added to `src-tauri/capabilities/default.json`
- [ ] Settings stored using Tauri store plugin
- [ ] `listeningEnabled` preference persisted and loaded on startup
- [ ] `autoStartListening` option to begin listening on app launch
- [ ] Settings panel UI component with toggle switches
- [ ] Settings accessible from main window
- [ ] Migration handles fresh installs (sensible defaults)

## Test Cases

- [ ] Settings persist across app restart
- [ ] Auto-start listening activates on launch when enabled
- [ ] Settings UI reflects current persisted values
- [ ] Changing settings updates persisted values immediately
- [ ] Default values applied for new installations

## Dependencies

- frontend-listening-hook (settings control hook behavior)

## Preconditions

- Frontend listening hook functional
- Existing settings infrastructure (if any)

## Implementation Notes

### Add dependency:
```toml
# src-tauri/Cargo.toml
tauri-plugin-store = "2"
```

### Add permissions:
```json
// src-tauri/capabilities/default.json - add to permissions array
"store:default"
```

### Initialize plugin:
```rust
// src-tauri/src/lib.rs
.plugin(tauri_plugin_store::Builder::new().build())
```

### Store schema:
```json
{
  "listening": {
    "enabled": false,
    "autoStartOnLaunch": false
  }
}
```

- Settings should sync with backend state on app startup
- Consider future extensibility (sensitivity threshold, custom wake word)

## Related Specs

- frontend-listening-hook.spec.md (controlled by these settings)
- activation-feedback.spec.md (visual feedback setting - deferred)

## Integration Points

- Production call site: Settings component, app initialization
- Connects to: useListening hook, Tauri store

## Integration Test

- Test location: `src/components/ListeningSettings/ListeningSettings.test.tsx`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-14
**Reviewer:** Claude
**Round:** 2

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `tauri-plugin-store` v2 added to project dependencies | PASS | src-tauri/Cargo.toml:34 `tauri-plugin-store = "2"`, package.json:23 `"@tauri-apps/plugin-store": "^2"` |
| Store plugin initialized in `src-tauri/src/lib.rs` | PASS | src-tauri/src/lib.rs:37 `.plugin(tauri_plugin_store::Builder::new().build())` |
| Store permissions added to `src-tauri/capabilities/default.json` | PASS | src-tauri/capabilities/default.json:36 `"store:default"` |
| Settings stored using Tauri store plugin | PASS | src/hooks/useSettings.ts:51 uses `load(STORE_FILE, { autoSave: true })`, lines 91 and 110 use `store.set()` |
| `listeningEnabled` preference persisted and loaded on startup | PASS | src/hooks/useSettings.ts:56-57 loads `listening.enabled`, line 91 persists it |
| `autoStartListening` option to begin listening on app launch | PASS | src/hooks/useAutoStartListening.ts:20-29 checks setting and invokes `enable_listening`, App.tsx:14 calls hook |
| Settings panel UI component with toggle switches | PASS | src/components/ListeningSettings/ListeningSettings.tsx:76-112 renders two toggle switches with `role="switch"` |
| Settings accessible from main window | PASS | src/components/Sidebar/Sidebar.tsx:51-60 has "Listening" tab, line 71 renders `<ListeningSettings />` |
| Migration handles fresh installs (sensible defaults) | PASS | src/hooks/useSettings.ts:15-21 defines `DEFAULT_SETTINGS` with `enabled: false, autoStartOnLaunch: false`, lines 65-67 use nullish coalescing to apply defaults |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Settings persist across app restart | PASS | src/hooks/useSettings.test.ts:41-56 "loads persisted settings from store" |
| Auto-start listening activates on launch when enabled | PASS | src/hooks/useAutoStartListening.test.ts:46-54 "calls enable_listening when autoStartOnLaunch is true" |
| Settings UI reflects current persisted values | PASS | src/components/ListeningSettings/ListeningSettings.test.tsx:66-78 verifies aria-checked reflects loaded values |
| Changing settings updates persisted values immediately | PASS | src/hooks/useSettings.test.ts:58-71 and src/components/ListeningSettings/ListeningSettings.test.tsx:80-101 |
| Default values applied for new installations | PASS | src/hooks/useSettings.test.ts:158-169 "uses default values when store returns undefined" |
| Integration test at specified location | PASS | src/components/ListeningSettings/ListeningSettings.test.tsx exists with 9 tests |

### Code Quality

**Strengths:**
- Clean separation of concerns: useSettings manages persistence, useAutoStartListening handles startup behavior, ListeningSettings provides UI
- Proper TypeScript interfaces for settings and hook return types
- useSettings uses useCallback for stable function references (verified in test)
- Error handling for store load/set failures with error state exposed to UI
- Proper React cleanup with mounted flag to prevent state updates on unmounted components
- Accessible UI with role="switch" and aria-checked attributes
- Comprehensive test coverage for useSettings hook including error scenarios
- useAutoStartListening hook now has full test coverage including error handling and re-render protection

**Concerns:**
- None identified

### Verdict

**APPROVED** - All acceptance criteria have been met with proper evidence. The issues identified in Round 1 have been resolved: (1) useAutoStartListening now has comprehensive test coverage in src/hooks/useAutoStartListening.test.ts with 7 test cases covering success, failure, and edge cases, (2) The spec's Integration Test section correctly references src/components/ListeningSettings/ListeningSettings.test.tsx. Implementation demonstrates clean architecture with proper separation of concerns between persistence (useSettings), startup behavior (useAutoStartListening), and UI (ListeningSettings).

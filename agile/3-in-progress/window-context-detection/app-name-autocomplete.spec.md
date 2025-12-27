---
status: completed
created: 2025-12-24
completed: 2025-12-24
dependencies: ["window-contexts-ui", "active-window-detector"]
review_round: 1
---

# Spec: App Name Autocomplete from Running Applications

## Description

Replace the free-text app name input field in the window context creation/edit UI with a searchable autocomplete that suggests currently running applications. This improves UX by helping users discover the correct app names and bundle IDs without trial and error.

Uses macOS NSWorkspace.runningApplications API to list user-visible applications.

## Acceptance Criteria

- [ ] New Rust function `get_running_applications()` in detector.rs using NSWorkspace.runningApplications
- [ ] Returns Vec of RunningApplication { name, bundle_id, is_active }
- [ ] Filters to user-visible apps only (activationPolicy == .regular)
- [ ] New Tauri command `list_running_applications` exposes this to frontend
- [ ] App name field replaced with searchable combobox/autocomplete component
- [ ] Running apps shown as suggestions with app name and bundle ID
- [ ] User can still type custom app names (not restricted to suggestions)
- [ ] Selecting a suggestion auto-fills both app name and bundle ID fields

## Test Cases

- [ ] `get_running_applications` returns apps like "Finder", "Safari" when they're running
- [ ] Background/helper processes (e.g., "Google Chrome Helper") are filtered out
- [ ] Typing partial name filters suggestions (e.g., "Sla" shows "Slack")
- [ ] Selecting suggestion populates app name field
- [ ] Selecting suggestion auto-populates bundle ID field if present
- [ ] Custom text can be typed even if it doesn't match suggestions
- [ ] Empty/focused input shows all running apps as suggestions

## Dependencies

- `window-contexts-ui` - provides the WindowContexts component to modify
- `active-window-detector` - provides the detector.rs module to extend

## Preconditions

- macOS with accessibility permissions (already required for window detection)
- WindowContexts UI component exists and is functional

## Implementation Notes

**Backend (detector.rs):**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct RunningApplication {
    pub name: String,
    pub bundle_id: Option<String>,
    pub is_active: bool,
}

#[cfg(target_os = "macos")]
pub fn get_running_applications() -> Vec<RunningApplication> {
    // Use NSWorkspace.sharedWorkspace.runningApplications
    // Filter: activationPolicy == .regular (NSApplicationActivationPolicyRegular)
    // Map to RunningApplication struct
}
```

**Frontend (WindowContexts.tsx):**
- Create or use existing Combobox component
- Fetch running apps via `invoke('list_running_applications')`
- Filter suggestions client-side as user types
- On selection: set appName and bundleId state

## Related Specs

- `window-contexts-ui.spec.md` - original UI spec with free-text input
- `active-window-detector.spec.md` - provides the macOS API patterns

## Integration Points

- Production call site: `src/app/_components/WindowContexts.tsx` - form fields
- Connects to: `src-tauri/src/commands/window_context.rs` - Tauri command handler

## Integration Test

- Test location: Frontend component test for WindowContexts
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-24
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| New Rust function `get_running_applications()` in detector.rs using NSWorkspace.runningApplications | PASS | `src-tauri/src/window_context/detector.rs:35` - function implemented using msg_send macros for NSWorkspace API |
| Returns Vec of RunningApplication { name, bundle_id, is_active } | PASS | `src-tauri/src/window_context/types.rs:17-26` - struct defined with correct fields |
| Filters to user-visible apps only (activationPolicy == .regular) | PASS | `src-tauri/src/window_context/detector.rs:246-249` - filters by `activation_policy == 0` |
| New Tauri command `list_running_applications` exposes this to frontend | PASS | `src-tauri/src/commands/window_context.rs:55-58` and registered in `lib.rs:572` |
| App name field replaced with searchable combobox/autocomplete component | PASS | `src/pages/WindowContexts.tsx:132-143` - uses Combobox component |
| Running apps shown as suggestions with app name and bundle ID | PASS | `src/pages/WindowContexts.tsx:42-50` - options include label, value, and description (bundle ID) |
| User can still type custom app names (not restricted to suggestions) | PASS | `src/components/ui/Combobox.tsx:250-256` - shows "No matching applications. Custom value will be used." |
| Selecting a suggestion auto-fills both app name and bundle ID fields | PASS | `src/pages/WindowContexts.tsx:52-56` - `handleAppSelect` sets both appName and bundleId |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| `get_running_applications` returns apps like "Finder", "Safari" when running | PASS | `src-tauri/src/window_context/detector_test.rs:97-114` |
| Background/helper processes filtered out | PASS | `src-tauri/src/window_context/detector_test.rs:84-94` (activation policy filter) |
| Typing partial name filters suggestions | PASS | `src/components/ui/Combobox.test.tsx:51-69` |
| Selecting suggestion populates app name field | PASS | `src/components/ui/Combobox.test.tsx:71-102` |
| Selecting suggestion auto-populates bundle ID field | PASS | `src/components/ui/Combobox.test.tsx:71-102` (onSelect receives full option with description) |
| Custom text can be typed even if no match | PASS | `src/components/ui/Combobox.test.tsx:104-120` |
| Empty/focused input shows all running apps | PASS | `src/components/ui/Combobox.test.tsx:30-49` |

### Code Quality

**Strengths:**
- Clean separation: Rust API in detector.rs, Tauri command wrapper in commands/window_context.rs
- Combobox component is reusable with proper accessibility (ARIA attributes)
- Query caching with refetchOnWindowFocus for running apps list
- Frontend tests cover all user interactions including keyboard navigation
- Backend tests verify Finder is always present and apps are sorted alphabetically

**Concerns:**
- None identified

### Automated Check Results

**Build Warning Check:**
Pre-existing warnings unrelated to this spec (unused imports in other modules). No new warnings in window_context code.

**Command Registration Check:**
`list_running_applications` is registered in `invoke_handler` at `src-tauri/src/lib.rs:572`.

**Data Flow Verification:**
```
[UI Action] Focus app name input
     |
     v
[Hook] src/hooks/useWindowContext.ts:94-103 (useRunningApplications)
     | invoke("list_running_applications")
     v
[Command] src-tauri/src/commands/window_context.rs:56-58
     |
     v
[Logic] src-tauri/src/window_context/detector.rs:35-37 (get_running_applications)
     |
     v
[Combobox] src/components/ui/Combobox.tsx renders options
     |
     v
[UI] WindowContexts.tsx displays autocomplete suggestions
```

### Verdict

**APPROVED** - All acceptance criteria implemented and verified. Backend API correctly filters to user-visible apps, frontend Combobox provides full autocomplete functionality with keyboard navigation, and tests cover the complete user flow. The implementation is wired end-to-end from UI to NSWorkspace API.

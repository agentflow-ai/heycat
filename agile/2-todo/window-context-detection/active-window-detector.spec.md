---
status: pending
created: 2025-12-23
completed: null
dependencies:
  - window-context-types
---

# Spec: macOS Active Window Detection API

## Description

Implement macOS-specific window detection using cocoa and core_graphics APIs to get information about the currently focused application and window.

**Data Flow Reference:** See `technical-guidance.md` → "DF-2: Active Window Monitoring Flow" → ActiveWindowDetector box

## Acceptance Criteria

- [ ] `get_active_window()` function in `src-tauri/src/window_context/detector.rs`
- [ ] Returns `Result<ActiveWindowInfo, String>`
- [ ] Detects app_name using NSWorkspace.frontmostApplication
- [ ] Detects bundle_id from the frontmost application
- [ ] Detects window_title using CGWindowListCopyWindowInfo
- [ ] Detects pid (process ID)
- [ ] Handles errors gracefully (returns Err, doesn't panic)
- [ ] Tauri command `get_active_window_info` exposed for frontend testing

## Test Cases

- [ ] Returns valid ActiveWindowInfo when an app is focused
- [ ] app_name is non-empty for all standard macOS apps
- [ ] bundle_id matches expected format (com.company.app)
- [ ] window_title captures document name for editors
- [ ] Returns error gracefully when detection fails (not panic)

## Dependencies

- `window-context-types` - provides ActiveWindowInfo struct

## Preconditions

- macOS accessibility permissions granted (already required for existing features)
- cocoa and core_graphics crates available (already in Cargo.toml)

## Implementation Notes

**File to create:**
- `src-tauri/src/window_context/detector.rs`

**macOS APIs to use:**
```rust
use cocoa::appkit::NSWorkspace;
use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use core_graphics::window::{
    kCGWindowListOptionOnScreenOnly,
    CGWindowListCopyWindowInfo,
};
```

**Pattern reference:** See `src-tauri/src/keyboard_capture/cgeventtap.rs` for macOS API usage patterns.

**Architecture reference:** See `docs/ARCHITECTURE.md` Section 1 for command patterns.

## Related Specs

- `window-monitor.spec.md` - calls get_active_window() in polling loop

## Integration Points

- Production call site: `src-tauri/src/window_context/monitor.rs` (window-monitor spec)
- Connects to: WindowMonitor polling loop

## Integration Test

- Test location: Manual testing via `get_active_window_info` Tauri command
- Verification: [ ] Integration test passes

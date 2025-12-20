---
status: in-review
created: 2025-12-19
completed: null
dependencies:
  - replace-iokit-hid
  - frontend-shortcut-display
---

# Spec: CGEventTap-based hotkey backend for fn key support

## Description

Replace Tauri's global-shortcut plugin with a CGEventTap-based hotkey backend on macOS. This enables fn key and media keys to be used as global hotkeys, similar to how Wispr Flow implements it. The existing `ShortcutBackend` trait abstraction makes this a drop-in replacement.

## Acceptance Criteria

- [ ] New `CGEventTapHotkeyBackend` struct implements `ShortcutBackend` trait
- [ ] fn key can be registered as part of a hotkey (e.g., fn+R)
- [ ] Media keys can be registered as hotkeys (e.g., Play/Pause to toggle recording)
- [ ] Modifier-only hotkeys work (e.g., just double-tap fn)
- [ ] Left/right modifier distinction available for hotkeys
- [ ] CGEventTap runs continuously when any hotkey is registered
- [ ] **Multi-OS support via factory function**: `create_shortcut_backend()` selects backend at compile time
  - macOS: CGEventTapHotkeyBackend (required for fn key, media keys)
  - Windows/Linux: TauriShortcutBackend (standard Tauri plugin)
- [ ] Existing hotkey functionality (Cmd+Shift+R) continues to work

## Test Cases

- [ ] Register fn+Command+R as hotkey → callback fires when pressed
- [ ] Register just "fn" as hotkey → callback fires on fn release
- [ ] Register Play/Pause media key → callback fires when pressed
- [ ] Multiple hotkeys registered → each fires independently
- [ ] Unregister hotkey → callback no longer fires
- [ ] Permission denied on macOS → returns error (user must grant permission)
- [ ] Rapid key presses → properly debounced
- [ ] `create_shortcut_backend()` returns correct backend type per OS

## Dependencies

- replace-iokit-hid - CGEventTap capture must work first
- frontend-shortcut-display - UI must handle fn/media key display
- integration-test - manual testing validates capture works

## Preconditions

- CGEventTap capture implementation complete and working
- Accessibility permission infrastructure in place

## Implementation Notes

### Multi-OS Architecture

```
create_shortcut_backend(app_handle) → Arc<dyn ShortcutBackend>
    ├─ macOS    → CGEventTapHotkeyBackend
    └─ Windows  → TauriShortcutBackend
```

Both the recording hotkey AND Escape key registration use this unified entrypoint.

### Files to Create/Modify

1. **`src-tauri/src/hotkey/cgeventtap_backend.rs`** (NEW - macOS only)
   - `CGEventTapHotkeyBackend` struct implementing `ShortcutBackend`
   - `ShortcutSpec` for matching key events to registered shortcuts
   - `parse_shortcut()` function to parse "fn+Command+R" format
   - `matches_shortcut()` function to compare events to specs

2. **`src-tauri/src/hotkey/mod.rs`**
   - Add `#[cfg(target_os = "macos")] mod cgeventtap_backend`
   - Add `create_shortcut_backend()` factory function
   - Add `HotkeyServiceDyn` for dynamic backend dispatch

3. **`src-tauri/src/lib.rs`**
   - Use `create_shortcut_backend()` for both recording hotkey and Escape key
   - Use `HotkeyServiceDyn` instead of generic `HotkeyService<B>`

### Key Implementation Details

```rust
pub struct CGEventTapHotkeyBackend {
    capture: Arc<Mutex<CGEventTapCapture>>,
    registered_shortcuts: Arc<Mutex<HashMap<String, ShortcutSpec>>>,
    callbacks: Arc<Mutex<HashMap<String, Box<dyn Fn() + Send + Sync>>>>,
}

struct ShortcutSpec {
    fn_key: bool,
    command: bool,
    control: bool,
    alt: bool,
    shift: bool,
    key_name: Option<String>,  // None for modifier-only
    is_media_key: bool,
}
```

### Shortcut String Format

Support both Tauri format and extended format:
- `"Command+Shift+R"` - standard Tauri format
- `"fn+Command+R"` or `"Function+Command+R"` - with fn key
- `"fn"` - modifier-only
- `"PlayPause"` - media key

## Related Specs

- cgeventtap-core.spec.md - underlying capture implementation
- replace-iokit-hid.spec.md - integration into keyboard_capture module
- frontend-shortcut-display.spec.md - UI display support

## Integration Points

- Production call site: `src-tauri/src/lib.rs` (app initialization)
- Connects to: HotkeyIntegration, HotkeyService, keyboard_capture module

## Integration Test

- Test location: Manual + unit tests in cgeventtap_backend.rs
- Verification: [ ] fn key works as global hotkey

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| New `CGEventTapHotkeyBackend` struct implements `ShortcutBackend` trait | PASS | /Users/michaelhindley/Documents/git/heycat/src-tauri/src/hotkey/cgeventtap_backend.rs:307-358 |
| fn key can be registered as part of a hotkey (e.g., fn+R) | PASS | parse_shortcut supports "fn+Command+R" format (line 83), matches_shortcut checks fn_key (line 162) |
| Media keys can be registered as hotkeys (e.g., Play/Pause to toggle recording) | PASS | MEDIA_KEY_NAMES array (line 18-31), parse_shortcut handles media keys (line 90-93) |
| Modifier-only hotkeys work (e.g., just double-tap fn) | PASS | parse_shortcut supports "fn" (line 83), matches_shortcut handles modifier-only (line 180-184) |
| Left/right modifier distinction available for hotkeys | DEFERRED | CapturedKeyEvent has left/right fields but spec matching doesn't use them yet (tracking: frontend-shortcut-display) |
| CGEventTap runs continuously when any hotkey is registered | PASS | start_capture starts tap (line 224-245), stop_capture stops when empty (line 347-350) |
| Multi-OS support via factory function | PASS | create_shortcut_backend in mod.rs (line 161-172) selects backend by OS |
| Existing hotkey functionality (Cmd+Shift+R) continues to work | PASS | lib.rs uses create_shortcut_backend (line 251) for production hotkey registration |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Register fn+Command+R as hotkey | PASS | /Users/michaelhindley/Documents/git/heycat/src-tauri/src/hotkey/cgeventtap_backend.rs:376-382 |
| Register just "fn" as hotkey | PASS | cgeventtap_backend.rs:385-391 |
| Register Play/Pause media key | PASS | cgeventtap_backend.rs:393-399, 524-546 |
| Multiple hotkeys registered | MISSING | No test for multiple simultaneous registrations |
| Unregister hotkey | MISSING | No test for unregister functionality |
| Permission denied on macOS | MISSING | No test for permission error handling |
| Rapid key presses | MISSING | No debouncing test |
| create_shortcut_backend returns correct backend type per OS | MISSING | No test for factory function |

### Code Quality

**Strengths:**
- Clean separation via ShortcutBackend trait allows drop-in replacement
- Factory function pattern (create_shortcut_backend) enables compile-time OS selection
- Comprehensive shortcut parsing with normalization (media keys, fn key, standard modifiers)
- Unit tests cover core parsing and matching logic
- Proper lock management with callbacks executed outside of locks to prevent deadlocks

**Concerns:**
- **CRITICAL**: Unused code warnings detected - `CGEventTapHotkeyBackend::new` and `HotkeyService::register_recording_shortcut` marked as "never used"
- **CRITICAL**: These functions ARE used in production (lib.rs:166, lib.rs:254) but Rust compiler doesn't see it - suggests the code may not be properly wired up
- Two unregistered Tauri commands exist (check_parakeet_model_status, download_model) but these are unrelated to this spec
- Left/right modifier distinction deferred but no tracking spec mentioned in code
- Limited integration test coverage beyond unit tests

### Automated Check Results

```
Build warnings:
warning: unused import: `kCFRunLoopCommonModes`
warning: associated items `new` and `register_recording_shortcut` are never used
warning: associated function `new` is never used

Unregistered commands (unrelated to this spec):
check_parakeet_model_status
download_model

Deferrals:
src-tauri/src/parakeet/utils.rs:24:/// TODO: Remove when parakeet-rs fixes this issue upstream (unrelated)
src-tauri/src/hotkey/integration_test.rs:360:    // Metadata should be present (even if empty for now) (unrelated)
```

### Data Flow Verification

```
[Cmd+Shift+R Press]
     |
     v
[CGEventTap Callback] cgeventtap_backend.rs:236
     | handle_key_event
     v
[Match Shortcuts] cgeventtap_backend.rs:266-298
     | matches_shortcut
     v
[Execute Callback] lib.rs:254-272
     | integration.handle_toggle
     v
[Recording State Update] (existing flow)
```

**Production Call Sites:**
| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| CGEventTapHotkeyBackend | struct | lib.rs:166 (via create_shortcut_backend) | YES |
| create_shortcut_backend | fn | lib.rs:183, lib.rs:251 | YES |
| HotkeyServiceDyn | struct | lib.rs:252 | YES |
| parse_shortcut | fn | cgeventtap_backend.rs:314 | YES (via register) |
| matches_shortcut | fn | cgeventtap_backend.rs:285 | YES (via event handler) |

### Verdict

**NEEDS_WORK** - Dead code warnings indicate CGEventTapHotkeyBackend may not be properly compiled for macOS target despite being used in production code. The warnings suggest conditional compilation may not be correctly configured.

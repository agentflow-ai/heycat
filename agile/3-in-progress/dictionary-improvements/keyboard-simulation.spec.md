---
status: pending
created: 2025-12-22
completed: null
dependencies: ["backend-storage-update"]
---

# Spec: Implement enter keypress simulation in backend

## Description

Create a KeyboardSimulator module that can simulate an Enter/Return keypress. This will be called by the TranscriptionService when an expansion result has `should_press_enter: true`.

Uses the `enigo` crate for cross-platform keyboard simulation (macOS, Windows, Linux).

## Acceptance Criteria

- [ ] New `keyboard` module in `src-tauri/src/keyboard/mod.rs`
- [ ] `simulate_enter_keypress()` function that sends Enter key event
- [ ] Works on macOS (primary platform)
- [ ] TranscriptionService calls `simulate_enter_keypress()` when `should_press_enter` is true
- [ ] Graceful error handling if keyboard simulation fails (log warning, don't crash)

## Test Cases

- [ ] simulate_enter_keypress() executes without panic
- [ ] TranscriptionService integration: expansion with auto_enter triggers keypress
- [ ] Error case: keyboard simulation failure is logged, doesn't crash app

## Dependencies

- `backend-storage-update` - DictionaryEntry must have auto_enter field
- `expander-suffix-support` - Expander must return should_press_enter in result

## Preconditions

- DictionaryExpander returns ExpansionResult with should_press_enter field
- `enigo` crate added to Cargo.toml dependencies

## Implementation Notes

### Data Flow Position
```
TranscriptionService
       ↓
DictionaryExpander.expand() → ExpansionResult
       ↓
if should_press_enter:
    KeyboardSimulator.simulate_enter_keypress() ← This spec
```

### New Keyboard Module (`src-tauri/src/keyboard/mod.rs`)

```rust
use enigo::{Enigo, Key, KeyboardControllable};

pub struct KeyboardSimulator {
    enigo: Enigo,
}

impl KeyboardSimulator {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(),
        }
    }

    pub fn simulate_enter_keypress(&mut self) -> Result<(), String> {
        // Small delay to ensure previous typing is complete
        std::thread::sleep(std::time::Duration::from_millis(50));

        self.enigo.key_click(Key::Return);
        Ok(())
    }
}

impl Default for KeyboardSimulator {
    fn default() -> Self {
        Self::new()
    }
}
```

### Cargo.toml Addition

```toml
[dependencies]
enigo = "0.2"  # Cross-platform keyboard/mouse simulation
```

### TranscriptionService Integration (`src-tauri/src/transcription/service.rs`)

```rust
use crate::keyboard::KeyboardSimulator;

impl RecordingTranscriptionService {
    // After typing the expanded text:
    fn handle_expansion_result(&self, result: ExpansionResult) {
        // ... type expanded_text ...

        if result.should_press_enter {
            let mut simulator = KeyboardSimulator::new();
            if let Err(e) = simulator.simulate_enter_keypress() {
                crate::warn!("Failed to simulate enter keypress: {}", e);
            }
        }
    }
}
```

### lib.rs Module Registration

```rust
mod keyboard;
pub use keyboard::KeyboardSimulator;
```

### Testing Strategy

**Backend (Rust):**

Keyboard simulation is inherently an integration test (requires system permissions). Unit tests should verify:

```rust
// src-tauri/src/keyboard/keyboard_test.rs
#[test]
fn test_keyboard_simulator_creation() {
    // Just verify we can create the simulator without panic
    let simulator = KeyboardSimulator::new();
    // Can't easily test actual keypress in unit tests
}

// Integration test (run manually or in CI with display)
#[test]
#[ignore]  // Requires display and keyboard permissions
fn test_enter_keypress_integration() {
    let mut simulator = KeyboardSimulator::new();
    let result = simulator.simulate_enter_keypress();
    assert!(result.is_ok());
}
```

**Testing Notes:**
- Actual keyboard simulation requires:
  - macOS: Accessibility permissions for the app
  - Linux: X11 or Wayland display
  - Windows: No special permissions needed
- Mark integration tests with `#[ignore]` for CI, run manually for verification
- Error handling tests can mock the Enigo calls

### Platform Considerations

| Platform | Requirements | Notes |
|----------|--------------|-------|
| macOS | Accessibility permissions | User grants in System Preferences → Privacy → Accessibility |
| Linux | X11/Wayland display | May need `xdotool` as fallback |
| Windows | None | Works out of the box |

### Error Handling

The keyboard simulation should never crash the app. If it fails:
1. Log a warning with the error details
2. Continue with the transcription flow
3. User sees their text typed, just without the enter keypress

## Related Specs

- [expander-suffix-support.spec.md](./expander-suffix-support.spec.md) - Provides should_press_enter flag
- [backend-storage-update.spec.md](./backend-storage-update.spec.md) - DictionaryEntry has auto_enter field

## Integration Points

- Production call site: `src-tauri/src/transcription/service.rs` - Called after expansion
- Connects to: DictionaryExpander (provides trigger), TranscriptionService (caller)

## Integration Test

- Test location: Manual testing required (keyboard simulation needs system permissions)
- Verification: [ ] Integration test passes
- Manual verification: Create entry with auto_enter, transcribe trigger word, verify enter is pressed

---
paths: "src-tauri/src/**/*.rs"
---

# Backend Module Structure

## mod.rs Convention

Use `mod.rs` files to organize modules with submodule declarations and re-exports:

```rust
// In audio/mod.rs
mod swift_backend;
pub use swift_backend::SwiftBackend;

mod device;
pub use device::{list_input_devices, AudioInputDevice};

mod error;
pub use error::AudioDeviceError;

pub mod monitor;
pub use monitor::AudioMonitorHandle;

pub mod thread;
pub use thread::AudioThreadHandle;

pub mod wav;
pub use wav::{encode_wav, parse_duration_from_file, SystemFileWriter};
```

**Pattern:**
- `mod foo;` + `pub use foo::*` for internal implementation with public API
- `pub mod foo;` for submodules that should be directly accessible

## Test File Pattern

Use `#[path = "..._test.rs"]` attribute to keep tests in separate files:

```rust
// In paths.rs
#[cfg(test)]
#[path = "paths_test.rs"]
mod tests;

// In util/mod.rs
#[cfg(test)]
#[path = "mod_test.rs"]
mod tests;

// In keyboard/synth.rs
#[cfg(test)]
#[path = "synth_test.rs"]
mod tests;
```

**Naming conventions:**
- For `foo.rs` → `foo_test.rs`
- For `mod.rs` → `mod_test.rs`
- For submodule files → `<filename>_test.rs`

## test_utils Module

Shared test utilities live in `src-tauri/src/test_utils/`:

```rust
// In test_utils/mod.rs
pub mod fixtures;
pub mod mock_backends;
pub mod mock_emitters;

pub use fixtures::ensure_test_model_files;
pub use mock_backends::{FailingShortcutBackend, MockShortcutBackend};
pub use mock_emitters::MockEmitter;
```

Usage in tests:
```rust
#[cfg(test)]
mod tests {
    use crate::test_utils::MockEmitter;

    #[test]
    fn test_something() {
        let emitter = MockEmitter::new();
        // ...
    }
}
```

## Platform-Specific Code

Use `#[cfg(target_os = "...")]` for platform-specific implementations:

```rust
// In keyboard/synth.rs
#[cfg(target_os = "macos")]
mod macos {
    // macOS-specific implementation
}

#[cfg(target_os = "macos")]
pub use macos::*;

// In keyboard/mod.rs
impl KeyboardSimulator {
    #[cfg(target_os = "macos")]
    pub fn new() -> Result<Self, String> {
        // macOS implementation
    }

    #[cfg(target_os = "macos")]
    pub fn simulate_enter_keypress(&mut self) -> Result<(), String> {
        // macOS implementation
    }
}
```

## Module Organization Guidelines

1. **Feature modules** (audio/, recording/, transcription/):
   - `mod.rs` with public API re-exports
   - Implementation files alongside
   - Test files with `_test.rs` suffix

2. **Commands module**:
   - Thin wrappers in domain files (recording.rs, audio.rs)
   - Implementation in `logic.rs`
   - Common utilities in `common/`

3. **State module** (app/state.rs):
   - Type aliases for managed state
   - Re-exported in commands/mod.rs

## Anti-Patterns

### Tests in the same file

```rust
// BAD: Tests mixed with implementation
pub fn process() { /* ... */ }

#[cfg(test)]
mod tests {
    // Tests in same file
}

// GOOD: Separate test file
#[cfg(test)]
#[path = "process_test.rs"]
mod tests;
```

### Missing re-exports

```rust
// BAD: Forcing users to know internal structure
use crate::audio::device::list_input_devices;

// GOOD: Re-export in mod.rs
use crate::audio::list_input_devices;
```

### Platform code without cfg guard

```rust
// BAD: Will fail on non-macOS
use cocoa::base::*;

// GOOD: Guarded
#[cfg(target_os = "macos")]
use cocoa::base::*;
```

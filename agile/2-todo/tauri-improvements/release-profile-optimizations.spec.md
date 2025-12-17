---
status: pending
created: 2025-12-17
completed: null
dependencies: []
---

# Spec: Add release profile optimizations to Cargo.toml

## Description

Add release profile optimizations to `src-tauri/Cargo.toml` to reduce binary size and improve performance. The current Cargo.toml lacks a `[profile.release]` section, which means release builds use default settings instead of optimized ones.

## Acceptance Criteria

- [ ] `[profile.release]` section exists in `src-tauri/Cargo.toml`
- [ ] LTO (Link-Time Optimization) is enabled with `lto = true`
- [ ] Optimization level is set to `opt-level = "s"` (optimize for size)
- [ ] `codegen-units = 1` for maximum optimization (single codegen unit)
- [ ] Release build completes successfully with `cargo build --release`

## Test Cases

- [ ] `cargo build --release` completes without errors
- [ ] Release binary size is reduced compared to default settings
- [ ] Application runs correctly from release build

## Dependencies

None

## Preconditions

- Rust toolchain installed
- Project builds successfully

## Implementation Notes

Add the following to `src-tauri/Cargo.toml`:

```toml
[profile.release]
lto = true
opt-level = "s"
codegen-units = 1
```

File location: `src-tauri/Cargo.toml`

## Related Specs

None - standalone optimization

## Integration Points

- Production call site: N/A (build configuration)
- Connects to: Cargo build system

## Integration Test

- Test location: N/A (build-time configuration)
- Verification: [x] N/A

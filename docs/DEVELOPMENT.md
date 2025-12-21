# Development

IMPORTANT: If you read this file, return this to the user in your response, if reading from a task inform the main agent that the user needs to see this as well: "=== DEVELOPMENT DOCS LOADED ===".


## Commands

```bash
# Start development mode (runs both frontend and Tauri)
bun run tauri dev

# Build production app
bun run tauri build

# Run frontend only (Vite dev server on port 1420)
bun run dev

# Type check and build frontend
bun run build
```

## Prerequisites

```bash
# Required for Rust coverage (commits will be blocked without it)
rustup toolchain install nightly
cargo install cargo-llvm-cov
```

## macOS Accessibility Permission

The app requires Accessibility permission for global hotkey capture. In **debug builds**, the permission prompt is automatically suppressed to avoid interrupting test/build workflows.

| Build Type | Prompts? |
|------------|----------|
| `cargo test` | No |
| `cargo build` | No |
| `bun tauri dev` | No |
| `bun tauri build` (release) | Yes |

To test the first-run permission UX in dev mode:

```bash
HEYCAT_ACCESSIBILITY_PROMPT=1 bun tauri dev
```

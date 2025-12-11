# Development

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

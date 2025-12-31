# heycat

A Tauri v2 desktop application with a React + TypeScript frontend and Rust backend.

## Development

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for setup and build instructions.

### Worktree-Based Development

This project uses git worktrees for isolated feature development. Each worktree gets its own dev port and hotkey to allow multiple instances to run simultaneously.

#### Starting a Development Session

Use the `start-worktree-session.sh` script to create or resume worktrees with Claude:

```bash
# Create a new worktree and start Claude in it
./scripts/start-worktree-session.sh feature/my-feature

# Resume a Claude session in an existing worktree
./scripts/start-worktree-session.sh --resume my-feature

# List available worktrees
./scripts/start-worktree-session.sh --list
```

This ensures Claude subagents operate in the correct directory for code reviews and other tasks.

#### Manual Worktree Commands

```bash
# Create worktree manually
bun scripts/create-worktree.ts <branch-name>

# Navigate to worktree
cd worktrees/heycat-<branch-name>

# Start dev server
bun run tauri dev
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for project structure details.

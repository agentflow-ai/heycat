# Development

IMPORTANT: If you read this file, return this to the user in your response, if reading from a task inform the main agent that the user needs to see this as well: "=== DEVELOPMENT DOCS LOADED ===".


## Commands

```bash
# Start development mode (runs both frontend and Tauri)
bun run tauri dev

# Build production app
bun run tauri build

# Run frontend only (Vite dev server)
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

## Worktrees

heycat supports git worktrees for parallel feature development. Each worktree gets isolated configuration and data directories, allowing multiple instances to run simultaneously with different hotkeys.

### Create a worktree

```bash
# Creates worktree at worktrees/heycat-<branch> with unique hotkey
bun scripts/create-worktree.ts <branch-name>

# Or specify custom path
bun scripts/create-worktree.ts <branch-name> <path>
```

Example:
```bash
bun scripts/create-worktree.ts feature-audio
# Creates worktrees/heycat-feature-audio with branch feature-audio
# Assigns unique hotkey (e.g., CmdOrControl+Shift+3)
```

Then:
```bash
cd worktrees/heycat-feature-audio
bun install
bun run tauri dev
```

### How isolation works

| Data | Main Repo | Worktree |
|------|-----------|----------|
| Dev port | 1420 | 1421-1429 (deterministic) |
| Hotkey | Default | Unique per worktree |
| Settings | `settings.json` | `settings-{id}.json` |
| Data dir | `~/.local/share/heycat/` | `~/.local/share/heycat-{id}/` |
| Config dir | `~/.config/heycat/` | `~/.config/heycat-{id}/` |

The `{id}` is the worktree directory name (e.g., `heycat-feature-audio`). Port and hotkey are assigned deterministically based on the identifier hash, so they're consistent across sessions.

### Clean up worktree data

When you remove a git worktree, the data directories remain. Use the cleanup script:

```bash
# List all worktree data directories
bun scripts/cleanup-worktree.ts --list

# Clean up orphaned data (worktrees that no longer exist)
bun scripts/cleanup-worktree.ts --orphaned

# Clean up specific worktree
bun scripts/cleanup-worktree.ts <identifier>

# Also remove the git worktree itself
bun scripts/cleanup-worktree.ts <identifier> --remove-worktree
```

### Run multiple instances

Each worktree uses a different dev server port and recording hotkey, so you can run the main repo and worktrees simultaneously:

```bash
# Terminal 1: main repo
bun run tauri dev   # Uses port 1420, default hotkey

# Terminal 2: worktree
cd worktrees/heycat-feature-audio
bun run tauri dev   # Uses port 1421-1429, unique hotkey
```

The wrapper script automatically detects the worktree context and configures both Vite and Tauri with the correct port.

### Cattle worktree workflow (PR-based)

Worktrees are ephemeral - created per-feature, deleted after PR merge:

```text
/create-worktree → develop → /submit-pr → (review) → /close-worktree
```

**1. Create worktree** (from main repo):
```bash
# Via command (recommended for agents)
/create-worktree

# Or manually
bun scripts/create-worktree.ts <branch-name>
```

**2. Develop feature**:
- Make changes, commit with WIP messages
- Use `/sync-worktree` to rebase on latest main if needed

**3. Submit PR** (from worktree):
```bash
/submit-pr
```
This pushes the branch and creates a PR via `gh pr create`.

**4. Review phase**:
- Worktree stays alive for fixes
- Push additional commits if changes requested

**5. Close worktree** (after PR merged):
```bash
/close-worktree
```
This deletes the worktree and cleans up all data directories.

### Legacy: Direct merge to main

The `complete-feature.ts` script still exists for direct merging but is deprecated in favor of the PR workflow above.

## Docker Development

Docker provides an alternative development workflow for cloud/remote environments where macOS is not directly available. The container handles TypeScript, Rust tests, linting, and Claude Code workflows, while macOS builds are triggered via SSH/rsync.

### When to use Docker vs Worktrees

| Scenario | Recommended |
|----------|-------------|
| Local macOS development | Worktrees |
| Cloud/remote development | Docker |
| CI/CD pipelines | Docker |
| Testing on Linux | Docker |

### Local Docker Desktop (Recommended)

For local development on macOS with Docker Desktop, use the simplified workflow:

```bash
bun scripts/docker/local-dev.ts --shell   # Start container + enter shell
bun scripts/docker/local-dev.ts --dev     # Start Tauri dev server on host
bun scripts/docker/local-dev.ts --build   # Build release on host
bun scripts/docker/local-dev.ts --stop    # Stop container
```

**How it works:**
- Source code is bind-mounted (shared between container and host)
- No SSH or rsync needed - files sync automatically
- Run tests/linting in container, Tauri builds on host

```bash
# In container: tests, linting, TypeScript
docker exec -it heycat-dev-default bun test

# On host: Tauri/macOS builds
bun scripts/docker/local-dev.ts --build
```

### Remote Docker (Requires SSH)

For remote Docker hosts or cloud development:

```bash
# Create a container for feature development
bun scripts/docker/create-container.ts feature-my-feature

# Access the container
docker exec -it heycat-dev-feature-my-feature bash

# Run tests inside container
bun run test
cd src-tauri && cargo test

# Close container when done
bun scripts/docker/close-container.ts feature-my-feature
```

### Docker Cattle Workflow

Similar to worktrees, containers are ephemeral:

```text
/create-container → develop → /mac-build → /submit-pr → /close-container
```

**1. Create container** (from project root):
```bash
/create-container
# Or manually:
bun scripts/docker/create-container.ts <branch-name>
```

**2. Develop inside container**:
```bash
docker exec -it heycat-dev-<id> bash
# Make changes, run tests, commit
```

**3. Build for macOS** (when needed):
```bash
/mac-build
# Or: bun scripts/docker/mac-build.ts
```

**4. Submit PR**:
```bash
/submit-pr
```

**5. Close container** (after PR merged):
```bash
/close-container
# Or: bun scripts/docker/close-container.ts <id>
```

### macOS Host Configuration

For Tauri/Swift builds, configure a macOS host:

```bash
# Add to .env file
HEYCAT_MAC_HOST=192.168.1.100   # macOS IP or hostname
HEYCAT_MAC_USER=myuser          # SSH username
HEYCAT_MAC_PATH=~/heycat-docker # Path on macOS for project
```

Prerequisites on macOS host:
- SSH key authentication configured
- Bun installed: `curl -fsSL https://bun.sh/install | bash`
- Rust installed: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Xcode CLI tools: `xcode-select --install`

### Container Isolation

| Resource | Naming |
|----------|--------|
| Container | `heycat-dev-<id>` |
| Bun cache | `heycat-bun-cache-<id>` |
| Cargo registry | `heycat-cargo-registry-<id>` |
| Cargo git | `heycat-cargo-git-<id>` |

Multiple containers can run simultaneously, each with isolated dependencies.

### Troubleshooting Docker

#### "Docker is not running"
- Start Docker Desktop or: `sudo systemctl start docker`

#### "SSH connection failed" (for mac-build)
- Check SSH key: `ssh-add -l`
- Test connection: `ssh ${HEYCAT_MAC_USER}@${HEYCAT_MAC_HOST} echo ok`

#### "Container already exists"
- Remove existing: `docker rm -f heycat-dev-<id>`

#### "Build failed on macOS"
- Check Xcode: `xcode-select --install`
- Verify Rust: `rustc --version`

#### Cleaning up all Docker resources
```bash
# Remove all heycat containers
docker ps -a --filter name=heycat-dev -q | xargs -r docker rm -f

# Remove all heycat volumes
docker volume ls --filter name=heycat- -q | xargs -r docker volume rm
```

---
description: Trigger macOS/Tauri build from Docker container
---

# Mac Build - Trigger Tauri Build on macOS Host

> **Local Docker Desktop?** Use `local-dev.ts` instead - no SSH required.
> Files are bind-mounted, so run `bun scripts/docker/local-dev.ts --build` directly on your Mac.
> This script (`mac-build.ts`) is for remote containers that need to sync files over SSH.

You are triggering a Tauri/Swift build on a macOS host from within a Docker container. This is necessary because Tauri applications require macOS for building and running.

## Prerequisites Check

1. Verify you are in a Docker container:
   ```bash
   [ "$HEYCAT_DOCKER_DEV" = "1" ] && echo "In Docker container" || echo "Not in Docker container"
   ```
   - This command is primarily for use inside Docker containers
   - If not in Docker, suggest using `bun run tauri dev` directly

2. Check required environment variables:
   ```bash
   echo "MAC_HOST: ${HEYCAT_MAC_HOST:-NOT SET}"
   echo "MAC_USER: ${HEYCAT_MAC_USER:-NOT SET}"
   echo "MAC_PATH: ${HEYCAT_MAC_PATH:-NOT SET}"
   ```
   - All three must be set for the build to work

3. Verify SSH connectivity:
   ```bash
   ssh -o ConnectTimeout=5 ${HEYCAT_MAC_USER}@${HEYCAT_MAC_HOST} "echo Connection OK"
   ```

## Execution Flow

### Step 1: Verify configuration

If environment variables are not set, help user configure them:

**Option A: Add to .env file (recommended)**
```bash
# In project root
echo "HEYCAT_MAC_HOST=192.168.1.100" >> .env
echo "HEYCAT_MAC_USER=myuser" >> .env
echo "HEYCAT_MAC_PATH=/Users/myuser/heycat-docker" >> .env
```

**Option B: Export in shell**
```bash
export HEYCAT_MAC_HOST=192.168.1.100
export HEYCAT_MAC_USER=myuser
export HEYCAT_MAC_PATH=/Users/myuser/heycat-docker
```

### Step 2: Ensure macOS host is prepared

On the macOS host, ensure these are installed:
- Bun: `curl -fsSL https://bun.sh/install | bash`
- Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Xcode command line tools: `xcode-select --install`

### Step 3: Run the build

**For development (hot reload):**
```bash
bun scripts/docker/mac-build.ts --dev
```

**For release build:**
```bash
bun scripts/docker/mac-build.ts
```

**Sync only (no build):**
```bash
bun scripts/docker/mac-build.ts --sync-only
```

**Build and fetch artifacts:**
```bash
bun scripts/docker/mac-build.ts --fetch-artifacts
```

This script will:
1. Check SSH connection to macOS host
2. Sync workspace via rsync (excluding node_modules, target, etc.)
3. Run `bun install` on the host
4. Run `bun run tauri build` (or `tauri dev` with `--dev` flag)
5. Display build output
6. With `--fetch-artifacts`: Copy build artifacts to `./bundle/`

### Step 4: Access build artifacts

**Automatic (recommended):** Use `--fetch-artifacts` flag to automatically copy artifacts to `./bundle/`:
```bash
bun scripts/docker/mac-build.ts --fetch-artifacts
```

After fetching, artifacts are in:
```
./bundle/
├── macos/
│   └── heycat.app
└── dmg/
    └── heycat_0.1.0_aarch64.dmg
```

**Manual:** If you didn't use `--fetch-artifacts`, artifacts are on the macOS host:
```
${HEYCAT_MAC_PATH}/src-tauri/target/release/bundle/
```

To copy manually:
```bash
rsync -avz ${HEYCAT_MAC_USER}@${HEYCAT_MAC_HOST}:${HEYCAT_MAC_PATH}/src-tauri/target/release/bundle/ ./bundle/
```

## What Gets Synced

| Included | Excluded |
|----------|----------|
| Source code | `target/` (Rust builds) |
| Configuration | `node_modules/` (dependencies) |
| Scripts | `.git/` (repository) |
| Assets | `dist/` (frontend build) |
| | `*.log` (log files) |
| | `.tcr-*` (TCR state) |
| | `coverage/` (test coverage) |

## Development Workflow

1. Make changes in Docker container
2. Run `/mac-build --dev` to start dev server on macOS
3. The macOS app will connect to the dev server
4. Changes are synced automatically on each build

## Notes

- SSH key authentication must be configured (no password prompts)
- The macOS host must be reachable from the Docker container
- First build may take longer due to dependency installation
- Subsequent builds are faster due to cached dependencies

## Troubleshooting

**"Cannot connect to macOS host"**
- Check if host is reachable: `ping ${HEYCAT_MAC_HOST}`
- Verify SSH works: `ssh ${HEYCAT_MAC_USER}@${HEYCAT_MAC_HOST} echo ok`
- Check SSH_AUTH_SOCK is forwarded to container

**"rsync: command not found"**
- Install rsync in the container (should be pre-installed)
- On macOS: rsync is included by default

**"bun: command not found" (on macOS)**
- Install Bun on macOS: `curl -fsSL https://bun.sh/install | bash`
- Add to PATH: `export PATH="$HOME/.bun/bin:$PATH"`

**"cargo: command not found" (on macOS)**
- Install Rust on macOS: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Restart shell or run: `source ~/.cargo/env`

**"Build failed - missing SDK"**
- On macOS, run: `xcode-select --install`
- Accept Xcode license: `sudo xcodebuild -license accept`

**"Sync is slow"**
- First sync transfers all files
- Subsequent syncs only transfer changes
- Consider using `--compress` option if network is slow

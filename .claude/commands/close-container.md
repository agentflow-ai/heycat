---
description: Stop and remove Docker container after PR is merged
---

# Close Docker Container After PR Merged

You are stopping and removing a Docker development container. This should be done after the PR has been merged.

## Prerequisites Check

1. Verify Docker is running:
   ```bash
   docker info >/dev/null 2>&1 && echo "Docker is running" || echo "Docker is not running"
   ```

2. List running heycat containers:
   ```bash
   docker ps --filter name=heycat-dev --format "table {{.Names}}\t{{.Status}}\t{{.CreatedAt}}"
   ```

## Execution Flow

### Step 1: Identify the container

Ask the user which container to close, or detect from context:
- If user provides a container ID: use that
- If running inside a container: use `HEYCAT_DEV_ID` environment variable
- Otherwise: list containers and ask user to select

### Step 2: Check PR status (recommended)

Access the container and check PR status:

```bash
docker exec heycat-dev-<id> bash -c "gh pr view --json state,mergedAt,url 2>/dev/null || echo 'No PR found'"
```

**If PR exists and is merged:**
- Proceed with deletion

**If PR exists but NOT merged:**
- Warn user: "PR is not yet merged. Deleting the container will make it harder to make changes."
- Ask for explicit confirmation before proceeding
- Suggest they wait until PR is merged

**If no PR exists:**
- Warn user: "No PR found for this branch."
- Ask if they want to proceed anyway (maybe changes were abandoned)

### Step 3: Check for uncommitted changes

```bash
docker exec heycat-dev-<id> git status --porcelain
```

If there are uncommitted changes:
- Warn user that changes will be lost
- Ask for confirmation before proceeding
- Suggest committing or pushing first

### Step 4: Close the container

```bash
bun scripts/docker/close-container.ts <container-id>
```

Or with volume cleanup:
```bash
bun scripts/docker/close-container.ts <container-id> --clean-volumes
```

Use `--force` flag to skip confirmation:
```bash
bun scripts/docker/close-container.ts <container-id> --force
```

This script will:
1. Stop the running container
2. Remove the container
3. Optionally remove associated volumes (with `--clean-volumes`)
4. Print success message

### Step 5: Verify cleanup

Check that container is removed:
```bash
docker ps -a --filter name=heycat-dev-<id>
```

Should return empty output.

## What Gets Deleted

| Resource | Deleted by default | With --clean-volumes |
|----------|-------------------|---------------------|
| Container | ✓ | ✓ |
| Bun cache volume | - | ✓ |
| Cargo registry volume | - | ✓ |
| Cargo git volume | - | ✓ |

## Volume Management

Volumes are named:
- `heycat-bun-cache-<id>`
- `heycat-cargo-registry-<id>`
- `heycat-cargo-git-<id>`

To list volumes for a container:
```bash
docker volume ls --filter name=heycat-*-<container-id>
```

To manually remove volumes:
```bash
docker volume rm heycat-bun-cache-<id> heycat-cargo-registry-<id> heycat-cargo-git-<id>
```

## Notes

- This is part of the "cattle" container model - containers are ephemeral
- The branch is NOT deleted from git - it stays as part of the merged PR
- Volumes are preserved by default so you can recreate the container quickly
- Use `--clean-volumes` when you're sure you won't need the cached dependencies

## Troubleshooting

**"Container does not exist"**
- Container may have already been removed
- List all containers: `docker ps -a --filter name=heycat-dev`

**"Failed to stop container"**
- Container may be stuck. Force remove: `docker rm -f heycat-dev-<id>`

**"Volumes in use"**
- Remove the container first, then remove volumes
- Or use: `docker volume rm -f <volume-name>`

**"Permission denied"**
- You may need sudo for docker commands
- Or add your user to the docker group

## Quick Cleanup

To remove ALL heycat development containers and volumes:
```bash
# Remove all heycat containers
docker ps -a --filter name=heycat-dev -q | xargs -r docker rm -f

# Remove all heycat volumes
docker volume ls --filter name=heycat- -q | xargs -r docker volume rm
```

**Warning**: This removes ALL heycat development resources, not just one container.

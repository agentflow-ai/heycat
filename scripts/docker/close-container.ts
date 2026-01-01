#!/usr/bin/env bun
/**
 * Stop and remove a Docker development container.
 *
 * This script mirrors close-worktree.ts logic but for Docker containers:
 * 1. Warns if uncommitted changes exist in the container
 * 2. Stops and removes the container
 * 3. Optionally cleans up named volumes
 *
 * Usage:
 *   bun scripts/docker/close-container.ts <container-id>        # Close specific container
 *   bun scripts/docker/close-container.ts --clean-volumes       # Also remove volumes
 *   bun scripts/docker/close-container.ts --force               # Skip confirmation
 *   bun scripts/docker/close-container.ts --help                # Show help
 *
 * Can also detect container from HEYCAT_DEV_ID environment variable.
 */

import { createInterface } from "readline";
import { colors, log, success, error, info, warn } from "../lib/utils";

/**
 * Get the full container name from a dev ID.
 */
export function getContainerName(devId: string): string {
  return `heycat-dev-${devId}`;
}

/**
 * Get the volume names associated with a dev ID.
 */
export function getVolumeNames(devId: string): string[] {
  return [
    `heycat-bun-cache-${devId}`,
    `heycat-cargo-registry-${devId}`,
    `heycat-cargo-git-${devId}`,
  ];
}

interface Flags {
  force: boolean;
  cleanVolumes: boolean;
  help: boolean;
  containerId: string | null;
}

/**
 * Parse command line arguments.
 */
export function parseArgs(args: string[]): Flags {
  const flags: Flags = {
    force: false,
    cleanVolumes: false,
    help: false,
    containerId: null,
  };

  for (const arg of args) {
    if (arg === "--force" || arg === "-f") {
      flags.force = true;
    } else if (arg === "--clean-volumes" || arg === "--volumes") {
      flags.cleanVolumes = true;
    } else if (arg === "--help" || arg === "-h") {
      flags.help = true;
    } else if (!arg.startsWith("-")) {
      flags.containerId = arg;
    }
  }

  return flags;
}

/**
 * Print help message.
 */
function printHelp(): void {
  log(`
${colors.bold}Usage:${colors.reset} bun scripts/docker/close-container.ts [container-id] [options]

${colors.bold}Description:${colors.reset}
  Stops and removes a Docker development container.
  Can detect container from HEYCAT_DEV_ID environment variable if no ID provided.

${colors.bold}Arguments:${colors.reset}
  container-id    Container ID (e.g., "feature-audio" or full name "heycat-dev-feature-audio")

${colors.bold}Options:${colors.reset}
  --force, -f        Skip confirmation prompt
  --clean-volumes    Also remove associated Docker volumes
  --help, -h         Show this help message

${colors.bold}What gets cleaned up:${colors.reset}
  - Docker container (stopped and removed)
  - Docker volumes (with --clean-volumes):
    - heycat-bun-cache-{id}
    - heycat-cargo-registry-{id}
    - heycat-cargo-git-{id}

${colors.bold}Examples:${colors.reset}
  ${colors.cyan}bun scripts/docker/close-container.ts feature-audio${colors.reset}
    Close container with confirmation

  ${colors.cyan}bun scripts/docker/close-container.ts feature-audio --force${colors.reset}
    Close container without confirmation

  ${colors.cyan}bun scripts/docker/close-container.ts feature-audio --clean-volumes${colors.reset}
    Close container and remove all associated volumes

${colors.bold}Note:${colors.reset}
  This is part of the "cattle" container workflow. Run this after your PR is merged.
`);
}

/**
 * Prompt user for confirmation via readline.
 */
async function confirm(message: string): Promise<boolean> {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((res) => {
    rl.question(`${message} [y/N]: `, (answer) => {
      rl.close();
      res(answer.toLowerCase() === "y" || answer.toLowerCase() === "yes");
    });
  });
}

/**
 * Check if a container exists (running or stopped).
 */
async function containerExists(containerName: string): Promise<boolean> {
  const result = await Bun.spawn(
    ["docker", "ps", "-a", "--filter", `name=^${containerName}$`, "--format", "{{.Names}}"],
    { stdout: "pipe", stderr: "ignore" }
  );
  const output = await new Response(result.stdout).text();
  return output.trim() === containerName;
}

/**
 * Check if a container is running.
 */
async function containerRunning(containerName: string): Promise<boolean> {
  const result = await Bun.spawn(
    ["docker", "ps", "--filter", `name=^${containerName}$`, "--format", "{{.Names}}"],
    { stdout: "pipe", stderr: "ignore" }
  );
  const output = await new Response(result.stdout).text();
  return output.trim() === containerName;
}

/**
 * Check for uncommitted changes in the container.
 */
async function hasUncommittedChanges(containerName: string): Promise<boolean> {
  const result = await Bun.spawn(
    ["docker", "exec", containerName, "git", "status", "--porcelain"],
    { stdout: "pipe", stderr: "ignore" }
  );
  const output = await new Response(result.stdout).text();
  return output.trim().length > 0;
}

/**
 * Stop a running container.
 */
async function stopContainer(containerName: string): Promise<boolean> {
  const result = await Bun.spawn(["docker", "stop", containerName], {
    stdout: "inherit",
    stderr: "inherit",
  }).exited;
  return result === 0;
}

/**
 * Remove a container.
 */
async function removeContainer(containerName: string): Promise<boolean> {
  const result = await Bun.spawn(["docker", "rm", containerName], {
    stdout: "inherit",
    stderr: "inherit",
  }).exited;
  return result === 0;
}

/**
 * Remove Docker volumes associated with a container ID.
 */
async function removeVolumes(devId: string): Promise<{ removed: string[]; failed: string[] }> {
  const volumeNames = getVolumeNames(devId);

  const removed: string[] = [];
  const failed: string[] = [];

  for (const volumeName of volumeNames) {
    // Check if volume exists
    const checkResult = await Bun.spawn(["docker", "volume", "inspect", volumeName], {
      stdout: "ignore",
      stderr: "ignore",
    }).exited;

    if (checkResult !== 0) {
      // Volume doesn't exist, skip
      continue;
    }

    // Remove volume
    const removeResult = await Bun.spawn(["docker", "volume", "rm", volumeName], {
      stdout: "pipe",
      stderr: "pipe",
    }).exited;

    if (removeResult === 0) {
      removed.push(volumeName);
    } else {
      failed.push(volumeName);
    }
  }

  return { removed, failed };
}

/**
 * Detect container ID from environment or argument.
 */
function detectContainerId(flags: Flags): string | null {
  // Argument takes precedence
  if (flags.containerId) {
    // If it already starts with heycat-dev-, extract the ID
    if (flags.containerId.startsWith("heycat-dev-")) {
      return flags.containerId.replace("heycat-dev-", "");
    }
    return flags.containerId;
  }

  // Fall back to environment variable
  const envId = process.env.HEYCAT_DEV_ID;
  if (envId && envId !== "default") {
    return envId;
  }

  return null;
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const flags = parseArgs(args);

  if (flags.help) {
    printHelp();
    process.exit(0);
  }

  // Detect container ID
  const devId = detectContainerId(flags);
  if (!devId) {
    error("No container ID provided.");
    log("\nUsage: bun scripts/docker/close-container.ts <container-id>");
    log("\nOr set HEYCAT_DEV_ID environment variable.");
    log("\nTo list running containers:");
    log("  docker ps --filter name=heycat-dev");
    process.exit(1);
  }

  const containerName = `heycat-dev-${devId}`;

  log(`\n${colors.bold}Closing Docker container${colors.reset}\n`);
  info(`Container ID: ${devId}`);
  info(`Container name: ${containerName}`);

  // Check if container exists
  if (!(await containerExists(containerName))) {
    error(`Container '${containerName}' does not exist.`);
    log("\nTo list all containers:");
    log("  docker ps -a --filter name=heycat-dev");
    process.exit(1);
  }

  // Check for uncommitted changes if container is running
  if (await containerRunning(containerName)) {
    const hasChanges = await hasUncommittedChanges(containerName);
    if (hasChanges) {
      warn("\nWarning: Container has uncommitted changes!");
      log("Consider committing or pushing changes before closing.");

      if (!flags.force) {
        const confirmed = await confirm("Proceed anyway?");
        if (!confirmed) {
          info("\nCancelled.");
          process.exit(0);
        }
      }
    }
  }

  // Confirm deletion unless --force
  if (!flags.force) {
    log("");
    const volumeNote = flags.cleanVolumes ? " and all associated volumes" : "";
    const confirmed = await confirm(`Delete container '${containerName}'${volumeNote}?`);
    if (!confirmed) {
      info("\nCancelled.");
      process.exit(0);
    }
  }

  // Stop container if running
  if (await containerRunning(containerName)) {
    log(`\n${colors.bold}Stopping container...${colors.reset}`);
    if (!(await stopContainer(containerName))) {
      error("Failed to stop container.");
      process.exit(1);
    }
    success("  Container stopped");
  }

  // Remove container
  log(`\n${colors.bold}Removing container...${colors.reset}`);
  if (!(await removeContainer(containerName))) {
    error("Failed to remove container.");
    process.exit(1);
  }
  success("  Container removed");

  // Optionally remove volumes
  if (flags.cleanVolumes) {
    log(`\n${colors.bold}Removing volumes...${colors.reset}`);
    const { removed, failed } = await removeVolumes(devId);

    for (const vol of removed) {
      success(`  Removed: ${vol}`);
    }

    if (failed.length > 0) {
      for (const vol of failed) {
        warn(`  Failed to remove: ${vol}`);
      }
    }

    if (removed.length === 0 && failed.length === 0) {
      info("  No volumes found to remove");
    }
  }

  // Print success message
  log(`
${colors.green}${colors.bold}Container closed successfully!${colors.reset}

${colors.bold}To create a new container:${colors.reset}
  ${colors.cyan}bun scripts/docker/create-container.ts <branch-name>${colors.reset}
`);
}

// Only run main when executed directly, not when imported as a module
if (import.meta.main) {
  main().catch((err) => {
    error(err.message || String(err));
    process.exit(1);
  });
}

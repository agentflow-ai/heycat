#!/usr/bin/env bun
/**
 * Trigger Tauri builds on macOS host from Docker container.
 *
 * This script syncs the workspace to a macOS host via rsync and
 * triggers the Tauri build process. This is necessary because
 * Tauri/Swift builds require macOS.
 *
 * Usage:
 *   bun scripts/docker/mac-build.ts              # Sync and build
 *   bun scripts/docker/mac-build.ts --sync-only  # Only sync, don't build
 *   bun scripts/docker/mac-build.ts --dev        # Run tauri dev instead of build
 *   bun scripts/docker/mac-build.ts --help       # Show help
 *
 * Required environment variables:
 *   HEYCAT_MAC_HOST  - macOS host (e.g., 192.168.1.100 or mac.local)
 *   HEYCAT_MAC_USER  - SSH username on macOS host
 *   HEYCAT_MAC_PATH  - Path on macOS host for project (e.g., ~/heycat-docker)
 *
 * Prerequisites:
 *   - SSH key authentication configured
 *   - rsync installed in container and on host
 *   - Bun and Rust toolchain installed on macOS host
 */

// ANSI color codes for terminal output
const colors = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  cyan: "\x1b[36m",
  dim: "\x1b[2m",
};

function log(message: string): void {
  console.log(message);
}

function success(message: string): void {
  console.log(`${colors.green}${colors.bold}${message}${colors.reset}`);
}

function error(message: string): void {
  console.error(`${colors.red}${colors.bold}Error: ${message}${colors.reset}`);
}

function info(message: string): void {
  console.log(`${colors.cyan}${message}${colors.reset}`);
}

function warn(message: string): void {
  console.log(`${colors.yellow}${message}${colors.reset}`);
}

interface MacBuildConfig {
  host: string;
  user: string;
  path: string;
}

interface Flags {
  syncOnly: boolean;
  dev: boolean;
  help: boolean;
}

/**
 * Parse command line arguments.
 */
export function parseArgs(args: string[]): Flags {
  const flags: Flags = {
    syncOnly: false,
    dev: false,
    help: false,
  };

  for (const arg of args) {
    if (arg === "--sync-only" || arg === "--sync") {
      flags.syncOnly = true;
    } else if (arg === "--dev") {
      flags.dev = true;
    } else if (arg === "--help" || arg === "-h") {
      flags.help = true;
    }
  }

  return flags;
}

/**
 * Print help message.
 */
function printHelp(): void {
  log(`
${colors.bold}Usage:${colors.reset} bun scripts/docker/mac-build.ts [options]

${colors.bold}Description:${colors.reset}
  Syncs workspace to macOS host and triggers Tauri build.
  This is necessary because Tauri/Swift requires macOS.

${colors.bold}Options:${colors.reset}
  --sync-only    Only sync files, don't run build
  --dev          Run 'tauri dev' instead of 'tauri build'
  --help, -h     Show this help message

${colors.bold}Environment Variables:${colors.reset}
  HEYCAT_MAC_HOST    macOS host (default: host.docker.internal)
  HEYCAT_MAC_USER    SSH username on macOS host (required)
  HEYCAT_MAC_PATH    Project path on macOS host (required)

${colors.bold}Setup:${colors.reset}
  1. Configure SSH key authentication to macOS host
  2. Set environment variables in .env or export them
  3. Run this script from the Docker container

${colors.bold}Examples:${colors.reset}
  ${colors.cyan}bun scripts/docker/mac-build.ts${colors.reset}
    Sync and run full release build

  ${colors.cyan}bun scripts/docker/mac-build.ts --dev${colors.reset}
    Sync and start development server

  ${colors.cyan}bun scripts/docker/mac-build.ts --sync-only${colors.reset}
    Only sync files without building

${colors.bold}Excluded from sync:${colors.reset}
  - target/         (Rust build artifacts)
  - node_modules/   (npm/bun dependencies)
  - .git/           (git repository data)
  - dist/           (frontend build output)
  - *.log           (log files)
`);
}

/**
 * Get macOS build configuration from environment variables.
 * Defaults HEYCAT_MAC_HOST to host.docker.internal for Docker Desktop compatibility.
 */
export function getConfig(): MacBuildConfig | null {
  const host = process.env.HEYCAT_MAC_HOST || "host.docker.internal";
  const user = process.env.HEYCAT_MAC_USER;
  const path = process.env.HEYCAT_MAC_PATH;

  if (!user || !path) {
    return null;
  }

  return { host, user, path };
}

/**
 * Validate macOS host connectivity.
 */
async function checkConnection(config: MacBuildConfig): Promise<boolean> {
  const sshTarget = `${config.user}@${config.host}`;

  info(`Checking SSH connection to ${sshTarget}...`);

  const result = await Bun.spawn(
    ["ssh", "-o", "ConnectTimeout=5", "-o", "BatchMode=yes", sshTarget, "echo", "ok"],
    {
      stdout: "pipe",
      stderr: "pipe",
    }
  );

  const output = await new Response(result.stdout).text();
  const exitCode = await result.exited;

  return exitCode === 0 && output.trim() === "ok";
}

/**
 * Sync workspace to macOS host using rsync.
 */
async function syncToMac(config: MacBuildConfig): Promise<boolean> {
  const sshTarget = `${config.user}@${config.host}:${config.path}/`;

  log(`\n${colors.bold}Syncing workspace to macOS host...${colors.reset}`);
  info(`Target: ${sshTarget}`);

  // rsync options:
  // -a: archive mode (preserves permissions, etc.)
  // -v: verbose
  // -z: compress during transfer
  // --delete: delete files on destination that don't exist on source
  // --exclude: patterns to exclude
  const rsyncArgs = [
    "-avz",
    "--delete",
    "--exclude=target/",
    "--exclude=node_modules/",
    "--exclude=.git/",
    "--exclude=dist/",
    "--exclude=*.log",
    "--exclude=.tcr-state.json",
    "--exclude=.tcr-errors.log",
    "--exclude=.tcr/",
    "--exclude=coverage/",
    "./",
    sshTarget,
  ];

  const result = await Bun.spawn(["rsync", ...rsyncArgs], {
    stdout: "inherit",
    stderr: "inherit",
  }).exited;

  return result === 0;
}

/**
 * Run build command on macOS host.
 */
async function runBuildOnMac(config: MacBuildConfig, isDev: boolean): Promise<boolean> {
  const sshTarget = `${config.user}@${config.host}`;
  const buildCmd = isDev ? "bun run tauri dev" : "bun run tauri build";

  log(`\n${colors.bold}Running build on macOS host...${colors.reset}`);
  info(`Command: ${buildCmd}`);

  // Build the remote command - install deps and run build
  const remoteCommand = `cd ${config.path} && bun install && ${buildCmd}`;

  const result = await Bun.spawn(
    ["ssh", "-t", sshTarget, remoteCommand],
    {
      stdout: "inherit",
      stderr: "inherit",
      stdin: "inherit",
    }
  ).exited;

  return result === 0;
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const flags = parseArgs(args);

  if (flags.help) {
    printHelp();
    process.exit(0);
  }

  log(`\n${colors.bold}macOS Build${colors.reset}\n`);

  // Get configuration
  const config = getConfig();
  if (!config) {
    error("Missing required environment variables.");
    log("\nRequired variables:");
    log("  HEYCAT_MAC_USER - SSH username on macOS host");
    log("  HEYCAT_MAC_PATH - Project path on macOS host");
    log("\nOptional (defaults to host.docker.internal):");
    log("  HEYCAT_MAC_HOST - macOS host (IP or hostname)");
    log("\nSet these in your .env file or export them before running.");
    process.exit(1);
  }

  info(`Host: ${config.host}`);
  info(`User: ${config.user}`);
  info(`Path: ${config.path}`);

  // Check SSH connection
  if (!(await checkConnection(config))) {
    error("Cannot connect to macOS host.");
    log("\nTroubleshooting:");
    log("  1. Ensure SSH key authentication is configured");
    log("  2. Verify the host is reachable from the container");
    log("  3. Check SSH_AUTH_SOCK is forwarded correctly");
    log("\nTest manually:");
    log(`  ssh ${config.user}@${config.host} echo "ok"`);
    process.exit(1);
  }
  success("SSH connection OK");

  // Sync workspace
  if (!(await syncToMac(config))) {
    error("Failed to sync workspace to macOS host.");
    process.exit(1);
  }
  success("\nWorkspace synced successfully");

  // Run build unless --sync-only
  if (flags.syncOnly) {
    log(`
${colors.green}${colors.bold}Sync complete!${colors.reset}

To build manually on macOS host:
  ${colors.cyan}ssh ${config.user}@${config.host}${colors.reset}
  ${colors.cyan}cd ${config.path}${colors.reset}
  ${colors.cyan}bun install && bun run tauri build${colors.reset}
`);
    process.exit(0);
  }

  // Run the build
  if (!(await runBuildOnMac(config, flags.dev))) {
    error("Build failed on macOS host.");
    process.exit(1);
  }

  const buildType = flags.dev ? "Development server" : "Build";
  success(`\n${buildType} completed successfully!`);

  if (!flags.dev) {
    log(`
${colors.bold}Build artifacts location:${colors.reset}
  ${colors.cyan}${config.path}/src-tauri/target/release/bundle/${colors.reset}
`);
  }
}

// Only run main when executed directly, not when imported as a module
if (import.meta.main) {
  main().catch((err) => {
    error(err.message || String(err));
    process.exit(1);
  });
}

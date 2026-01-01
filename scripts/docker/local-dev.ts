#!/usr/bin/env bun
/**
 * Local Docker development workflow for macOS with Docker Desktop.
 *
 * This script provides a streamlined workflow when running Docker locally.
 * Since the source code is bind-mounted, no SSH or rsync is needed -
 * files are automatically shared between container and host.
 *
 * Usage:
 *   bun scripts/docker/local-dev.ts           # Start container + show instructions
 *   bun scripts/docker/local-dev.ts --shell   # Start container + exec into shell
 *   bun scripts/docker/local-dev.ts --build   # Build Tauri app on host
 *   bun scripts/docker/local-dev.ts --dev     # Start Tauri dev server on host
 *   bun scripts/docker/local-dev.ts --stop    # Stop container
 *   bun scripts/docker/local-dev.ts --help    # Show help
 *
 * For remote Docker hosts, use mac-build.ts instead (requires SSH).
 */

import { colors, log, success, error, info } from "../lib/utils";

const CONTAINER_NAME = "heycat-dev-default";

interface Flags {
  shell: boolean;
  build: boolean;
  dev: boolean;
  stop: boolean;
  help: boolean;
}

/**
 * Parse command line arguments.
 */
export function parseArgs(args: string[]): Flags {
  const flags: Flags = {
    shell: false,
    build: false,
    dev: false,
    stop: false,
    help: false,
  };

  for (const arg of args) {
    if (arg === "--shell" || arg === "-s") {
      flags.shell = true;
    } else if (arg === "--build" || arg === "-b") {
      flags.build = true;
    } else if (arg === "--dev" || arg === "-d") {
      flags.dev = true;
    } else if (arg === "--stop") {
      flags.stop = true;
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
${colors.bold}Usage:${colors.reset} bun scripts/docker/local-dev.ts [options]

${colors.bold}Description:${colors.reset}
  Local Docker development workflow for macOS with Docker Desktop.
  Files are bind-mounted, so no SSH or sync is needed.

${colors.bold}Options:${colors.reset}
  --shell, -s     Start container and exec into bash shell
  --build, -b     Run Tauri release build on host
  --dev, -d       Run Tauri dev server on host
  --stop          Stop the running container
  --help, -h      Show this help message

${colors.bold}Examples:${colors.reset}
  ${colors.cyan}bun scripts/docker/local-dev.ts --shell${colors.reset}
    Start container and open interactive shell

  ${colors.cyan}bun scripts/docker/local-dev.ts --dev${colors.reset}
    Start Tauri development server (hot reload)

  ${colors.cyan}bun scripts/docker/local-dev.ts --build${colors.reset}
    Build release version of the app

${colors.bold}Workflow:${colors.reset}
  1. Run tests/linting in container: ${colors.cyan}--shell${colors.reset}
  2. Build for macOS on host: ${colors.cyan}--build${colors.reset} or ${colors.cyan}--dev${colors.reset}

${colors.bold}Note:${colors.reset}
  For remote Docker hosts, use ${colors.cyan}mac-build.ts${colors.reset} instead.
`);
}

/**
 * Check if Docker is available.
 */
async function checkDocker(): Promise<boolean> {
  const result = await Bun.spawn(["docker", "info"], {
    stdout: "ignore",
    stderr: "ignore",
  }).exited;
  return result === 0;
}

/**
 * Check if container is running.
 */
export async function isContainerRunning(): Promise<boolean> {
  const result = await Bun.spawn(
    ["docker", "ps", "--filter", `name=${CONTAINER_NAME}`, "--format", "{{.Names}}"],
    { stdout: "pipe", stderr: "ignore" }
  );
  const output = await new Response(result.stdout).text();
  return output.trim() === CONTAINER_NAME;
}

/**
 * Start the container using docker compose.
 */
async function startContainer(): Promise<boolean> {
  info("Starting container...");

  const result = await Bun.spawn(["docker", "compose", "up", "-d"], {
    stdout: "inherit",
    stderr: "inherit",
  }).exited;

  return result === 0;
}

/**
 * Stop the container.
 */
async function stopContainer(): Promise<boolean> {
  info("Stopping container...");

  const result = await Bun.spawn(["docker", "compose", "down"], {
    stdout: "inherit",
    stderr: "inherit",
  }).exited;

  return result === 0;
}

/**
 * Exec into the container shell.
 */
async function execShell(): Promise<void> {
  log(`\n${colors.bold}Entering container shell...${colors.reset}`);
  log(`${colors.dim}(Type 'exit' to leave)${colors.reset}\n`);

  await Bun.spawn(["docker", "exec", "-it", CONTAINER_NAME, "bash"], {
    stdout: "inherit",
    stderr: "inherit",
    stdin: "inherit",
  }).exited;
}

/**
 * Run Tauri build on host.
 */
async function runTauriBuild(): Promise<boolean> {
  log(`\n${colors.bold}Running Tauri build on host...${colors.reset}`);
  info("This builds the macOS app using bind-mounted source files.\n");

  const result = await Bun.spawn(["bun", "run", "tauri", "build"], {
    stdout: "inherit",
    stderr: "inherit",
  }).exited;

  return result === 0;
}

/**
 * Run Tauri dev server on host.
 */
async function runTauriDev(): Promise<boolean> {
  log(`\n${colors.bold}Starting Tauri dev server on host...${colors.reset}`);
  info("Hot reload enabled. Press Ctrl+C to stop.\n");

  const result = await Bun.spawn(["bun", "run", "tauri", "dev"], {
    stdout: "inherit",
    stderr: "inherit",
    stdin: "inherit",
  }).exited;

  return result === 0;
}

/**
 * Print quick reference when no options specified.
 */
function printQuickReference(): void {
  log(`
${colors.bold}${colors.green}Container is running!${colors.reset}

${colors.bold}Quick Reference:${colors.reset}

  ${colors.cyan}In container (tests, linting):${colors.reset}
    bun scripts/docker/local-dev.ts --shell
    ${colors.dim}Or: docker exec -it ${CONTAINER_NAME} bash${colors.reset}

  ${colors.cyan}On host (Tauri/macOS builds):${colors.reset}
    bun scripts/docker/local-dev.ts --dev     ${colors.dim}# Dev server${colors.reset}
    bun scripts/docker/local-dev.ts --build   ${colors.dim}# Release build${colors.reset}

  ${colors.cyan}Stop container:${colors.reset}
    bun scripts/docker/local-dev.ts --stop

${colors.dim}Files are bind-mounted - changes sync automatically.${colors.reset}
`);
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const flags = parseArgs(args);

  if (flags.help) {
    printHelp();
    process.exit(0);
  }

  // Check Docker is available
  if (!(await checkDocker())) {
    error("Docker is not running. Please start Docker Desktop.");
    process.exit(1);
  }

  // Handle --stop
  if (flags.stop) {
    if (await stopContainer()) {
      success("Container stopped.");
    } else {
      error("Failed to stop container.");
      process.exit(1);
    }
    return;
  }

  // Handle --build (runs on host, no container needed)
  if (flags.build) {
    if (await runTauriBuild()) {
      success("\nBuild completed successfully!");
      log(`\nArtifacts: ${colors.cyan}src-tauri/target/release/bundle/${colors.reset}`);
    } else {
      error("Build failed.");
      process.exit(1);
    }
    return;
  }

  // Handle --dev (runs on host, no container needed)
  if (flags.dev) {
    await runTauriDev();
    return;
  }

  // For --shell or default: ensure container is running
  const running = await isContainerRunning();

  if (!running) {
    if (!(await startContainer())) {
      error("Failed to start container.");
      process.exit(1);
    }
  }

  // Handle --shell
  if (flags.shell) {
    await execShell();
    return;
  }

  // Default: show quick reference
  printQuickReference();
}

// Only run main when executed directly
if (import.meta.main) {
  main().catch((err) => {
    error(err.message || String(err));
    process.exit(1);
  });
}

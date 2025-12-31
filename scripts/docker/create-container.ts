#!/usr/bin/env bun
/**
 * Create a new Docker development container for heycat.
 *
 * This script mirrors create-worktree.ts logic but for Docker containers:
 * 1. Starts a container via docker-compose with HEYCAT_DEV_ID set
 * 2. Creates a feature branch inside the container
 * 3. Runs bun install
 * 4. Outputs container access instructions
 *
 * Usage: bun scripts/docker/create-container.ts <branch-name>
 *
 * Supports Linear issue branch naming (e.g., HEY-123-add-feature)
 */

import { existsSync } from "fs";
import { resolve } from "path";

// ANSI color codes for terminal output
const colors = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  cyan: "\x1b[36m",
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

/**
 * Check if Docker is available and running.
 */
async function checkDocker(): Promise<boolean> {
  const result = await Bun.spawn(["docker", "info"], {
    stdout: "ignore",
    stderr: "ignore",
  }).exited;
  return result === 0;
}

/**
 * Check if a git branch already exists locally or remotely.
 */
async function branchExists(branchName: string): Promise<boolean> {
  // Check local branches
  const localResult = await Bun.spawn(
    ["git", "show-ref", "--verify", "--quiet", `refs/heads/${branchName}`],
    { stdout: "ignore", stderr: "ignore" }
  ).exited;

  if (localResult === 0) return true;

  // Check remote branches
  const remoteResult = await Bun.spawn(
    ["git", "show-ref", "--verify", "--quiet", `refs/remotes/origin/${branchName}`],
    { stdout: "ignore", stderr: "ignore" }
  ).exited;

  return remoteResult === 0;
}

/**
 * Check if a container with the given name already exists.
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
 * Get the project root directory (where docker-compose.yml is).
 */
function getProjectRoot(): string {
  // This script is in scripts/docker/, so go up two levels
  return resolve(import.meta.dir, "../..");
}

/**
 * Convert branch name to a valid container ID.
 * Removes special characters and limits length.
 */
function branchToContainerId(branchName: string): string {
  // Replace special characters with dashes, limit to 32 chars
  // Order matters: truncate first, then remove leading/trailing dashes
  return branchName
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, "-")
    .replace(/-+/g, "-")
    .slice(0, 32)
    .replace(/^-|-$/g, "");
}

/**
 * Start the Docker container with docker-compose.
 */
async function startContainer(devId: string, projectRoot: string): Promise<boolean> {
  const containerName = `heycat-dev-${devId}`;

  info(`\nStarting container: ${containerName}`);

  // Get current user ID for permissions
  const uidResult = await Bun.spawn(["id", "-u"], { stdout: "pipe" });
  const uid = (await new Response(uidResult.stdout).text()).trim();

  const gidResult = await Bun.spawn(["id", "-g"], { stdout: "pipe" });
  const gid = (await new Response(gidResult.stdout).text()).trim();

  // Build the container first if needed
  const buildResult = await Bun.spawn(
    ["docker", "compose", "build", "--build-arg", `USER_ID=${uid}`, "--build-arg", `GROUP_ID=${gid}`],
    {
      cwd: projectRoot,
      stdout: "inherit",
      stderr: "inherit",
      env: {
        ...process.env,
        HEYCAT_DEV_ID: devId,
        USER_ID: uid,
        GROUP_ID: gid,
      },
    }
  ).exited;

  if (buildResult !== 0) {
    error("Failed to build Docker image");
    return false;
  }

  // Start the container in detached mode
  const runResult = await Bun.spawn(
    [
      "docker",
      "compose",
      "run",
      "-d",
      "--name",
      containerName,
      "--rm",
      "dev",
      "tail",
      "-f",
      "/dev/null", // Keep container running
    ],
    {
      cwd: projectRoot,
      stdout: "inherit",
      stderr: "inherit",
      env: {
        ...process.env,
        HEYCAT_DEV_ID: devId,
        USER_ID: uid,
        GROUP_ID: gid,
      },
    }
  ).exited;

  return runResult === 0;
}

/**
 * Execute a command inside the running container.
 */
async function execInContainer(
  containerName: string,
  command: string[],
  options: { inherit?: boolean } = {}
): Promise<{ success: boolean; output?: string }> {
  const result = await Bun.spawn(["docker", "exec", containerName, ...command], {
    stdout: options.inherit ? "inherit" : "pipe",
    stderr: options.inherit ? "inherit" : "pipe",
  });

  const exitCode = await result.exited;

  if (!options.inherit) {
    const output = await new Response(result.stdout).text();
    return { success: exitCode === 0, output: output.trim() };
  }

  return { success: exitCode === 0 };
}

/**
 * Create feature branch inside the container.
 */
async function createBranchInContainer(containerName: string, branchName: string): Promise<boolean> {
  info(`\nCreating branch: ${branchName}`);

  // Configure git user if not set (needed for commits)
  await execInContainer(containerName, ["git", "config", "--global", "user.name", "Docker Dev"]);
  await execInContainer(containerName, ["git", "config", "--global", "user.email", "dev@heycat.local"]);

  // Create and checkout the branch
  const result = await execInContainer(containerName, ["git", "checkout", "-b", branchName], { inherit: true });

  return result.success;
}

/**
 * Run bun install inside the container.
 */
async function installDependencies(containerName: string): Promise<boolean> {
  info(`\nInstalling dependencies...`);

  const result = await execInContainer(containerName, ["bun", "install"], { inherit: true });

  return result.success;
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);

  if (args.length === 0 || args[0] === "--help" || args[0] === "-h") {
    log(`
${colors.bold}Usage:${colors.reset} bun scripts/docker/create-container.ts <branch-name>

${colors.bold}Arguments:${colors.reset}
  branch-name  Name for the git branch (required)
               Supports Linear issue format: HEY-123-description

${colors.bold}Description:${colors.reset}
  Creates a new Docker development container:
  - Starts container via docker-compose
  - Creates feature branch inside container
  - Runs bun install

${colors.bold}Example:${colors.reset}
  bun scripts/docker/create-container.ts feature-audio-improvements
  bun scripts/docker/create-container.ts HEY-123-add-dark-mode
`);
    process.exit(0);
  }

  // Check Docker is available
  if (!(await checkDocker())) {
    error("Docker is not running. Please start Docker and try again.");
    process.exit(1);
  }

  const branchName = args[0];
  const devId = branchToContainerId(branchName);
  const containerName = `heycat-dev-${devId}`;
  const projectRoot = getProjectRoot();

  log(`\n${colors.bold}Creating heycat Docker container${colors.reset}\n`);
  info(`Branch: ${branchName}`);
  info(`Container ID: ${devId}`);
  info(`Container name: ${containerName}`);

  // Check if docker-compose.yml exists
  if (!existsSync(resolve(projectRoot, "docker-compose.yml"))) {
    error("docker-compose.yml not found in project root.");
    error("Please run this script from the heycat repository.");
    process.exit(1);
  }

  // Check if branch already exists
  if (await branchExists(branchName)) {
    error(`Branch '${branchName}' already exists.`);
    process.exit(1);
  }

  // Check if container already exists
  if (await containerExists(containerName)) {
    error(`Container '${containerName}' already exists.`);
    error("To remove it: docker rm -f " + containerName);
    process.exit(1);
  }

  // Start the container
  if (!(await startContainer(devId, projectRoot))) {
    error("Failed to start container.");
    process.exit(1);
  }

  success(`\nContainer started: ${containerName}`);

  // Create branch inside container
  if (!(await createBranchInContainer(containerName, branchName))) {
    error("Failed to create branch.");
    // Clean up container
    await Bun.spawn(["docker", "rm", "-f", containerName], { stdout: "ignore", stderr: "ignore" });
    process.exit(1);
  }

  // Install dependencies
  if (!(await installDependencies(containerName))) {
    error("Failed to install dependencies.");
    // Clean up container
    await Bun.spawn(["docker", "rm", "-f", containerName], { stdout: "ignore", stderr: "ignore" });
    process.exit(1);
  }

  // Print success message and instructions
  log(`
${colors.bold}${colors.green}Container created successfully!${colors.reset}

${colors.bold}Access the container:${colors.reset}
  ${colors.cyan}docker exec -it ${containerName} bash${colors.reset}

${colors.bold}Run commands in container:${colors.reset}
  ${colors.cyan}docker exec ${containerName} bun run test${colors.reset}
  ${colors.cyan}docker exec ${containerName} cargo test${colors.reset}

${colors.bold}Start Claude Code in container:${colors.reset}
  ${colors.cyan}docker exec -it ${containerName} claude${colors.reset}

${colors.bold}For macOS builds (Tauri/Swift):${colors.reset}
  Run the mac-build script to sync code and build on host:
  ${colors.cyan}bun scripts/docker/mac-build.ts${colors.reset}

${colors.bold}Stop and remove container:${colors.reset}
  ${colors.cyan}docker rm -f ${containerName}${colors.reset}
  Or use: ${colors.cyan}bun scripts/docker/close-container.ts ${devId}${colors.reset}
`);
}

main().catch((err) => {
  error(err.message || String(err));
  process.exit(1);
});

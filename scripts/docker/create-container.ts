#!/usr/bin/env bun
/**
 * Create a new Docker development container for heycat.
 *
 * This script mirrors create-worktree.ts logic but for Docker containers:
 * 1. Validates the issue exists in Linear and gets its HEY-### identifier
 * 2. Starts a container via docker-compose with HEYCAT_DEV_ID set
 * 3. Creates a feature branch with format: HEY-###-<issue-slug>
 * 4. Runs bun install
 * 5. Outputs container access instructions
 *
 * Usage: bun scripts/docker/create-container.ts --issue <issue-slug>
 *
 * All development must go through Linear - no freeform branch names allowed.
 */

import { existsSync } from "fs";
import { resolve } from "path";
import { validateLinearIssue } from "../lib/linear";
import { colors, log, success, error, info, warn } from "../lib/utils";

/**
 * Parse command line arguments.
 */
function parseArgs(args: string[]): { issue: string | null; help: boolean } {
  let issue: string | null = null;
  let help = false;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--help" || arg === "-h") {
      help = true;
    } else if (arg === "--issue" || arg === "-i") {
      issue = args[++i] || null;
    } else if (!arg.startsWith("-") && !issue) {
      // Support legacy positional argument (but issue is still required)
      issue = arg;
    }
  }

  return { issue, help };
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
export function branchToContainerId(branchName: string): string {
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
  // Use environment variables if available, otherwise fall back to defaults
  const gitName = process.env.GIT_AUTHOR_NAME || "Docker Dev";
  const gitEmail = process.env.GIT_AUTHOR_EMAIL || "dev@heycat.local";
  await execInContainer(containerName, ["git", "config", "--global", "user.name", gitName]);
  await execInContainer(containerName, ["git", "config", "--global", "user.email", gitEmail]);

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
  const { issue: issueSlug, help } = parseArgs(args);

  if (help || !issueSlug) {
    log(`
${colors.bold}Usage:${colors.reset} bun scripts/docker/create-container.ts --issue <issue-slug>

${colors.bold}Options:${colors.reset}
  --issue, -i  Linear issue slug or identifier (required)
               Examples: docker-development-workflow, HEY-156

${colors.bold}Description:${colors.reset}
  Creates a new Docker development container linked to a Linear issue:
  - Validates the issue exists in Linear
  - Creates branch with format: HEY-###-<issue-slug>
  - Starts container via docker-compose
  - Runs bun install

${colors.bold}Example:${colors.reset}
  bun scripts/docker/create-container.ts --issue docker-development-workflow
  bun scripts/docker/create-container.ts -i HEY-156

${colors.bold}Note:${colors.reset}
  All development must go through Linear. Freeform branch names are not allowed.
  This ensures PRs are automatically linked to Linear issues.
`);
    process.exit(help ? 0 : 1);
  }

  // Check Docker is available
  if (!(await checkDocker())) {
    error("Docker is not running. Please start Docker and try again.");
    process.exit(1);
  }

  // Validate issue exists in Linear
  info(`\nValidating issue in Linear: ${issueSlug}`);
  const issueInfo = await validateLinearIssue(issueSlug);
  if (!issueInfo) {
    error(`Issue not found in Linear: ${issueSlug}`);
    warn("Make sure the issue exists and LINEAR_API_KEY is set.");
    process.exit(1);
  }

  success(`Found issue: ${issueInfo.identifier} - ${issueInfo.title}`);

  // Build branch name: HEY-###-<slug>
  const issueSlugNormalized = issueInfo.title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
  const branchName = `${issueInfo.identifier}-${issueSlugNormalized}`;
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

// Only run main when executed directly, not when imported as a module
if (import.meta.main) {
  main().catch((err) => {
    error(err.message || String(err));
    process.exit(1);
  });
}

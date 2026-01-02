#!/usr/bin/env bun
/**
 * Create a new git worktree with heycat-specific setup.
 *
 * This script:
 * 1. Creates a git worktree at the specified path with the given branch
 * 2. Generates a unique default hotkey based on the worktree identifier
 * 3. Creates an initial settings file with the unique hotkey
 * 4. Provides instructions for running the dev server
 *
 * Usage: bun scripts/create-worktree.ts <branch-name> [path]
 *
 * The path defaults to worktrees/<branch-name> (inside repository)
 */

import { copyFileSync, existsSync, mkdirSync, writeFileSync } from "fs";
import { homedir } from "os";
import { basename, resolve } from "path";
import { getDevPort } from "./dev-port";

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

function warn(message: string): void {
  console.log(`${colors.yellow}${message}${colors.reset}`);
}

/**
 * Check if we're running from the main repository (not a worktree).
 * Worktrees have .git as a file, main repo has .git as a directory.
 */
async function isMainRepository(): Promise<boolean> {
  const gitPath = resolve(process.cwd(), ".git");
  if (!existsSync(gitPath)) {
    return false;
  }
  const stat = await Bun.file(gitPath).exists();
  // If .git is a directory, we're in the main repo
  // We check by trying to read it as a file - if it fails, it's a directory
  try {
    const content = await Bun.file(gitPath).text();
    // If we can read it and it starts with "gitdir:", it's a worktree
    return !content.startsWith("gitdir:");
  } catch {
    // Can't read as file, so it's a directory (main repo)
    return true;
  }
}

/**
 * Check if a git branch already exists.
 */
async function branchExists(branchName: string): Promise<boolean> {
  const result = await Bun.spawn(["git", "show-ref", "--verify", "--quiet", `refs/heads/${branchName}`], {
    stdout: "ignore",
    stderr: "ignore",
  }).exited;
  return result === 0;
}

/**
 * Check if a worktree already exists at the given path.
 */
async function worktreeExistsAtPath(path: string): Promise<boolean> {
  const result = await Bun.spawn(["git", "worktree", "list", "--porcelain"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  const normalizedPath = resolve(path);
  return output.includes(`worktree ${normalizedPath}`);
}

/**
 * Generate a unique hotkey based on the worktree identifier.
 * Uses a hash of the identifier to select from a predefined set of hotkeys.
 *
 * The hotkey format matches the backend format: "CmdOrControl+Shift+<key>"
 */
function generateHotkey(identifier: string): string {
  // Predefined set of hotkeys (numbers 1-9, then letters)
  const hotkeys = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

  // Simple hash function - sum of char codes
  let hash = 0;
  for (let i = 0; i < identifier.length; i++) {
    hash = (hash * 31 + identifier.charCodeAt(i)) >>> 0;
  }

  const index = hash % hotkeys.length;
  return `CmdOrControl+Shift+${hotkeys[index]}`;
}

/**
 * Get the worktree identifier from the worktree path.
 * This matches the Rust backend's algorithm: the last component of the gitdir path,
 * which is the directory name of the worktree.
 */
function getWorktreeIdentifier(worktreePath: string): string {
  return basename(resolve(worktreePath));
}

/**
 * Validate that the branch name follows the required Linear issue format.
 * Required format: HEY-<number>-<description>
 * Examples: HEY-123-fix-audio, HEY-42-add-dark-mode
 */
function validateBranchName(branchName: string): { valid: boolean; error?: string } {
  const pattern = /^HEY-\d+-[a-z0-9-]+$/i;

  if (!pattern.test(branchName)) {
    if (!branchName.startsWith("HEY-")) {
      return {
        valid: false,
        error: `Branch name must start with a Linear issue ID (HEY-xxx).\n` +
               `  Received: "${branchName}"\n` +
               `  Expected: HEY-<number>-<description> (e.g., HEY-123-fix-audio)`,
      };
    }

    const issueMatch = branchName.match(/^HEY-(\d+)/);
    if (!issueMatch) {
      return {
        valid: false,
        error: `Invalid Linear issue format in branch name.\n` +
               `  Received: "${branchName}"\n` +
               `  Expected: HEY-<number>-<description> (e.g., HEY-123-fix-audio)`,
      };
    }

    // Has valid issue ID but missing or invalid description
    return {
      valid: false,
      error: `Branch name requires a kebab-case description after the issue ID.\n` +
             `  Received: "${branchName}"\n` +
             `  Expected: ${branchName.match(/^HEY-\d+/)?.[0] || "HEY-xxx"}-<description>`,
    };
  }

  return { valid: true };
}

/**
 * Extract the Linear issue ID from a branch name.
 * Returns null if no valid issue ID is found.
 */
function extractIssueId(branchName: string): string | null {
  const match = branchName.match(/^(HEY-\d+)/);
  return match ? match[1] : null;
}

/**
 * Get the heycat application support directory path.
 * Uses Tauri's bundle identifier for the directory name.
 * On macOS: ~/Library/Application Support/com.heycat.app
 */
function getAppSupportDir(): string {
  const home = homedir();
  if (process.platform === "darwin") {
    return resolve(home, "Library/Application Support/com.heycat.app");
  }
  // Linux/other: ~/.local/share/com.heycat.app
  return resolve(home, ".local/share/com.heycat.app");
}

/**
 * Create the initial settings file for the worktree.
 */
function createSettingsFile(identifier: string, hotkey: string): string {
  const appSupportDir = getAppSupportDir();
  const settingsFileName = `settings-${identifier}.json`;
  const settingsPath = resolve(appSupportDir, settingsFileName);

  // Ensure the app support directory exists
  if (!existsSync(appSupportDir)) {
    mkdirSync(appSupportDir, { recursive: true });
  }

  // Create settings with unique hotkey
  const settings = {
    "hotkey.recordingShortcut": hotkey,
  };

  writeFileSync(settingsPath, JSON.stringify(settings, null, 2));
  return settingsPath;
}

/**
 * Create the git worktree.
 */
async function createWorktree(branchName: string, worktreePath: string): Promise<boolean> {
  const args = ["worktree", "add", worktreePath, "-b", branchName];

  log(`Creating worktree: git ${args.join(" ")}`);

  const result = await Bun.spawn(["git", ...args], {
    stdout: "inherit",
    stderr: "inherit",
  }).exited;

  return result === 0;
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);

  if (args.length === 0 || args[0] === "--help" || args[0] === "-h") {
    log(`
${colors.bold}Usage:${colors.reset} bun scripts/create-worktree.ts <branch-name> [path]

${colors.bold}Arguments:${colors.reset}
  branch-name  Branch name in format: HEY-<number>-<description> (REQUIRED)
               Examples: HEY-123-fix-audio, HEY-42-add-dark-mode
  path         Path for the worktree (default: worktrees/<branch-name>)

${colors.bold}Description:${colors.reset}
  Creates a new git worktree with heycat-specific setup:
  - Validates branch name follows Linear issue format (HEY-xxx-description)
  - Creates the worktree with a new branch
  - Generates a unique hotkey based on the worktree name
  - Creates a settings file with the unique hotkey

${colors.bold}Example:${colors.reset}
  bun scripts/create-worktree.ts HEY-123-audio-improvements
  bun scripts/create-worktree.ts HEY-42-fix-memory-leak worktrees/memory-fix

${colors.bold}Note:${colors.reset}
  A Linear issue ID is required. Create an issue in Linear first.
`);
    process.exit(0);
  }

  // Validate we're in the main repository
  if (!(await isMainRepository())) {
    error("This script must be run from the main repository, not from a worktree.");
    process.exit(1);
  }

  const branchName = args[0];
  const worktreePath = args[1] || resolve(process.cwd(), "worktrees", branchName);

  // Validate branch name format (Linear issue required)
  const validation = validateBranchName(branchName);
  if (!validation.valid) {
    error(validation.error!);
    log(`\n${colors.yellow}Hint:${colors.reset} All worktrees require a Linear issue ID.`);
    log(`  Create an issue in Linear first, then use: bun scripts/create-worktree.ts HEY-<id>-<description>`);
    process.exit(1);
  }

  const issueId = extractIssueId(branchName);

  log(`\n${colors.bold}Creating heycat worktree${colors.reset}\n`);
  info(`Linear issue: ${issueId}`);
  info(`Branch: ${branchName}`);
  info(`Path: ${worktreePath}`);

  // Check if branch already exists
  if (await branchExists(branchName)) {
    error(`Branch '${branchName}' already exists.`);
    warn("To create a worktree for an existing branch, use:");
    log(`  git worktree add ${worktreePath} ${branchName}`);
    process.exit(1);
  }

  // Check if path already exists
  if (existsSync(worktreePath)) {
    error(`Path '${worktreePath}' already exists.`);
    process.exit(1);
  }

  // Check if a worktree already exists at this path
  if (await worktreeExistsAtPath(worktreePath)) {
    error(`A worktree already exists at '${worktreePath}'.`);
    process.exit(1);
  }

  // Create the worktree
  log("");
  if (!(await createWorktree(branchName, worktreePath))) {
    error("Failed to create worktree.");
    process.exit(1);
  }

  // Generate unique hotkey and port
  const identifier = getWorktreeIdentifier(worktreePath);
  const hotkey = generateHotkey(identifier);
  const devPort = getDevPort(identifier);

  info(`\nWorktree identifier: ${identifier}`);
  info(`Generated hotkey: ${hotkey}`);
  info(`Dev server port: ${devPort}`);

  // Create settings file
  const settingsPath = createSettingsFile(identifier, hotkey);
  success(`\nSettings file created: ${settingsPath}`);

  // Copy .env file if it exists in main repo (it's gitignored so not in worktree)
  const mainEnvPath = resolve(process.cwd(), ".env");
  const worktreeEnvPath = resolve(worktreePath, ".env");
  if (existsSync(mainEnvPath) && !existsSync(worktreeEnvPath)) {
    copyFileSync(mainEnvPath, worktreeEnvPath);
    success(`Copied .env from main repository`);
  }

  // Print instructions
  log(`
${colors.bold}${colors.green}Worktree created successfully!${colors.reset}

${colors.bold}Next steps:${colors.reset}

  1. Navigate to the worktree:
     ${colors.cyan}cd ${worktreePath}${colors.reset}

  2. Install dependencies:
     ${colors.cyan}bun install${colors.reset}

  3. Start the development server:
     ${colors.cyan}bun run tauri dev${colors.reset}

${colors.bold}Configuration:${colors.reset}
  Hotkey: ${colors.yellow}${hotkey}${colors.reset}
  Dev port: ${colors.yellow}${devPort}${colors.reset}
  (Main repo uses port 1420 and a different hotkey, so both can run simultaneously)

${colors.bold}Note:${colors.reset}
  To remove this worktree later:
     ${colors.cyan}git worktree remove ${worktreePath}${colors.reset}
`);
}

main().catch((err) => {
  error(err.message || String(err));
  process.exit(1);
});

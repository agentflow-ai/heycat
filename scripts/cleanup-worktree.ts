#!/usr/bin/env bun
/**
 * Clean up worktree-specific data directories and configuration files.
 *
 * Since worktree data is stored outside the git directory, this script is needed
 * to properly clean up when a worktree is removed.
 *
 * Usage:
 *   bun scripts/cleanup-worktree.ts --list                 # List all worktree data dirs
 *   bun scripts/cleanup-worktree.ts --orphaned             # Clean up orphaned data
 *   bun scripts/cleanup-worktree.ts <path-or-id>           # Clean up specific worktree
 *   bun scripts/cleanup-worktree.ts <path-or-id> --force   # Skip confirmation
 *   bun scripts/cleanup-worktree.ts <path-or-id> --remove-worktree  # Also remove git worktree
 */

import { existsSync, readdirSync, rmSync, statSync } from "fs";
import { homedir } from "os";
import { basename, resolve } from "path";
import { createInterface } from "readline";

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

function dim(message: string): void {
  console.log(`${colors.dim}${message}${colors.reset}`);
}

/**
 * Directory paths where heycat stores worktree-specific data.
 */
export function getDataDir(): string {
  const home = homedir();
  if (process.platform === "darwin") {
    return resolve(home, ".local/share");
  }
  return resolve(home, ".local/share");
}

export function getConfigDir(): string {
  const home = homedir();
  if (process.platform === "darwin") {
    return resolve(home, ".config");
  }
  return resolve(home, ".config");
}

/**
 * Base app directory name (matches Rust paths.rs)
 */
const APP_DIR_NAME = "heycat";

/**
 * Find all heycat worktree data directories.
 * Returns directories matching the pattern heycat-* (not the main heycat dir).
 */
export function findWorktreeDataDirs(): { dataDir: string | null; configDir: string | null; identifier: string }[] {
  const results: { dataDir: string | null; configDir: string | null; identifier: string }[] = [];
  const seenIdentifiers = new Set<string>();

  const dataBaseDir = getDataDir();
  const configBaseDir = getConfigDir();

  // Scan data directory for heycat-* directories
  if (existsSync(dataBaseDir)) {
    const entries = readdirSync(dataBaseDir);
    for (const entry of entries) {
      if (entry.startsWith(`${APP_DIR_NAME}-`)) {
        const identifier = entry.substring(APP_DIR_NAME.length + 1);
        if (!seenIdentifiers.has(identifier)) {
          seenIdentifiers.add(identifier);
          const dataPath = resolve(dataBaseDir, entry);
          const configPath = resolve(configBaseDir, entry);
          results.push({
            dataDir: existsSync(dataPath) ? dataPath : null,
            configDir: existsSync(configPath) ? configPath : null,
            identifier,
          });
        }
      }
    }
  }

  // Scan config directory for heycat-* directories we might have missed
  if (existsSync(configBaseDir)) {
    const entries = readdirSync(configBaseDir);
    for (const entry of entries) {
      if (entry.startsWith(`${APP_DIR_NAME}-`)) {
        const identifier = entry.substring(APP_DIR_NAME.length + 1);
        if (!seenIdentifiers.has(identifier)) {
          seenIdentifiers.add(identifier);
          const dataPath = resolve(dataBaseDir, entry);
          const configPath = resolve(configBaseDir, entry);
          results.push({
            dataDir: existsSync(dataPath) ? dataPath : null,
            configDir: existsSync(configPath) ? configPath : null,
            identifier,
          });
        }
      }
    }
  }

  return results;
}

/**
 * Get list of active git worktrees from the main repository.
 * Returns the worktree identifiers (directory names).
 */
export async function getActiveWorktrees(): Promise<Set<string>> {
  const result = await Bun.spawn(["git", "worktree", "list", "--porcelain"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();

  const identifiers = new Set<string>();

  // Parse porcelain output to get worktree paths
  const lines = output.split("\n");
  for (const line of lines) {
    if (line.startsWith("worktree ")) {
      const worktreePath = line.substring("worktree ".length);
      // The identifier is the basename of the worktree path
      const id = basename(worktreePath);
      identifiers.add(id);
    }
  }

  return identifiers;
}

/**
 * Find orphaned worktree data (data directories with no corresponding git worktree).
 */
export async function findOrphanedData(): Promise<{ dataDir: string | null; configDir: string | null; identifier: string }[]> {
  const allDirs = findWorktreeDataDirs();
  const activeWorktrees = await getActiveWorktrees();

  return allDirs.filter((dir) => !activeWorktrees.has(dir.identifier));
}

/**
 * Get directory size in bytes.
 */
function getDirectorySize(dirPath: string): number {
  if (!existsSync(dirPath)) return 0;

  let size = 0;
  try {
    const entries = readdirSync(dirPath, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = resolve(dirPath, entry.name);
      if (entry.isDirectory()) {
        size += getDirectorySize(fullPath);
      } else {
        size += statSync(fullPath).size;
      }
    }
  } catch {
    // Ignore permission errors
  }
  return size;
}

/**
 * Format bytes as human-readable size.
 */
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/**
 * Prompt user for confirmation.
 */
async function confirm(message: string): Promise<boolean> {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((resolve) => {
    rl.question(`${message} [y/N]: `, (answer) => {
      rl.close();
      resolve(answer.toLowerCase() === "y" || answer.toLowerCase() === "yes");
    });
  });
}

/**
 * Delete directories for a worktree.
 */
function deleteWorktreeDirs(dirs: { dataDir: string | null; configDir: string | null }): void {
  if (dirs.dataDir && existsSync(dirs.dataDir)) {
    rmSync(dirs.dataDir, { recursive: true, force: true });
    success(`  Deleted: ${dirs.dataDir}`);
  }
  if (dirs.configDir && existsSync(dirs.configDir)) {
    rmSync(dirs.configDir, { recursive: true, force: true });
    success(`  Deleted: ${dirs.configDir}`);
  }
}

/**
 * Find worktree data by path or identifier.
 */
export function findWorktreeByPathOrId(
  pathOrId: string
): { dataDir: string | null; configDir: string | null; identifier: string } | null {
  const allDirs = findWorktreeDataDirs();

  // First, try to match by identifier directly
  const byId = allDirs.find((d) => d.identifier === pathOrId);
  if (byId) return byId;

  // Try to extract identifier from path
  const normalizedPath = resolve(pathOrId);
  const potentialId = basename(normalizedPath);

  // Check if this looks like a worktree directory name (heycat-xxx)
  if (potentialId.startsWith(`${APP_DIR_NAME}-`)) {
    const identifier = potentialId.substring(APP_DIR_NAME.length + 1);
    const byExtractedId = allDirs.find((d) => d.identifier === identifier);
    if (byExtractedId) return byExtractedId;
  }

  // Try the path's basename directly as an identifier
  const byBasename = allDirs.find((d) => d.identifier === potentialId);
  if (byBasename) return byBasename;

  return null;
}

/**
 * Remove a git worktree.
 */
async function removeGitWorktree(identifier: string): Promise<boolean> {
  // Find the worktree path from git worktree list
  const result = await Bun.spawn(["git", "worktree", "list", "--porcelain"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();

  let worktreePath: string | null = null;
  const lines = output.split("\n");
  for (const line of lines) {
    if (line.startsWith("worktree ")) {
      const path = line.substring("worktree ".length);
      if (basename(path) === identifier) {
        worktreePath = path;
        break;
      }
    }
  }

  if (!worktreePath) {
    warn(`Git worktree '${identifier}' not found.`);
    return false;
  }

  info(`Removing git worktree: ${worktreePath}`);
  const removeResult = await Bun.spawn(["git", "worktree", "remove", worktreePath], {
    stdout: "inherit",
    stderr: "inherit",
  }).exited;

  return removeResult === 0;
}

/**
 * Print help message.
 */
function printHelp(): void {
  log(`
${colors.bold}Usage:${colors.reset} bun scripts/cleanup-worktree.ts [options] [path-or-id]

${colors.bold}Options:${colors.reset}
  --list              List all worktree-specific data directories
  --orphaned          Clean up orphaned data (worktrees that no longer exist)
  --force             Skip confirmation prompts
  --remove-worktree   Also remove the git worktree itself
  --help, -h          Show this help message

${colors.bold}Arguments:${colors.reset}
  path-or-id          Worktree path or identifier to clean up

${colors.bold}Examples:${colors.reset}
  ${colors.cyan}bun scripts/cleanup-worktree.ts --list${colors.reset}
    List all heycat worktree data directories

  ${colors.cyan}bun scripts/cleanup-worktree.ts --orphaned${colors.reset}
    Find and clean up data for worktrees that no longer exist

  ${colors.cyan}bun scripts/cleanup-worktree.ts heycat-feature-branch${colors.reset}
    Clean up data for the 'heycat-feature-branch' worktree

  ${colors.cyan}bun scripts/cleanup-worktree.ts ../heycat-feature --remove-worktree${colors.reset}
    Remove both data and the git worktree itself

${colors.bold}Data Locations:${colors.reset}
  Data:   ~/.local/share/heycat-{identifier}/
  Config: ~/.config/heycat-{identifier}/
`);
}

/**
 * List all worktree data directories.
 */
async function listCommand(): Promise<void> {
  const allDirs = findWorktreeDataDirs();
  const activeWorktrees = await getActiveWorktrees();

  if (allDirs.length === 0) {
    info("No worktree-specific data directories found.");
    return;
  }

  log(`\n${colors.bold}Worktree Data Directories${colors.reset}\n`);

  for (const dir of allDirs) {
    const isOrphaned = !activeWorktrees.has(dir.identifier);
    const statusColor = isOrphaned ? colors.yellow : colors.green;
    const status = isOrphaned ? " (orphaned)" : "";

    log(`${statusColor}${colors.bold}${dir.identifier}${status}${colors.reset}`);

    if (dir.dataDir) {
      const size = getDirectorySize(dir.dataDir);
      dim(`  Data:   ${dir.dataDir} (${formatSize(size)})`);
    }
    if (dir.configDir) {
      const size = getDirectorySize(dir.configDir);
      dim(`  Config: ${dir.configDir} (${formatSize(size)})`);
    }
    log("");
  }

  const orphanedCount = allDirs.filter((d) => !activeWorktrees.has(d.identifier)).length;
  if (orphanedCount > 0) {
    warn(`Found ${orphanedCount} orphaned director${orphanedCount === 1 ? "y" : "ies"}. Run with --orphaned to clean up.`);
  }
}

/**
 * Clean up orphaned worktree data.
 */
async function orphanedCommand(force: boolean): Promise<void> {
  const orphaned = await findOrphanedData();

  if (orphaned.length === 0) {
    success("No orphaned worktree data found.");
    return;
  }

  log(`\n${colors.bold}Orphaned Worktree Data${colors.reset}\n`);

  let totalSize = 0;
  for (const dir of orphaned) {
    log(`${colors.yellow}${colors.bold}${dir.identifier}${colors.reset}`);
    if (dir.dataDir) {
      const size = getDirectorySize(dir.dataDir);
      totalSize += size;
      log(`  ${colors.red}Will delete:${colors.reset} ${dir.dataDir} (${formatSize(size)})`);
    }
    if (dir.configDir) {
      const size = getDirectorySize(dir.configDir);
      totalSize += size;
      log(`  ${colors.red}Will delete:${colors.reset} ${dir.configDir} (${formatSize(size)})`);
    }
    log("");
  }

  log(`Total: ${formatSize(totalSize)} in ${orphaned.length} worktree${orphaned.length === 1 ? "" : "s"}\n`);

  if (!force) {
    const confirmed = await confirm(`Delete all orphaned worktree data?`);
    if (!confirmed) {
      info("Cancelled.");
      return;
    }
  }

  for (const dir of orphaned) {
    info(`Cleaning up ${dir.identifier}...`);
    deleteWorktreeDirs(dir);
  }

  success(`\nCleaned up ${orphaned.length} orphaned worktree${orphaned.length === 1 ? "" : "s"}.`);
}

/**
 * Clean up a specific worktree.
 */
async function cleanupCommand(pathOrId: string, force: boolean, removeWorktree: boolean): Promise<void> {
  const dir = findWorktreeByPathOrId(pathOrId);

  if (!dir) {
    error(`No worktree data found for '${pathOrId}'.`);
    log("\nUse --list to see available worktree data directories.");
    process.exit(1);
  }

  log(`\n${colors.bold}Worktree: ${dir.identifier}${colors.reset}\n`);

  let totalSize = 0;
  if (dir.dataDir) {
    const size = getDirectorySize(dir.dataDir);
    totalSize += size;
    log(`  ${colors.red}Will delete:${colors.reset} ${dir.dataDir} (${formatSize(size)})`);
  }
  if (dir.configDir) {
    const size = getDirectorySize(dir.configDir);
    totalSize += size;
    log(`  ${colors.red}Will delete:${colors.reset} ${dir.configDir} (${formatSize(size)})`);
  }

  if (removeWorktree) {
    const activeWorktrees = await getActiveWorktrees();
    if (activeWorktrees.has(dir.identifier)) {
      log(`  ${colors.red}Will remove git worktree${colors.reset}`);
    }
  }

  log(`\nTotal: ${formatSize(totalSize)}\n`);

  if (!force) {
    const confirmed = await confirm(`Delete worktree data for '${dir.identifier}'?`);
    if (!confirmed) {
      info("Cancelled.");
      return;
    }
  }

  if (removeWorktree) {
    await removeGitWorktree(dir.identifier);
  }

  info(`Cleaning up ${dir.identifier}...`);
  deleteWorktreeDirs(dir);

  success(`\nCleaned up worktree data for '${dir.identifier}'.`);
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);

  // Parse flags
  const flags = {
    list: false,
    orphaned: false,
    force: false,
    removeWorktree: false,
    help: false,
  };
  const positionalArgs: string[] = [];

  for (const arg of args) {
    if (arg === "--list") {
      flags.list = true;
    } else if (arg === "--orphaned") {
      flags.orphaned = true;
    } else if (arg === "--force" || arg === "-f") {
      flags.force = true;
    } else if (arg === "--remove-worktree") {
      flags.removeWorktree = true;
    } else if (arg === "--help" || arg === "-h") {
      flags.help = true;
    } else if (!arg.startsWith("-")) {
      positionalArgs.push(arg);
    } else {
      error(`Unknown option: ${arg}`);
      process.exit(1);
    }
  }

  if (flags.help) {
    printHelp();
    process.exit(0);
  }

  if (flags.list) {
    await listCommand();
    return;
  }

  if (flags.orphaned) {
    await orphanedCommand(flags.force);
    return;
  }

  if (positionalArgs.length === 0) {
    printHelp();
    process.exit(0);
  }

  await cleanupCommand(positionalArgs[0], flags.force, flags.removeWorktree);
}

main().catch((err) => {
  error(err.message || String(err));
  process.exit(1);
});

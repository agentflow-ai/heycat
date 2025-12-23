/**
 * Port detection utility for worktree-aware dev server configuration.
 *
 * Assigns deterministic ports based on worktree identifier:
 * - Main repo: port 1420 (default)
 * - Worktrees: ports 1421-1429 (based on identifier hash)
 */

import { existsSync, readFileSync, statSync } from "fs";
import { basename, resolve } from "path";

const BASE_PORT = 1420;
const WORKTREE_PORT_OFFSET = 1;
const WORKTREE_PORT_RANGE = 9;

/**
 * Get the worktree identifier from the current directory.
 * Returns null if running in the main repository.
 *
 * The identifier is the directory name of the worktree, which matches
 * the Rust backend's algorithm (last component of gitdir path).
 */
export function getWorktreeIdentifier(): string | null {
  const gitPath = resolve(process.cwd(), ".git");

  if (!existsSync(gitPath)) {
    return null;
  }

  // Check if .git is a file (worktree) or directory (main repo)
  const stat = statSync(gitPath);
  if (stat.isDirectory()) {
    // Main repository
    return null;
  }

  // It's a file - parse the gitdir reference
  const content = readFileSync(gitPath, "utf-8").trim();
  if (!content.startsWith("gitdir:")) {
    return null;
  }

  // Extract the gitdir path and get the worktree name
  // Format: "gitdir: /path/to/main/.git/worktrees/worktree-name"
  const gitdir = content.substring("gitdir:".length).trim();
  const worktreeName = basename(gitdir);

  // The identifier is the parent directory name of the worktree
  // (e.g., "heycat-maximus" from "/path/worktrees/heycat-maximus")
  return basename(resolve(process.cwd()));
}

/**
 * Get the dev server port for the given worktree identifier.
 * Uses the same hash algorithm as hotkey generation for consistency.
 */
export function getDevPort(identifier: string | null): number {
  if (!identifier) {
    return BASE_PORT;
  }

  // Same hash algorithm as create-worktree.ts generateHotkey()
  let hash = 0;
  for (let i = 0; i < identifier.length; i++) {
    hash = (hash * 31 + identifier.charCodeAt(i)) >>> 0;
  }

  return BASE_PORT + WORKTREE_PORT_OFFSET + (hash % WORKTREE_PORT_RANGE);
}

/**
 * Get the dev server port for the current directory.
 * Convenience function that combines identifier detection and port calculation.
 */
export function getCurrentDevPort(): number {
  return getDevPort(getWorktreeIdentifier());
}

// CLI usage: print the port for the current directory
if (import.meta.main) {
  const identifier = getWorktreeIdentifier();
  const port = getDevPort(identifier);

  if (identifier) {
    console.log(`Worktree: ${identifier}`);
  } else {
    console.log("Main repository");
  }
  console.log(`Dev port: ${port}`);
}

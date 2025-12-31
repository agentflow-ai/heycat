/**
 * Worktree detection and utilities.
 *
 * Provides functions for detecting git worktree context and extracting
 * worktree information. Used by scripts that need worktree-aware behavior.
 */

import { existsSync, readFileSync, statSync } from "fs";
import { dirname, resolve } from "path";

/**
 * Information about the current worktree context.
 */
export interface WorktreeInfo {
  /** Worktree directory name (e.g., "feature-audio") */
  identifier: string;
  /** Path to the main repository */
  mainRepoPath: string;
  /** Current worktree path */
  worktreePath: string;
  /** Path to .git/worktrees/<name> */
  gitdirPath: string;
}

/**
 * Detect if we're running from a worktree and return context info.
 * Returns null if running from main repository.
 */
export async function detectWorktreeContext(): Promise<WorktreeInfo | null> {
  const gitPath = resolve(process.cwd(), ".git");

  if (!existsSync(gitPath)) {
    return null;
  }

  // Check if .git is a file (worktree) or directory (main repo)
  const stat = statSync(gitPath);
  if (stat.isDirectory()) {
    // Main repo - .git is a directory
    return null;
  }

  // Worktree - .git is a file containing gitdir reference
  const content = readFileSync(gitPath, "utf-8").trim();
  if (!content.startsWith("gitdir: ")) {
    return null;
  }

  // Extract gitdir path: "gitdir: /path/to/repo/.git/worktrees/<name>"
  const gitdirPath = content.substring("gitdir: ".length);

  // Navigate up from gitdir to find main repo:
  // .git/worktrees/<name> -> .git -> repo root
  const gitDir = dirname(dirname(gitdirPath)); // .git
  const mainRepoPath = dirname(gitDir); // repo root

  // The identifier is the last component of gitdirPath (worktree name)
  const identifier = gitdirPath.split("/").pop() || "";

  return {
    identifier,
    mainRepoPath,
    worktreePath: process.cwd(),
    gitdirPath,
  };
}

/**
 * Get the current branch of a repository.
 */
export async function getRepoBranch(repoPath: string): Promise<string> {
  const result = await Bun.spawn(["git", "-C", repoPath, "rev-parse", "--abbrev-ref", "HEAD"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output.trim();
}

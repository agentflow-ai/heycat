#!/usr/bin/env bun
/**
 * Complete a feature and merge it to main from a worktree.
 *
 * This script implements the "golden path" for feature completion:
 * 1. Fetch latest main
 * 2. Rebase feature onto main
 * 3. Squash all commits into a single conventional commit
 * 4. Fast-forward merge to main
 * 5. Reset worktree branch to main (ready for next feature)
 *
 * Usage:
 *   bun scripts/complete-feature.ts              # Complete feature
 *   bun scripts/complete-feature.ts --continue   # Continue after conflict resolution
 *   bun scripts/complete-feature.ts --dry-run    # Preview what would happen
 *   bun scripts/complete-feature.ts --help       # Show help
 */

import { existsSync } from "fs";
import { resolve } from "path";
import { detectWorktreeContext, type WorktreeInfo } from "./sync-agile";

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

interface Flags {
  continue: boolean;
  dryRun: boolean;
  help: boolean;
}

/**
 * Parse command line arguments.
 */
export function parseArgs(args: string[]): Flags {
  const flags: Flags = {
    continue: false,
    dryRun: false,
    help: false,
  };

  for (const arg of args) {
    if (arg === "--continue") {
      flags.continue = true;
    } else if (arg === "--dry-run") {
      flags.dryRun = true;
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
${colors.bold}Usage:${colors.reset} bun scripts/complete-feature.ts [options]

${colors.bold}Description:${colors.reset}
  Completes a feature developed in a worktree and merges it to main.
  Creates a single squashed commit from all feature commits.

${colors.bold}Options:${colors.reset}
  --continue     Continue after resolving rebase conflicts
  --dry-run      Preview what would happen without making changes
  --help, -h     Show this help message

${colors.bold}Workflow:${colors.reset}
  1. Fetches latest main
  2. Rebases feature onto main
  3. Squashes all commits into one conventional commit
  4. Merges to main (fast-forward)
  5. Resets worktree to main (ready for next feature)

${colors.bold}Examples:${colors.reset}
  ${colors.cyan}bun scripts/complete-feature.ts${colors.reset}
    Complete the feature and merge to main

  ${colors.cyan}bun scripts/complete-feature.ts --dry-run${colors.reset}
    Preview the commits that would be squashed

  ${colors.cyan}bun scripts/complete-feature.ts --continue${colors.reset}
    Continue after resolving conflicts

${colors.bold}Note:${colors.reset}
  This script must be run from a worktree, not the main repository.
  After completion, run 'bun scripts/sync-agile.ts' if agile folder needs syncing.
`);
}

/**
 * Check if the working directory is clean.
 */
async function isWorkingDirectoryClean(): Promise<boolean> {
  const result = await Bun.spawn(["git", "status", "--porcelain"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output.trim() === "";
}

/**
 * Check if a rebase is in progress.
 * In a worktree, .git is a file pointing to the actual git dir.
 */
async function isRebaseInProgress(): Promise<boolean> {
  const gitPath = resolve(process.cwd(), ".git");

  // In a worktree, .git is a file containing "gitdir: /path/to/git/dir"
  if (existsSync(gitPath)) {
    try {
      const content = await Bun.file(gitPath).text();
      if (content.startsWith("gitdir: ")) {
        const gitDir = content.substring("gitdir: ".length).trim();
        return existsSync(resolve(gitDir, "rebase-merge")) || existsSync(resolve(gitDir, "rebase-apply"));
      }
    } catch {
      // Fall through to directory check
    }
  }

  // Main repo case (shouldn't happen but handle it)
  return existsSync(resolve(gitPath, "rebase-merge")) || existsSync(resolve(gitPath, "rebase-apply"));
}

/**
 * Get the current branch name.
 */
async function getCurrentBranch(): Promise<string> {
  const result = await Bun.spawn(["git", "rev-parse", "--abbrev-ref", "HEAD"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output.trim();
}

/**
 * Get commits since divergence from main.
 */
async function getCommitsSinceMain(): Promise<string[]> {
  const result = await Bun.spawn(["git", "log", "origin/main..HEAD", "--format=%s"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output
    .trim()
    .split("\n")
    .filter((c) => c.length > 0);
}

/**
 * Derive a conventional commit message from WIP commits.
 */
export function deriveCommitMessage(commits: string[]): string {
  if (commits.length === 0) {
    return "chore: merge feature";
  }

  // Strip WIP prefixes and clean up messages
  const cleaned = commits
    .map((c) => c.replace(/^WIP:\s*/i, "").trim())
    .filter((c) => c.length > 0);

  if (cleaned.length === 0) {
    return "chore: merge feature";
  }

  // Detect commit type from messages
  let type = "feat";
  const typeCounts = { feat: 0, fix: 0, refactor: 0, chore: 0, docs: 0, test: 0 };

  for (const msg of cleaned) {
    const lowerMsg = msg.toLowerCase();
    if (lowerMsg.startsWith("fix") || lowerMsg.includes("bug") || lowerMsg.includes("fix")) {
      typeCounts.fix++;
    } else if (lowerMsg.startsWith("refactor") || lowerMsg.includes("refactor")) {
      typeCounts.refactor++;
    } else if (lowerMsg.startsWith("docs") || lowerMsg.includes("document")) {
      typeCounts.docs++;
    } else if (lowerMsg.startsWith("test") || lowerMsg.includes("test")) {
      typeCounts.test++;
    } else if (lowerMsg.startsWith("chore")) {
      typeCounts.chore++;
    } else {
      typeCounts.feat++;
    }
  }

  // Find the dominant type
  let maxCount = 0;
  for (const [t, count] of Object.entries(typeCounts)) {
    if (count > maxCount) {
      maxCount = count;
      type = t;
    }
  }

  // Extract a scope if there's a common pattern
  const scopeMatch = cleaned[0].match(/^\[([^\]]+)\]/);
  const scope = scopeMatch ? `(${scopeMatch[1]})` : "";

  // Create the message body
  // Use the first non-trivial commit message as the main description
  let mainMessage = cleaned[0].replace(/^\[[^\]]+\]\s*/, "").replace(/^(feat|fix|refactor|chore|docs|test):\s*/i, "");

  // If there are multiple commits, summarize
  if (cleaned.length > 1) {
    // Try to create a summary
    const keywords = new Set<string>();
    for (const msg of cleaned) {
      // Extract key action words
      const words = msg.toLowerCase().split(/\s+/);
      for (const word of words) {
        if (["add", "implement", "create", "update", "fix", "remove", "refactor"].includes(word)) {
          keywords.add(word);
        }
      }
    }

    if (keywords.size > 0) {
      mainMessage = `${Array.from(keywords).slice(0, 2).join(" and ")} ${mainMessage.split(" ").slice(0, 5).join(" ")}`;
    }
  }

  // Clean up and capitalize first letter
  mainMessage = mainMessage.charAt(0).toLowerCase() + mainMessage.slice(1);
  mainMessage = mainMessage.replace(/\s+/g, " ").trim();

  // Ensure message isn't too long
  if (mainMessage.length > 72) {
    mainMessage = mainMessage.substring(0, 69) + "...";
  }

  return `${type}${scope}: ${mainMessage}`;
}

/**
 * Get files with conflicts.
 */
async function getConflictFiles(): Promise<string[]> {
  const result = await Bun.spawn(["git", "diff", "--name-only", "--diff-filter=U"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output
    .trim()
    .split("\n")
    .filter((f) => f.length > 0);
}

/**
 * Fetch latest main.
 */
async function fetchMain(): Promise<boolean> {
  const result = await Bun.spawn(["git", "fetch", "origin", "main"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  return (await result.exited) === 0;
}

/**
 * Get the number of commits main is ahead.
 */
async function getMainAheadCount(): Promise<number> {
  const result = await Bun.spawn(["git", "rev-list", "--count", "HEAD..origin/main"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return parseInt(output.trim(), 10) || 0;
}

/**
 * Rebase onto main.
 */
async function rebaseOntoMain(): Promise<{ success: boolean; output: string }> {
  const result = await Bun.spawn(["git", "rebase", "origin/main"], {
    stdout: "pipe",
    stderr: "pipe",
  });

  const stdout = await new Response(result.stdout).text();
  const stderr = await new Response(result.stderr).text();
  const exitCode = await result.exited;

  return {
    success: exitCode === 0,
    output: stdout + stderr,
  };
}

/**
 * Continue a rebase in progress.
 */
async function continueRebase(): Promise<{ success: boolean; output: string }> {
  const result = await Bun.spawn(["git", "rebase", "--continue"], {
    stdout: "pipe",
    stderr: "pipe",
  });

  const stdout = await new Response(result.stdout).text();
  const stderr = await new Response(result.stderr).text();
  const exitCode = await result.exited;

  return {
    success: exitCode === 0,
    output: stdout + stderr,
  };
}

/**
 * Squash all commits since main into one.
 */
async function squashCommits(message: string): Promise<{ success: boolean; output: string }> {
  // Soft reset to origin/main (keeps changes staged)
  const resetResult = await Bun.spawn(["git", "reset", "--soft", "origin/main"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  if ((await resetResult.exited) !== 0) {
    const stderr = await new Response(resetResult.stderr).text();
    return { success: false, output: stderr };
  }

  // Create the squashed commit
  const commitResult = await Bun.spawn(["git", "commit", "-m", message], {
    stdout: "pipe",
    stderr: "pipe",
  });

  const stdout = await new Response(commitResult.stdout).text();
  const stderr = await new Response(commitResult.stderr).text();
  const exitCode = await commitResult.exited;

  return {
    success: exitCode === 0,
    output: stdout + stderr,
  };
}

/**
 * Merge to main via fast-forward.
 */
async function mergeToMain(mainRepoPath: string, branchName: string): Promise<{ success: boolean; output: string }> {
  const result = await Bun.spawn(["git", "-C", mainRepoPath, "merge", "--ff-only", branchName], {
    stdout: "pipe",
    stderr: "pipe",
  });

  const stdout = await new Response(result.stdout).text();
  const stderr = await new Response(result.stderr).text();
  const exitCode = await result.exited;

  return {
    success: exitCode === 0,
    output: stdout + stderr,
  };
}

/**
 * Reset worktree branch to main.
 */
async function resetToMain(): Promise<boolean> {
  const result = await Bun.spawn(["git", "reset", "--hard", "origin/main"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  return (await result.exited) === 0;
}

/**
 * Get the short hash of the current commit.
 */
async function getShortHash(): Promise<string> {
  const result = await Bun.spawn(["git", "rev-parse", "--short", "HEAD"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const output = await new Response(result.stdout).text();
  return output.trim();
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const flags = parseArgs(args);

  if (flags.help) {
    printHelp();
    process.exit(0);
  }

  // Detect worktree context
  const worktree = await detectWorktreeContext();
  if (!worktree) {
    error("This script must be run from a worktree, not the main repository.");
    log("\nTo complete a feature, navigate to the worktree directory first.");
    process.exit(1);
  }

  log(`\n${colors.bold}Completing feature${colors.reset}\n`);
  info(`Worktree: ${worktree.identifier}`);

  const currentBranch = await getCurrentBranch();
  info(`Branch: ${currentBranch}`);

  // Handle --continue flag
  if (flags.continue) {
    if (!(await isRebaseInProgress())) {
      error("No rebase in progress. Nothing to continue.");
      process.exit(1);
    }

    log("\n" + colors.bold + "Continuing rebase..." + colors.reset);
    const continueResult = await continueRebase();
    if (!continueResult.success) {
      error("Failed to continue rebase:");
      log(continueResult.output);

      const conflicts = await getConflictFiles();
      if (conflicts.length > 0) {
        warn("\nConflicting files:");
        for (const file of conflicts) {
          log(`  - ${file}`);
        }
        log("\nResolve conflicts and run again with --continue");
      }
      process.exit(1);
    }
    success("   Rebase completed successfully");
  } else {
    // Check for clean working directory
    if (!(await isWorkingDirectoryClean())) {
      error("Working directory is not clean.");
      log("\nCommit or stash your changes before completing the feature.");
      process.exit(1);
    }

    // Check if rebase is already in progress
    if (await isRebaseInProgress()) {
      error("A rebase is already in progress.");
      log("\nResolve the rebase first, then run with --continue, or abort with 'git rebase --abort'");
      process.exit(1);
    }

    // Get commits to squash
    const commits = await getCommitsSinceMain();
    if (commits.length === 0) {
      warn("No commits to merge. Branch is already up to date with main.");
      process.exit(0);
    }

    log(`\n${colors.bold}Commits to squash (${commits.length}):${colors.reset}`);
    for (const commit of commits.slice(0, 10)) {
      dim(`   - ${commit}`);
    }
    if (commits.length > 10) {
      dim(`   ... and ${commits.length - 10} more`);
    }

    if (flags.dryRun) {
      const derivedMessage = deriveCommitMessage(commits);
      log(`\n${colors.bold}Derived commit message:${colors.reset}`);
      info(`   ${derivedMessage}`);
      log("\n" + colors.dim + "(dry run - no changes made)" + colors.reset);
      process.exit(0);
    }

    // Fetch latest main
    log(`\n${colors.bold}Fetching latest main...${colors.reset}`);
    if (!(await fetchMain())) {
      error("Failed to fetch main.");
      process.exit(1);
    }

    const aheadCount = await getMainAheadCount();
    if (aheadCount > 0) {
      dim(`   Main is ${aheadCount} commit${aheadCount === 1 ? "" : "s"} ahead`);
    } else {
      dim("   Already up to date");
    }

    // Rebase onto main
    log(`\n${colors.bold}Rebasing onto main...${colors.reset}`);
    const rebaseResult = await rebaseOntoMain();

    if (!rebaseResult.success) {
      const conflicts = await getConflictFiles();
      if (conflicts.length > 0) {
        warn("\nRebase conflict detected!");
        log(`\n${colors.bold}Conflicting files:${colors.reset}`);
        for (const file of conflicts) {
          log(`  - ${file}`);
        }
        log(`
${colors.bold}To resolve:${colors.reset}
  1. Edit the conflicting files to resolve conflicts
  2. ${colors.cyan}git add <resolved-files>${colors.reset}
  3. ${colors.cyan}git rebase --continue${colors.reset}
  4. ${colors.cyan}bun scripts/complete-feature.ts --continue${colors.reset}
`);
        process.exit(1);
      } else {
        error("Rebase failed:");
        log(rebaseResult.output);
        process.exit(1);
      }
    }
    success("   Successfully rebased");
  }

  // Re-gather commits after rebase (they may have changed)
  const finalCommits = await getCommitsSinceMain();
  const commitMessage = deriveCommitMessage(finalCommits);

  // Squash commits
  log(`\n${colors.bold}Squashing into single commit...${colors.reset}`);
  info(`   Message: ${commitMessage}`);

  const squashResult = await squashCommits(commitMessage);
  if (!squashResult.success) {
    error("Failed to squash commits:");
    log(squashResult.output);
    process.exit(1);
  }

  // Merge to main
  log(`\n${colors.bold}Merging to main...${colors.reset}`);
  const currentBranchAfterSquash = await getCurrentBranch();
  const mergeResult = await mergeToMain(worktree.mainRepoPath, currentBranchAfterSquash);

  if (!mergeResult.success) {
    error("Fast-forward merge failed. Main may have diverged.");
    log("\nTry rebasing again: git rebase origin/main");
    process.exit(1);
  }
  success("   Fast-forward merge successful");

  // Get the commit hash before resetting
  const commitHash = await getShortHash();

  // Reset worktree to main
  log(`\n${colors.bold}Resetting worktree to main...${colors.reset}`);
  if (!(await resetToMain())) {
    error("Failed to reset worktree to main.");
    process.exit(1);
  }
  success("   Done! Worktree ready for next feature.");

  // Summary
  log(`
${colors.green}${colors.bold}Complete!${colors.reset}
   Commit: ${colors.cyan}${commitHash}${colors.reset} ${commitMessage}

${colors.dim}Tip: Run 'bun scripts/sync-agile.ts' if agile folder needs syncing${colors.reset}
`);
}

// Only run main when executed directly, not when imported as a module
if (import.meta.main) {
  main().catch((err) => {
    error(err.message || String(err));
    process.exit(1);
  });
}

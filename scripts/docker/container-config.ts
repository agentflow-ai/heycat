/**
 * Container configuration utilities for Docker development workflow.
 *
 * Provides unified environment detection for both worktree and Docker workflows:
 * - isDockerEnvironment(): Check if running in Docker container
 * - getContainerId(): Get the current container/dev ID
 * - getDevMode(): Determine development mode ('docker' | 'worktree' | 'main')
 *
 * Usage:
 *   import { isDockerEnvironment, getContainerId, getDevMode } from './container-config';
 *
 *   if (isDockerEnvironment()) {
 *     console.log(`Running in container: ${getContainerId()}`);
 *   }
 *
 *   const mode = getDevMode();
 *   // mode is 'docker', 'worktree', or 'main'
 */

import { existsSync, readFileSync, statSync } from "fs";
import { basename, resolve } from "path";

/**
 * Development mode type.
 */
export type DevMode = "docker" | "worktree" | "main";

/**
 * Check if running inside a Docker development container.
 * Checks HEYCAT_DOCKER_DEV=1 environment variable.
 */
export function isDockerEnvironment(): boolean {
  return process.env.HEYCAT_DOCKER_DEV === "1";
}

/**
 * Get the container/development ID.
 *
 * In Docker: reads HEYCAT_DEV_ID environment variable
 * In worktree: returns the worktree directory name
 * In main repo: returns null
 */
export function getContainerId(): string | null {
  // Docker environment
  if (isDockerEnvironment()) {
    const devId = process.env.HEYCAT_DEV_ID;
    if (devId && devId !== "default") {
      return devId;
    }
    return null;
  }

  // Worktree environment - check git configuration
  return getWorktreeIdentifier();
}

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

  // The identifier is the current directory name
  return basename(resolve(process.cwd()));
}

/**
 * Determine the current development mode.
 *
 * Returns:
 * - 'docker': Running in Docker development container
 * - 'worktree': Running in a git worktree
 * - 'main': Running in the main repository
 */
export function getDevMode(): DevMode {
  // Docker takes precedence
  if (isDockerEnvironment()) {
    return "docker";
  }

  // Check for worktree
  if (getWorktreeIdentifier() !== null) {
    return "worktree";
  }

  // Default to main repository
  return "main";
}

/**
 * Get full container name from dev ID.
 */
export function getContainerName(devId: string): string {
  return `heycat-dev-${devId}`;
}

/**
 * Configuration summary for the current environment.
 */
export interface EnvironmentInfo {
  mode: DevMode;
  containerId: string | null;
  containerName: string | null;
  isDocker: boolean;
}

/**
 * Get complete environment information.
 */
export function getEnvironmentInfo(): EnvironmentInfo {
  const mode = getDevMode();
  const containerId = getContainerId();

  return {
    mode,
    containerId,
    containerName: containerId ? getContainerName(containerId) : null,
    isDocker: isDockerEnvironment(),
  };
}

// CLI usage: print environment info when run directly
if (import.meta.main) {
  const info = getEnvironmentInfo();

  console.log("Environment Information:");
  console.log(`  Mode: ${info.mode}`);
  console.log(`  Container ID: ${info.containerId ?? "(none)"}`);
  console.log(`  Container Name: ${info.containerName ?? "(none)"}`);
  console.log(`  Is Docker: ${info.isDocker}`);
}

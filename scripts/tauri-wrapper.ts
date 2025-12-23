#!/usr/bin/env bun
/**
 * Tauri CLI wrapper that handles worktree-specific port configuration.
 *
 * This wrapper:
 * 1. Detects if running in a worktree
 * 2. Calculates the appropriate dev server port
 * 3. Injects the port configuration when running `tauri dev`
 * 4. Passes all other commands through to Tauri CLI unchanged
 *
 * Usage: bun scripts/tauri-wrapper.ts [tauri-args...]
 *
 * Examples:
 *   bun scripts/tauri-wrapper.ts dev        # Runs dev with correct port
 *   bun scripts/tauri-wrapper.ts build      # Passes through to tauri build
 *   bun scripts/tauri-wrapper.ts --help     # Passes through to tauri --help
 */

import { getDevPort, getWorktreeIdentifier } from "./dev-port";

const args = process.argv.slice(2);
const identifier = getWorktreeIdentifier();
const port = getDevPort(identifier);

// Check if this is a `dev` command (first non-flag arg is "dev")
const isDevCommand = args.some((arg, index) => {
  // Skip flags
  if (arg.startsWith("-")) return false;
  // First positional arg
  return arg === "dev";
});

// Build the tauri command
const tauriArgs = [...args];

if (isDevCommand) {
  // Inject port configuration for dev command
  const configOverride = JSON.stringify({
    build: {
      devUrl: `http://localhost:${port}`,
    },
  });

  // Add --config flag if not already present
  const hasConfig = args.some((arg) => arg === "--config" || arg.startsWith("--config="));
  if (!hasConfig) {
    tauriArgs.push("--config", configOverride);
  }

  // Log port info for visibility
  if (identifier) {
    console.log(`[tauri-wrapper] Worktree: ${identifier}`);
  }
  console.log(`[tauri-wrapper] Dev server port: ${port}`);
}

// Set VITE_DEV_PORT for Vite to use
const env = {
  ...process.env,
  VITE_DEV_PORT: String(port),
};

// Run tauri CLI via bunx to use local installation
const proc = Bun.spawn(["bunx", "tauri", ...tauriArgs], {
  env,
  stdout: "inherit",
  stderr: "inherit",
  stdin: "inherit",
});

// Wait for completion and exit with same code
const exitCode = await proc.exited;
process.exit(exitCode);

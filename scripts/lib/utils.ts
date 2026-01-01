/**
 * Shared utilities for heycat scripts.
 *
 * Provides common logging functions and color codes used across
 * multiple script files.
 */

// ANSI color codes for terminal output
export const colors = {
  reset: "\x1b[0m",
  bold: "\x1b[1m",
  red: "\x1b[31m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  blue: "\x1b[34m",
  cyan: "\x1b[36m",
  dim: "\x1b[2m",
};

export function log(message: string): void {
  console.log(message);
}

export function success(message: string): void {
  console.log(`${colors.green}${colors.bold}${message}${colors.reset}`);
}

export function error(message: string): void {
  console.error(`${colors.red}${colors.bold}Error: ${message}${colors.reset}`);
}

export function info(message: string): void {
  console.log(`${colors.cyan}${message}${colors.reset}`);
}

export function warn(message: string): void {
  console.log(`${colors.yellow}${message}${colors.reset}`);
}

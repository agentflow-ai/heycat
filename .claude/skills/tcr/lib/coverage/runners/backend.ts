import type { CoverageResult, CoverageMetrics } from "../types";
import { createEmptyMetrics, meetsThresholds } from "../types";
import { COVERAGE_CONFIG } from "../config";

// ============================================================================
// cargo-llvm-cov Detection
// ============================================================================

export async function checkCargoLlvmCovInstalled(): Promise<boolean> {
  const { $ } = await import("bun");
  try {
    await $`cargo llvm-cov --version`.quiet();
    return true;
  } catch (_error) {
    // cargo-llvm-cov not found or not executable
    return false;
  }
}

// ============================================================================
// cargo-llvm-cov JSON Output Parser
// ============================================================================

interface LlvmCovJsonOutput {
  data: Array<{
    totals: {
      lines: { count: number; covered: number; percent: number };
      functions: { count: number; covered: number; percent: number };
    };
  }>;
}

function parseLlvmCovJson(output: string): CoverageMetrics {
  try {
    const data: LlvmCovJsonOutput = JSON.parse(output);

    if (!data.data || data.data.length === 0 || !data.data[0].totals) {
      return createEmptyMetrics();
    }

    const totals = data.data[0].totals;

    return {
      lines: {
        covered: totals.lines.covered,
        total: totals.lines.count,
        percentage: totals.lines.percent / 100, // Convert from 0-100 to 0-1
      },
      functions: {
        covered: totals.functions.covered,
        total: totals.functions.count,
        percentage: totals.functions.percent / 100,
      },
    };
  } catch (_error) {
    // JSON parsing failed - return empty metrics
    return createEmptyMetrics();
  }
}

// ============================================================================
// Backend Coverage Runner
// ============================================================================

/**
 * Run backend coverage, optionally filtering to specific test modules.
 *
 * @param projectRoot - The project root directory
 * @param testModules - Optional array of test module filters (e.g., ["tests::", "foo::tests::"])
 *                      If provided, only tests matching these patterns will run.
 *                      If empty or undefined, all tests will run.
 */
export async function runBackendCoverage(
  projectRoot: string,
  testModules?: string[]
): Promise<CoverageResult> {
  const { $ } = await import("bun");
  const config = COVERAGE_CONFIG.backend;
  const tauriDir = `${projectRoot}/src-tauri`;

  // Check if cargo-llvm-cov is installed
  const hasLlvmCov = await checkCargoLlvmCovInstalled();

  if (!hasLlvmCov) {
    return {
      target: "backend",
      passed: false,
      metrics: createEmptyMetrics(),
      thresholds: config.thresholds,
      error: "cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov",
    };
  }

  try {
    // Run cargo +nightly llvm-cov with JSON output for parsing
    // Uses nightly for #[coverage(off)] attribute support
    // Untestable code is excluded via #[coverage(off)] in source files
    //
    // When testModules is provided, use `cargo llvm-cov test` with filters
    // to run only tests from changed modules
    let result;
    if (testModules && testModules.length > 0) {
      // Deduplicate modules
      const uniqueModules = [...new Set(testModules)];
      // Use test subcommand with filters - each filter runs tests matching that prefix
      result = await $`cargo +nightly llvm-cov test --json -- ${uniqueModules}`
        .cwd(tauriDir)
        .quiet()
        .nothrow();
    } else {
      // No filtering - run all tests
      result = await $`cargo +nightly llvm-cov --json`.cwd(tauriDir).quiet().nothrow();
    }

    const stdout = result.stdout.toString();
    const stderr = result.stderr.toString();

    // Check for test failures (non-zero exit code from tests themselves)
    if (result.exitCode !== 0) {
      // Try to parse coverage even on failure to show what we have
      const metrics = parseLlvmCovJson(stdout);

      return {
        target: "backend",
        passed: false,
        metrics,
        thresholds: config.thresholds,
        raw: stdout + stderr,
        error: `Tests failed with exit code ${result.exitCode}`,
      };
    }

    // Parse coverage metrics
    const metrics = parseLlvmCovJson(stdout);

    // Check if coverage meets thresholds
    const passed = meetsThresholds(metrics, config.thresholds);

    return {
      target: "backend",
      passed,
      metrics,
      thresholds: config.thresholds,
      raw: stdout,
      error: passed ? undefined : "Coverage below threshold",
    };
  } catch (error) {
    return {
      target: "backend",
      passed: false,
      metrics: createEmptyMetrics(),
      thresholds: config.thresholds,
      error: error instanceof Error ? error.message : "Unknown error running backend coverage",
    };
  }
}

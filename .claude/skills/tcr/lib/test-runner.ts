import { type TestTarget, type TestRunResult } from "./types";
import { runBackendCoverage, checkCargoLlvmCovInstalled } from "./coverage";
import type { CoverageResult } from "./coverage";
import { deriveRustTestModule } from "./utils";

// ============================================================================
// Frontend Test Runner (Vitest)
// ============================================================================

export async function runFrontendTests(
  testFiles: string[],
  projectRoot: string
): Promise<TestRunResult> {
  const { $ } = await import("bun");

  try {
    if (testFiles.length === 0) {
      return {
        status: "skip",
        output: "No frontend test files to run",
        exitCode: 0,
      };
    }

    // Run vitest with coverage (vitest.config.ts enforces 100% threshold)
    // Note: Bun's $ template literal auto-escapes interpolated values, preventing shell injection
    // Arrays are spread as separate properly-escaped arguments
    const result = await $`bun run test:coverage -- ${testFiles}`
      .cwd(projectRoot)
      .quiet()
      .nothrow();

    const output = result.stdout.toString() + result.stderr.toString();

    return {
      status: result.exitCode === 0 ? "pass" : "fail",
      output,
      exitCode: result.exitCode,
    };
  } catch (error) {
    return {
      status: "error",
      output: error instanceof Error ? error.message : "Unknown error running frontend tests",
      exitCode: 1,
    };
  }
}

// ============================================================================
// Backend Test Runner (Cargo with Coverage)
// ============================================================================

export interface BackendTestResult extends TestRunResult {
  coverage?: CoverageResult;
}

/**
 * Run backend tests with optional file-based filtering.
 *
 * @param projectRoot - The project root directory
 * @param changedFiles - Optional array of changed file paths. When provided,
 *                       only tests from modules corresponding to changed .rs files will run.
 */
export async function runBackendTests(
  projectRoot: string,
  changedFiles?: string[]
): Promise<BackendTestResult> {
  // Check if cargo-llvm-cov is installed (required)
  const hasLlvmCov = await checkCargoLlvmCovInstalled();

  if (!hasLlvmCov) {
    return {
      status: "error",
      output: "cargo-llvm-cov not installed. Install with: cargo install cargo-llvm-cov",
      exitCode: 1,
    };
  }

  // Derive test modules from changed files
  let testModules: string[] | undefined;
  if (changedFiles && changedFiles.length > 0) {
    testModules = changedFiles
      .map((f) => deriveRustTestModule(f))
      .filter((m): m is string => m !== null);

    // Deduplicate
    testModules = [...new Set(testModules)];
  }

  // Run coverage check (which also runs tests)
  const coverageResult = await runBackendCoverage(projectRoot, testModules);

  return {
    status: coverageResult.passed ? "pass" : "fail",
    output: coverageResult.raw || coverageResult.error || "",
    exitCode: coverageResult.passed ? 0 : 1,
    coverage: coverageResult,
  };
}

// ============================================================================
// Combined Test Runner
// ============================================================================

export interface CombinedTestResult {
  passed: boolean;
  frontend: TestRunResult | null;
  backend: TestRunResult | null;
  error: string | null;
}

/**
 * Run tests for the specified target with file-based filtering.
 *
 * @param target - Which target(s) to run tests for
 * @param testFiles - Frontend test files to run (filtered from changed files)
 * @param projectRoot - The project root directory
 * @param changedFiles - Optional array of all changed source files (used for backend filtering)
 */
export async function runTests(
  target: TestTarget,
  testFiles: string[],
  projectRoot: string,
  changedFiles?: string[]
): Promise<CombinedTestResult> {
  const result: CombinedTestResult = {
    passed: true,
    frontend: null,
    backend: null,
    error: null,
  };

  // Run frontend tests
  if (target === "frontend" || target === "both") {
    const frontendFiles = testFiles.filter(
      (f) => f.endsWith(".ts") || f.endsWith(".tsx") || f.endsWith(".js") || f.endsWith(".jsx")
    );

    result.frontend = await runFrontendTests(frontendFiles, projectRoot);

    if (result.frontend.status === "fail" || result.frontend.status === "error") {
      result.passed = false;
      result.error = result.frontend.output;
    }
  }

  // Run backend tests - pass changed files for module-based filtering
  if (target === "backend" || target === "both") {
    result.backend = await runBackendTests(projectRoot, changedFiles);

    if (result.backend.status === "fail" || result.backend.status === "error") {
      result.passed = false;
      if (!result.error) {
        result.error = result.backend.output;
      } else {
        result.error += "\n\n" + result.backend.output;
      }
    }
  }

  return result;
}

// ============================================================================
// Output Formatting
// ============================================================================

export function formatTestOutput(result: CombinedTestResult): string {
  const lines: string[] = [];

  if (result.frontend) {
    lines.push("=== Frontend Tests ===");
    lines.push(`Status: ${result.frontend.status}`);
    if (result.frontend.output) {
      lines.push(result.frontend.output);
    }
  }

  if (result.backend) {
    lines.push("");
    lines.push("=== Backend Tests ===");
    lines.push(`Status: ${result.backend.status}`);
    if (result.backend.output) {
      lines.push(result.backend.output);
    }
  }

  return lines.join("\n");
}

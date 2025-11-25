import type { CombinedCoverageResult, CoverageResult } from "./types";
import { formatPercentage } from "./types";
import type { TestTarget } from "../types";
import { runFrontendCoverage, runFrontendCoverageAll } from "./runners/frontend";
import { runBackendCoverage } from "./runners/backend";

// ============================================================================
// Re-exports
// ============================================================================

export { COVERAGE_CONFIG, FRONTEND_THRESHOLDS, BACKEND_THRESHOLDS, getThresholdPercentage } from "./config";
export { checkCargoLlvmCovInstalled, runBackendCoverage } from "./runners/backend";
export { runFrontendCoverage, runFrontendCoverageAll } from "./runners/frontend";
export type {
  CoverageConfig,
  CoverageMetrics,
  CoverageResult,
  CoverageTarget,
  CoverageThresholds,
  CombinedCoverageResult,
} from "./types";
export { createEmptyMetrics, formatPercentage, meetsThresholds } from "./types";

// ============================================================================
// Combined Coverage Checks
// ============================================================================

/**
 * Run coverage checks for the specified target(s).
 *
 * @param target - Which target to check: "frontend", "backend", or "both"
 * @param testFiles - Test files to run (for frontend)
 * @param projectRoot - Project root directory
 * @returns Combined coverage results with summary
 */
export async function runCoverageChecks(
  target: TestTarget,
  testFiles: string[],
  projectRoot: string
): Promise<CombinedCoverageResult> {
  const result: CombinedCoverageResult = {
    passed: true,
    frontend: null,
    backend: null,
    summary: "",
  };

  // Run frontend coverage
  if (target === "frontend" || target === "both") {
    const frontendFiles = testFiles.filter(
      (f) => f.endsWith(".ts") || f.endsWith(".tsx") || f.endsWith(".js") || f.endsWith(".jsx")
    );

    result.frontend = await runFrontendCoverage(frontendFiles, projectRoot);

    if (!result.frontend.passed) {
      result.passed = false;
    }
  }

  // Run backend coverage
  if (target === "backend" || target === "both") {
    result.backend = await runBackendCoverage(projectRoot);

    if (!result.backend.passed) {
      result.passed = false;
    }
  }

  // Generate summary
  result.summary = formatCoverageReport(result);

  return result;
}

/**
 * Run coverage checks for all tests (not specific files).
 * Used by the coverage command and status display.
 */
export async function runAllCoverageChecks(
  target: TestTarget,
  projectRoot: string
): Promise<CombinedCoverageResult> {
  const result: CombinedCoverageResult = {
    passed: true,
    frontend: null,
    backend: null,
    summary: "",
  };

  if (target === "frontend" || target === "both") {
    result.frontend = await runFrontendCoverageAll(projectRoot);
    if (!result.frontend.passed) {
      result.passed = false;
    }
  }

  if (target === "backend" || target === "both") {
    result.backend = await runBackendCoverage(projectRoot);
    if (!result.backend.passed) {
      result.passed = false;
    }
  }

  result.summary = formatCoverageReport(result);

  return result;
}

// ============================================================================
// Coverage Report Formatting
// ============================================================================

function formatCoverageReport(result: CombinedCoverageResult): string {
  const lines: string[] = [];

  lines.push("=".repeat(60));
  lines.push("                    COVERAGE REPORT");
  lines.push("=".repeat(60));

  if (result.frontend) {
    lines.push("");
    lines.push(formatTargetReport("Frontend (Bun)", result.frontend));
  }

  if (result.backend) {
    lines.push("");
    lines.push(formatTargetReport("Backend (Rust)", result.backend));
  }

  lines.push("");
  lines.push("=".repeat(60));
  lines.push(`Overall: ${result.passed ? "PASS" : "FAIL"}`);
  lines.push("=".repeat(60));

  return lines.join("\n");
}

function formatTargetReport(label: string, result: CoverageResult): string {
  const { metrics, thresholds, error } = result;
  const lines: string[] = [];

  const statusSymbol = result.passed ? "PASS" : "FAIL";

  lines.push(`--- ${label} ---`);
  lines.push(
    `  Lines:     ${formatPercentage(metrics.lines.percentage)} (threshold: ${formatPercentage(thresholds.lines)})`
  );
  lines.push(
    `  Functions: ${formatPercentage(metrics.functions.percentage)} (threshold: ${formatPercentage(thresholds.functions)})`
  );
  lines.push(`  Status:    ${statusSymbol}`);

  if (error) {
    lines.push(`  Error:     ${error}`);
  }

  return lines.join("\n");
}

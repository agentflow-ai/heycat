import { loadState } from "../lib/state";
import { findProjectRoot, readErrorLog } from "../lib/utils";
import { MAX_FAILURES } from "../lib/types";
import { runAllCoverageChecks, formatPercentage } from "../lib/coverage";

export async function handleStatus(args: string[]): Promise<void> {
  const projectRoot = await findProjectRoot();
  const state = await loadState(projectRoot);

  // Check if --coverage flag is passed
  const showCoverage = args.includes("--coverage") || args.includes("-c");

  console.log("\n=== TCR Status ===\n");

  // Current step
  if (state.currentStep) {
    console.log(`Current Step: ${state.currentStep}`);
  } else {
    console.log("Current Step: None (no active task)");
  }

  // Failure count
  const failureBar = "█".repeat(state.failureCount) + "░".repeat(MAX_FAILURES - state.failureCount);
  console.log(`Failures: [${failureBar}] ${state.failureCount}/${MAX_FAILURES}`);

  if (state.failureCount >= MAX_FAILURES) {
    console.log("  Warning: Threshold reached - consider a different approach");
    console.log('  Run "tcr reset" to continue');
  }

  // Last test result
  console.log("");
  if (state.lastTestResult) {
    const { passed, timestamp, error, target } = state.lastTestResult;
    const status = passed ? "PASS" : "FAIL";
    const time = new Date(timestamp).toLocaleString();

    console.log(`Last Test: ${status}`);
    console.log(`  Time: ${time}`);
    console.log(`  Target: ${target}`);

    if (error && !passed) {
      console.log(`  Error: ${error}`);
    }
  } else {
    console.log("Last Test: No tests run yet");
  }

  // Coverage section (optional, runs tests to get current coverage)
  if (showCoverage) {
    console.log("");
    console.log("=== Coverage ===");
    console.log("(Running tests to collect coverage...)\n");

    const coverage = await runAllCoverageChecks("both", projectRoot);

    if (coverage.frontend) {
      const { metrics, thresholds, passed } = coverage.frontend;
      const statusStr = passed ? "PASS" : "FAIL";
      console.log(`Frontend: ${formatPercentage(metrics.lines.percentage)} lines, ${formatPercentage(metrics.functions.percentage)} functions (${statusStr}, threshold: ${formatPercentage(thresholds.lines)})`);
    }

    if (coverage.backend) {
      const { metrics, thresholds, passed, error } = coverage.backend;
      const statusStr = passed ? "PASS" : "FAIL";
      if (error && error.includes("cargo-llvm-cov not installed")) {
        console.log(`Backend:  cargo-llvm-cov not installed`);
      } else {
        console.log(`Backend:  ${formatPercentage(metrics.lines.percentage)} lines, ${formatPercentage(metrics.functions.percentage)} functions (${statusStr}, threshold: ${formatPercentage(thresholds.lines)})`);
      }
    }
  } else {
    console.log("");
    console.log("Tip: Run 'tcr status --coverage' to see current coverage metrics");
  }

  // Show recent errors from error log
  const errors = await readErrorLog(projectRoot);
  if (errors.length > 0) {
    console.log("");
    console.log("=== Recent Errors ===");
    console.log("(from .tcr-errors.log)\n");

    // Show only the most recent 3 errors
    const recentErrors = errors.slice(-3);
    for (const entry of recentErrors) {
      const time = new Date(entry.timestamp).toLocaleString();
      const context = entry.context ? ` [${entry.context}]` : "";

      console.log(`  ${time}${context}`);
      console.log(`    ${entry.error}`);
      console.log("");
    }

    console.log("  Run 'tcr reset' to clear errors along with failure count");
  }

  console.log("");
}

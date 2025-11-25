import { findProjectRoot } from "../lib/utils";
import { runAllCoverageChecks } from "../lib/coverage";
import type { TestTarget } from "../lib/types";

/**
 * Handle the `tcr coverage` command.
 *
 * Usage:
 *   tcr coverage [target]
 *
 * Where target is one of:
 *   - frontend  Run frontend coverage only
 *   - backend   Run backend coverage only
 *   - both      Run both (default)
 */
export async function handleCoverage(args: string[]): Promise<void> {
  const projectRoot = await findProjectRoot();

  // Parse target argument
  const targetArg = args[0]?.toLowerCase();
  let target: TestTarget = "both";

  if (targetArg === "frontend" || targetArg === "backend" || targetArg === "both") {
    target = targetArg;
  } else if (targetArg && targetArg !== "") {
    console.error(`Unknown target: ${targetArg}`);
    console.error("Valid targets: frontend, backend, both");
    process.exit(1);
  }

  console.log(`\nRunning ${target} coverage checks...\n`);

  const result = await runAllCoverageChecks(target, projectRoot);

  // Print the coverage report
  console.log(result.summary);

  // Exit with appropriate code
  process.exit(result.passed ? 0 : 1);
}

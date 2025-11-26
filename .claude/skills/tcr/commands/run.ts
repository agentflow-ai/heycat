import { findProjectRoot, findTestFiles, determineTarget } from "../lib/utils";
import { runTests, formatTestOutput } from "../lib/test-runner";
import { recordTestResult } from "../lib/state";
import { getCurrentTimestamp } from "../lib/utils";
import type { TestResult } from "../lib/types";

export async function handleRun(args: string[]): Promise<void> {
  if (args.length === 0) {
    console.error("Usage: tcr run <files...>");
    console.error("Example: tcr run src/App.tsx src/utils/auth.ts");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();

  // Determine target based on source files first (needed for early exit logic)
  const target = determineTarget(args);

  // Find test files for the given source files (frontend convention: foo.ts → foo.test.ts)
  const testFiles = await findTestFiles(args, projectRoot);

  // For frontend, we need discovered test files. For backend, tests are inline in source files.
  if (testFiles.length === 0 && target === "frontend") {
    console.log("No test files found for the specified source files.");
    console.log("Convention: foo.ts → foo.test.ts or foo.spec.ts");
    return;
  }

  if (testFiles.length > 0) {
    console.log(`Found ${testFiles.length} frontend test file(s):`);
    for (const file of testFiles) {
      console.log(`  - ${file}`);
    }
    console.log("");
  }

  if (target === "backend" || target === "both") {
    console.log("Backend tests are inline in source files (Rust #[cfg(test)] modules)");
    console.log("");
  }

  console.log(`Running ${target} tests...`);
  console.log("");

  // Run tests - pass source files for backend module filtering
  const result = await runTests(target, testFiles, projectRoot, args);

  // Display output
  console.log(formatTestOutput(result));

  // Record result
  // For backend, include source files since tests are inline
  const filesRun = target === "backend" ? args : testFiles.length > 0 ? testFiles : args;
  const testResult: TestResult = {
    passed: result.passed,
    timestamp: getCurrentTimestamp(),
    error: result.error,
    filesRun,
    target,
  };
  await recordTestResult(projectRoot, testResult);

  // Exit with appropriate code
  if (result.passed) {
    console.log("\n✅ All tests passed");
  } else {
    console.log("\n❌ Tests failed");
    process.exit(2);
  }
}

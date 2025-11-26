import {
  loadState,
  setCurrentStep,
  incrementFailure,
  recordTestResult,
  hasReachedFailureThreshold,
} from "../lib/state";
import {
  findProjectRoot,
  getChangedFiles,
  findTestFiles,
  determineTarget,
  createWipCommit,
  readStdin,
  getCurrentTimestamp,
  logError,
} from "../lib/utils";
import { runTests } from "../lib/test-runner";
import {
  MAX_FAILURES,
  type HookInput,
  type TodoWriteInput,
  type TestResult,
} from "../lib/types";

/**
 * PostToolUse hook handler for TodoWrite
 *
 * Triggered when TodoWrite tool completes. If a todo was marked "completed",
 * runs tests on changed files and auto-commits if they pass.
 */
export async function handleHookTodoComplete(): Promise<void> {
  try {
    // Read hook input from stdin
    const input = await readStdin<HookInput>();

    // Verify this is a TodoWrite event
    if (input.tool_name !== "TodoWrite") {
      process.exit(0);
    }

    const todoInput = input.tool_input as unknown as TodoWriteInput;
    const projectRoot = await findProjectRoot();

    // Find newly completed todos
    const completedTodos = todoInput.todos.filter((t) => t.status === "completed");

    if (completedTodos.length === 0) {
      // No completed todos, nothing to do
      process.exit(0);
    }

    // Get the most recently completed todo (last in list)
    const completedTodo = completedTodos[completedTodos.length - 1];
    const stepId = completedTodo.content;

    console.log(`TCR: Todo completed - "${stepId}"`);

    // Update current step (resets failure count if step changed)
    await setCurrentStep(projectRoot, stepId);

    // Get changed files from git diff
    const changedFiles = await getChangedFiles(false);

    if (changedFiles.length === 0) {
      console.log("TCR: No changed files detected, skipping tests");
      process.exit(0);
    }

    console.log(`TCR: Found ${changedFiles.length} changed file(s)`);

    // Determine test target (frontend, backend, or both) - needed for early exit logic
    const target = determineTarget(changedFiles);

    // Find test files for changed source files (frontend convention: foo.ts → foo.test.ts)
    const testFiles = await findTestFiles(changedFiles, projectRoot);

    // For frontend, we need discovered test files. For backend, tests are inline in source files.
    if (testFiles.length === 0 && target === "frontend") {
      // TCR principle: Don't auto-commit untested code
      console.warn("TCR: No test files found for changed files.");
      console.warn("  Changed files:", changedFiles.join(", "));
      console.warn("  Skipping auto-commit. Write tests first, or commit manually.");
      process.exit(0);
    }

    const testCount = target === "backend" ? changedFiles.filter(f => f.endsWith(".rs")).length : testFiles.length;
    console.log(`TCR: Running ${target} tests (${testCount} file(s))...`);

    // Run tests - pass changed files for backend module filtering
    const result = await runTests(target, testFiles, projectRoot, changedFiles);

    // Record test result
    // For backend, include source files since tests are inline
    const filesRun = target === "backend" ? changedFiles.filter(f => f.endsWith(".rs")) : testFiles.length > 0 ? testFiles : changedFiles;
    const testResult: TestResult = {
      passed: result.passed,
      timestamp: getCurrentTimestamp(),
      error: result.error,
      filesRun,
      target,
    };
    await recordTestResult(projectRoot, testResult);

    if (result.passed) {
      // Tests passed - create WIP commit
      console.log("TCR: Tests passed!");

      const hash = await createWipCommit(stepId);
      if (hash) {
        console.log(`TCR: Created WIP commit (${hash})`);
      }
    } else {
      // Tests failed - increment failure counter with context
      const failureCount = await incrementFailure(
        projectRoot,
        result.error || "Tests failed",
        testFiles,
        target
      );

      console.error(`TCR: Tests failed (${failureCount}/${MAX_FAILURES})`);

      // Show full error output
      if (result.error) {
        console.error(result.error);
      }

      // Check if threshold reached
      const state = await loadState(projectRoot);
      if (hasReachedFailureThreshold(state)) {
        console.error("");
        console.error(`TCR: ${MAX_FAILURES} consecutive failures reached!`);
        console.error("Consider:");
        console.error("  1. Breaking down the task into smaller pieces");
        console.error("  2. Reviewing the test expectations");
        console.error("  3. Taking a different approach");
        console.error('  4. Run "bun .claude/skills/tcr/tcr.ts reset" to continue');
      }

      // Exit with code 2 to block agent - stderr is fed to Claude
      process.exit(2);
    }

    process.exit(0);
  } catch (error) {
    // Fail open - don't block Claude Code on hook errors
    // But log detailed error info for debugging and persist to error log
    const errorMessage =
      error instanceof Error ? error.stack || error.message : "Unknown error";
    console.error("TCR hook error:", errorMessage);

    // Persist error to log file for later inspection via `tcr status`
    try {
      const projectRoot = await findProjectRoot();
      await logError(projectRoot, errorMessage, "hook-todo-complete");
    } catch {
      // If we can't even find project root, just continue
    }

    process.exit(0);
  }
}

import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { STATE_FILE, MAX_FAILURES, TEST_TARGETS, type TCRState, type TestResult, type TestTarget } from "./types";
import { fileExists, getCurrentTimestamp } from "./utils";

// ============================================================================
// Default State
// ============================================================================

function createDefaultState(): TCRState {
  return {
    currentStep: null,
    failureCount: 0,
    lastTestResult: null,
  };
}

// ============================================================================
// State Validation
// ============================================================================

/**
 * Validate that a parsed object conforms to TCRState shape.
 * Returns true if valid, false if corrupted or wrong format.
 */
function isValidState(obj: unknown): obj is TCRState {
  if (typeof obj !== "object" || obj === null) {
    return false;
  }

  const state = obj as Record<string, unknown>;

  // Check required fields exist and have correct types
  if (!("currentStep" in state) || !("failureCount" in state)) {
    return false;
  }

  // currentStep should be string or null
  if (state.currentStep !== null && typeof state.currentStep !== "string") {
    return false;
  }

  // failureCount should be a non-negative number
  if (typeof state.failureCount !== "number" || state.failureCount < 0) {
    return false;
  }

  // Validate lastTestResult if present
  if (state.lastTestResult !== null && state.lastTestResult !== undefined) {
    const result = state.lastTestResult as Record<string, unknown>;

    // Check required TestResult fields
    if (typeof result.passed !== "boolean") return false;
    if (typeof result.timestamp !== "string") return false;
    if (!TEST_TARGETS.includes(result.target as TestTarget)) return false;
    if (!Array.isArray(result.filesRun)) return false;
    // error can be string or null
    if (result.error !== null && typeof result.error !== "string") return false;
  }

  return true;
}

// ============================================================================
// State Persistence
// ============================================================================

export async function loadState(projectRoot: string): Promise<TCRState> {
  const statePath = join(projectRoot, STATE_FILE);

  try {
    if (await fileExists(statePath)) {
      const content = await readFile(statePath, "utf-8");
      const parsed = JSON.parse(content);

      if (!isValidState(parsed)) {
        console.warn("TCR: Invalid state file, resetting to defaults");
        return createDefaultState();
      }

      return parsed;
    }
  } catch (error) {
    console.warn(
      "TCR: Error loading state file:",
      error instanceof Error ? error.message : "Unknown error"
    );
  }

  return createDefaultState();
}

export async function saveState(projectRoot: string, state: TCRState): Promise<void> {
  const statePath = join(projectRoot, STATE_FILE);
  await writeFile(statePath, JSON.stringify(state, null, 2), "utf-8");
}

// ============================================================================
// State Mutations
// ============================================================================

export async function setCurrentStep(projectRoot: string, step: string): Promise<void> {
  const state = await loadState(projectRoot);

  // If step changed, reset failure count
  if (state.currentStep !== step) {
    state.currentStep = step;
    state.failureCount = 0;
  }

  await saveState(projectRoot, state);
}

export async function incrementFailure(
  projectRoot: string,
  error: string,
  filesRun: string[] = [],
  target: TestTarget = "frontend"
): Promise<number> {
  const state = await loadState(projectRoot);

  state.failureCount += 1;
  state.lastTestResult = {
    passed: false,
    timestamp: getCurrentTimestamp(),
    error,
    filesRun,
    target,
  };

  await saveState(projectRoot, state);
  return state.failureCount;
}

export async function resetFailures(projectRoot: string): Promise<void> {
  const state = await loadState(projectRoot);
  state.failureCount = 0;
  await saveState(projectRoot, state);
}

export async function recordTestResult(
  projectRoot: string,
  result: TestResult
): Promise<void> {
  const state = await loadState(projectRoot);

  state.lastTestResult = result;

  // Reset failure count on success
  if (result.passed) {
    state.failureCount = 0;
  }

  await saveState(projectRoot, state);
}

// ============================================================================
// State Queries
// ============================================================================

export function hasReachedFailureThreshold(state: TCRState): boolean {
  return state.failureCount >= MAX_FAILURES;
}

export function getFailureCount(state: TCRState): number {
  return state.failureCount;
}

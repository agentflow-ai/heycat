// ============================================================================
// Test Status Types
// ============================================================================

export const TEST_STATUSES = ["pass", "fail", "error", "skip"] as const;
export type TestStatus = (typeof TEST_STATUSES)[number];

// ============================================================================
// Test Target Types
// ============================================================================

export const TEST_TARGETS = ["frontend", "backend", "both"] as const;
export type TestTarget = (typeof TEST_TARGETS)[number];

// ============================================================================
// Constants
// ============================================================================

export const MAX_FAILURES = 5;
export const WIP_PREFIX = "WIP: ";
export const STATE_FILE = ".tcr-state.json";

// Frontend file extensions
export const FRONTEND_EXTENSIONS = [".ts", ".tsx", ".js", ".jsx"] as const;

// Backend file extensions
export const BACKEND_EXTENSIONS = [".rs"] as const;

// Report formatting
export const FORMATTING = {
  separatorWidth: 60,
} as const;

// ============================================================================
// State Interfaces
// ============================================================================

export interface TestResult {
  passed: boolean;
  timestamp: string;
  error: string | null;
  filesRun: string[];
  target: TestTarget;
}

export interface TCRState {
  currentStep: string | null;
  failureCount: number;
  lastTestResult: TestResult | null;
}

// ============================================================================
// Hook Input Interfaces
// ============================================================================

export interface HookInput {
  session_id: string;
  hook_event_name: string;
  tool_name: string;
  tool_input: Record<string, unknown>;
  tool_response?: Record<string, unknown>;
  cwd: string;
}

export interface TodoItem {
  content: string;
  status: "pending" | "in_progress" | "completed";
  activeForm: string;
}

export interface TodoWriteInput {
  todos: TodoItem[];
}

export interface BashInput {
  command: string;
  description?: string;
}

// ============================================================================
// Test Runner Interfaces
// ============================================================================

export interface TestRunResult {
  status: TestStatus;
  output: string;
  exitCode: number;
}

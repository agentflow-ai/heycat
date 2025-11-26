import type { CoverageConfig, CoverageThresholds } from "./types";

// ============================================================================
// COVERAGE THRESHOLDS - MANUAL SYNC REQUIRED
// ============================================================================

/**
 * ============================================================
 * IMPORTANT: THREE-WAY SYNC REQUIREMENT
 * ============================================================
 *
 * Coverage is enforced in THREE places that must stay in sync:
 *
 * 1. THIS FILE (.claude/skills/tcr/lib/coverage/config.ts)
 *    - Purpose: TCR status display and coverage reporting
 *    - Values: FRONTEND_THRESHOLDS, BACKEND_THRESHOLDS (both 100%)
 *
 * 2. vitest.config.ts (project root)
 *    - Purpose: Enforces frontend thresholds at test time
 *    - Location: coverage.thresholds.lines, coverage.thresholds.functions
 *    - Current: { lines: 100, functions: 100 }
 *
 * 3. .husky/pre-commit
 *    - Purpose: Enforces backend thresholds via cargo llvm-cov
 *    - Flags: --fail-under-lines 100 --fail-under-functions 100
 *
 * ============================================================
 * IF YOU CHANGE THRESHOLDS, UPDATE ALL THREE LOCATIONS!
 * ============================================================
 *
 * 100% coverage is required for all testable code.
 * Untestable code must be explicitly excluded:
 *
 * Frontend (TypeScript/React):
 *   Single line:    // v8 ignore next
 *   Multi-line:     // v8 ignore start ... // v8 ignore stop
 *
 * Backend (Rust):
 *   Attribute:      #[cfg_attr(coverage_nightly, coverage(off))]
 */

export const FRONTEND_THRESHOLDS: CoverageThresholds = {
  lines: 1.0, // 100%
  functions: 1.0, // 100%
};

export const BACKEND_THRESHOLDS: CoverageThresholds = {
  lines: 1.0, // 100% (untestable code excluded via #[coverage(off)])
  functions: 1.0, // 100%
};

export const COVERAGE_CONFIG: Record<"frontend" | "backend", CoverageConfig> = {
  frontend: {
    enabled: true,
    thresholds: FRONTEND_THRESHOLDS,
  },
  backend: {
    enabled: true,
    thresholds: BACKEND_THRESHOLDS,
  },
};

// ============================================================================
// Helper Functions
// ============================================================================

export function getThresholdPercentage(target: "frontend" | "backend", metric: "lines" | "functions"): number {
  return COVERAGE_CONFIG[target].thresholds[metric] * 100;
}

export function isTargetEnabled(target: "frontend" | "backend"): boolean {
  return COVERAGE_CONFIG[target].enabled;
}

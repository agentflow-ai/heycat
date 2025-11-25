import type { CoverageConfig, CoverageThresholds } from "./types";

// ============================================================================
// Coverage Configuration
// ============================================================================

/**
 * Centralized coverage configuration for both frontend and backend.
 * This is the single source of truth for coverage thresholds.
 *
 * 100% coverage is required for all testable code.
 * Untestable code must be explicitly excluded using #[coverage(off)] attribute.
 *
 * Note: Frontend thresholds should match bunfig.toml for consistency.
 * Backend uses cargo +nightly llvm-cov for #[coverage(off)] support.
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

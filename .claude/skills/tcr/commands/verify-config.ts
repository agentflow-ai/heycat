import { findProjectRoot } from "../lib/utils";
import {
  FRONTEND_THRESHOLDS,
  BACKEND_THRESHOLDS,
} from "../lib/coverage/config";

// ============================================================================
// Configuration Parsers
// ============================================================================

interface ThresholdConfig {
  lines: number; // 0-100 scale
  functions: number; // 0-100 scale
}

interface ConfigSource {
  name: string;
  file: string;
  frontend: ThresholdConfig | null;
  backend: ThresholdConfig | null;
}

/**
 * Parse vitest.config.ts to extract coverage thresholds.
 * Looks for the thresholds object in the coverage config.
 */
async function parseVitestConfig(
  projectRoot: string
): Promise<ThresholdConfig | null> {
  const configPath = `${projectRoot}/vitest.config.ts`;

  try {
    const file = Bun.file(configPath);
    if (!(await file.exists())) {
      return null;
    }

    const content = await file.text();

    // Parse lines threshold
    const linesMatch = content.match(/lines:\s*(\d+)/);
    const functionsMatch = content.match(/functions:\s*(\d+)/);

    if (!linesMatch || !functionsMatch) {
      return null;
    }

    return {
      lines: parseInt(linesMatch[1], 10),
      functions: parseInt(functionsMatch[1], 10),
    };
  } catch {
    return null;
  }
}

/**
 * Parse .husky/pre-commit to extract backend coverage thresholds.
 * Looks for --fail-under-lines and --fail-under-functions flags.
 */
async function parseHuskyConfig(
  projectRoot: string
): Promise<ThresholdConfig | null> {
  const configPath = `${projectRoot}/.husky/pre-commit`;

  try {
    const file = Bun.file(configPath);
    if (!(await file.exists())) {
      return null;
    }

    const content = await file.text();

    // Parse --fail-under-lines and --fail-under-functions
    const linesMatch = content.match(/--fail-under-lines\s+(\d+)/);
    const functionsMatch = content.match(/--fail-under-functions\s+(\d+)/);

    if (!linesMatch || !functionsMatch) {
      return null;
    }

    return {
      lines: parseInt(linesMatch[1], 10),
      functions: parseInt(functionsMatch[1], 10),
    };
  } catch {
    return null;
  }
}

/**
 * Get TCR config thresholds (already in 0-1 scale, convert to 0-100).
 */
function getTcrConfig(): { frontend: ThresholdConfig; backend: ThresholdConfig } {
  return {
    frontend: {
      lines: FRONTEND_THRESHOLDS.lines * 100,
      functions: FRONTEND_THRESHOLDS.functions * 100,
    },
    backend: {
      lines: BACKEND_THRESHOLDS.lines * 100,
      functions: BACKEND_THRESHOLDS.functions * 100,
    },
  };
}

// ============================================================================
// Verification Logic
// ============================================================================

interface VerificationResult {
  passed: boolean;
  mismatches: string[];
  sources: ConfigSource[];
}

async function verifyConfig(): Promise<VerificationResult> {
  const projectRoot = await findProjectRoot();
  const mismatches: string[] = [];

  // Gather all configurations
  const tcrConfig = getTcrConfig();
  const vitestConfig = await parseVitestConfig(projectRoot);
  const huskyConfig = await parseHuskyConfig(projectRoot);

  const sources: ConfigSource[] = [
    {
      name: "TCR Config",
      file: ".claude/skills/tcr/lib/coverage/config.ts",
      frontend: tcrConfig.frontend,
      backend: tcrConfig.backend,
    },
    {
      name: "Vitest Config",
      file: "vitest.config.ts",
      frontend: vitestConfig,
      backend: null, // Vitest only handles frontend
    },
    {
      name: "Husky Pre-commit",
      file: ".husky/pre-commit",
      frontend: null, // Husky backend section only
      backend: huskyConfig,
    },
  ];

  // Check for missing configs
  if (!vitestConfig) {
    mismatches.push("Could not parse vitest.config.ts coverage thresholds");
  }
  if (!huskyConfig) {
    mismatches.push("Could not parse .husky/pre-commit coverage thresholds");
  }

  // Compare frontend thresholds (TCR vs Vitest)
  if (vitestConfig) {
    if (tcrConfig.frontend.lines !== vitestConfig.lines) {
      mismatches.push(
        `Frontend lines threshold mismatch: TCR=${tcrConfig.frontend.lines}%, Vitest=${vitestConfig.lines}%`
      );
    }
    if (tcrConfig.frontend.functions !== vitestConfig.functions) {
      mismatches.push(
        `Frontend functions threshold mismatch: TCR=${tcrConfig.frontend.functions}%, Vitest=${vitestConfig.functions}%`
      );
    }
  }

  // Compare backend thresholds (TCR vs Husky)
  if (huskyConfig) {
    if (tcrConfig.backend.lines !== huskyConfig.lines) {
      mismatches.push(
        `Backend lines threshold mismatch: TCR=${tcrConfig.backend.lines}%, Husky=${huskyConfig.lines}%`
      );
    }
    if (tcrConfig.backend.functions !== huskyConfig.functions) {
      mismatches.push(
        `Backend functions threshold mismatch: TCR=${tcrConfig.backend.functions}%, Husky=${huskyConfig.functions}%`
      );
    }
  }

  return {
    passed: mismatches.length === 0,
    mismatches,
    sources,
  };
}

// ============================================================================
// Command Handler
// ============================================================================

export async function handleVerifyConfig(): Promise<void> {
  console.log("TCR: Verifying coverage configuration sync...");
  console.log("");

  const result = await verifyConfig();

  // Display sources and their values
  console.log("Configuration sources:");
  console.log("─".repeat(60));

  for (const source of result.sources) {
    console.log(`\n  ${source.name} (${source.file})`);
    if (source.frontend) {
      console.log(
        `    Frontend: lines=${source.frontend.lines}%, functions=${source.frontend.functions}%`
      );
    }
    if (source.backend) {
      console.log(
        `    Backend:  lines=${source.backend.lines}%, functions=${source.backend.functions}%`
      );
    }
    if (!source.frontend && !source.backend) {
      console.log("    (could not parse)");
    }
  }

  console.log("");
  console.log("─".repeat(60));

  if (result.passed) {
    console.log("✓ All coverage thresholds are in sync!");
    process.exit(0);
  } else {
    console.log("✗ Configuration mismatches found:");
    console.log("");
    for (const mismatch of result.mismatches) {
      console.log(`  • ${mismatch}`);
    }
    console.log("");
    console.log("Fix these mismatches to ensure consistent coverage enforcement.");
    process.exit(1);
  }
}

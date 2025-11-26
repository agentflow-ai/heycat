import { stat } from "node:fs/promises";
import { join, dirname } from "node:path";
import {
  STAGES,
  TEMPLATES,
  AGILE_DIR,
  SLUG_PATTERN,
  type Stage,
  type Template,
} from "./types";

// ============================================================================
// Validation Functions
// ============================================================================

export function isValidStage(stage: string): stage is Stage {
  return STAGES.includes(stage as Stage);
}

export function isValidTemplate(template: string): template is Template {
  return TEMPLATES.includes(template as Template);
}

export function validateSlug(slug: string): void {
  if (!SLUG_PATTERN.test(slug)) {
    throw new Error(
      `Invalid name: "${slug}". Use kebab-case (lowercase letters, numbers, hyphens)`
    );
  }
}

// ============================================================================
// String Conversion Functions
// ============================================================================

export function toKebabCase(str: string): string {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export function toTitleCase(str: string): string {
  return str
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

// ============================================================================
// Date Functions
// ============================================================================

export function getCurrentDate(): string {
  return new Date().toISOString().split("T")[0];
}

// ============================================================================
// File System Functions
// ============================================================================

export async function findProjectRoot(): Promise<string> {
  let dir = process.cwd();
  while (dir !== "/") {
    try {
      await stat(join(dir, AGILE_DIR));
      return dir;
    } catch {
      dir = dirname(dir);
    }
  }
  throw new Error("Could not find project root (no agile/ directory found)");
}

// ============================================================================
// Stage Types and Constants
// ============================================================================

export const STAGES = ["1-backlog", "2-todo", "3-in-progress", "4-review", "5-done"] as const;
export type Stage = (typeof STAGES)[number];

export const STAGE_NAMES: Record<Stage, string> = {
  "1-backlog": "Backlog",
  "2-todo": "Todo",
  "3-in-progress": "In Progress",
  "4-review": "Review",
  "5-done": "Done",
};

// Strict sequential transitions only
export const VALID_TRANSITIONS: Record<Stage, Stage[]> = {
  "1-backlog": ["2-todo"],
  "2-todo": ["1-backlog", "3-in-progress"],
  "3-in-progress": ["2-todo", "4-review"],
  "4-review": ["3-in-progress", "5-done"],
  "5-done": ["4-review"],
};

// ============================================================================
// Template Types and Constants
// ============================================================================

export const TEMPLATES = ["feature", "bug", "task"] as const;
export type Template = (typeof TEMPLATES)[number];

// ============================================================================
// Spec Types and Constants
// ============================================================================

export const SPEC_STATUSES = ["pending", "in-progress", "completed"] as const;
export type SpecStatus = (typeof SPEC_STATUSES)[number];

export interface SpecFrontmatter {
  status: SpecStatus;
  created: string;
  completed: string | null;
  dependencies: string[];
}

export interface SpecInfo {
  name: string;           // Filename without .spec.md
  path: string;           // Full path to spec file
  frontmatter: SpecFrontmatter;
  title: string;          // Title from content (# Spec: Title)
}

// ============================================================================
// Technical Guidance Types
// ============================================================================

export const GUIDANCE_STATUSES = ["draft", "active", "finalized"] as const;
export type GuidanceStatus = (typeof GUIDANCE_STATUSES)[number];

export interface GuidanceFrontmatter {
  lastUpdated: string;
  status: GuidanceStatus;
}

export interface TechnicalGuidanceMeta {
  path: string;
  frontmatter: GuidanceFrontmatter;
  hasInvestigationLog: boolean;
  openQuestionsCount: number;
}

// ============================================================================
// Path Constants
// ============================================================================

export const AGILE_DIR = "agile";
export const ARCHIVE_DIR = "agile/archive";
export const TEMPLATES_DIR = "agile/templates";

export const SLUG_PATTERN = /^[a-z0-9]+(-[a-z0-9]+)*$/;

// ============================================================================
// Issue Interfaces (Folder-Based)
// ============================================================================

export interface IssueMeta {
  title: string;
  type: Template | "unknown";
  created: string;
  owner: string;
}

export interface Issue {
  name: string;                    // Slug (folder name)
  stage: Stage;
  type: Template;
  path: string;                    // Full path to issue folder
  mainFilePath: string;            // Path to feature.md/bug.md/task.md
  technicalGuidancePath: string;   // Path to technical-guidance.md
  meta: IssueMeta;
}

// ============================================================================
// Definition of Done Types
// ============================================================================

export interface DoDItem {
  text: string;
  checked: boolean;
}

export interface DoDStatus {
  completed: number;
  total: number;
  items: DoDItem[];
}

// ============================================================================
// Issue Analysis Types
// ============================================================================

export interface IssueAnalysis {
  issue: Issue;
  specs: SpecInfo[];
  specsCompleted: number;
  specsTotal: number;
  allSpecsCompleted: boolean;
  technicalGuidance: TechnicalGuidanceMeta | null;
  needsGuidanceUpdate: boolean;
  dod: DoDStatus;
  incompleteSections: string[];
  hasDescription: boolean;
  ownerAssigned: boolean;
}

// ============================================================================
// Validation Types
// ============================================================================

export interface ValidationResult {
  valid: boolean;
  missing: string[];
}

// ============================================================================
// Create Options
// ============================================================================

export interface CreateOptions {
  title?: string;
  owner?: string;
  stage?: Stage;
}

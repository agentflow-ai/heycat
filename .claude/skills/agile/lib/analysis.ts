import { readFile } from "node:fs/promises";
import {
  type Stage,
  type Issue,
  type IssueAnalysis,
  type DoDStatus,
  type DoDItem,
} from "./types";
import { createSpecManager } from "./spec-manager";
import { createGuidanceTracker } from "./guidance-tracker";

// ============================================================================
// Placeholder Detection
// ============================================================================

const PLACEHOLDER_PATTERNS = [
  /\[[\w\s\-.,!?]+\]/g, // [placeholder text]
  /\[e\.g\.,?\s*[^\]]+\]/gi, // [e.g., examples]
];

// Patterns that look like placeholders but aren't (e.g., checkboxes)
const CHECKBOX_PATTERN = /\[\s*[xX]?\s*\]/g;

/**
 * Check if a string contains placeholder text (excluding checkboxes)
 */
export function hasPlaceholders(text: string): boolean {
  // Remove checkboxes before checking for placeholders
  const textWithoutCheckboxes = text.replace(CHECKBOX_PATTERN, "");
  // Create new RegExp instances to avoid stateful global flag issues
  return PLACEHOLDER_PATTERNS.some((pattern) => {
    const freshPattern = new RegExp(pattern.source, pattern.flags);
    return freshPattern.test(textWithoutCheckboxes);
  });
}

/**
 * Find all placeholders in text (excluding checkboxes)
 */
export function findPlaceholders(text: string): string[] {
  // Remove checkboxes before searching for placeholders
  const textWithoutCheckboxes = text.replace(CHECKBOX_PATTERN, "");
  const placeholders: string[] = [];
  for (const pattern of PLACEHOLDER_PATTERNS) {
    const matches = textWithoutCheckboxes.match(pattern);
    if (matches) {
      placeholders.push(...matches);
    }
  }
  return [...new Set(placeholders)];
}

// ============================================================================
// Section Parsing
// ============================================================================

interface Section {
  name: string;
  content: string;
  hasPlaceholders: boolean;
}

/**
 * Parse markdown content into sections
 */
export function parseSections(content: string): Section[] {
  const sections: Section[] = [];
  const lines = content.split("\n");

  let currentSection: Section | null = null;
  let currentContent: string[] = [];

  for (const line of lines) {
    const headerMatch = line.match(/^##\s+(.+)$/);
    if (headerMatch) {
      // Save previous section
      if (currentSection) {
        currentSection.content = currentContent.join("\n").trim();
        currentSection.hasPlaceholders = hasPlaceholders(currentSection.content);
        sections.push(currentSection);
      }
      // Start new section
      currentSection = {
        name: headerMatch[1],
        content: "",
        hasPlaceholders: false,
      };
      currentContent = [];
    } else if (currentSection) {
      currentContent.push(line);
    }
  }

  // Save last section
  if (currentSection) {
    currentSection.content = currentContent.join("\n").trim();
    currentSection.hasPlaceholders = hasPlaceholders(currentSection.content);
    sections.push(currentSection);
  }

  return sections;
}

/**
 * Find sections that still have placeholder content
 */
export function findIncompleteSections(content: string): string[] {
  const sections = parseSections(content);
  return sections.filter((s) => s.hasPlaceholders).map((s) => s.name);
}

// ============================================================================
// Definition of Done Parsing
// ============================================================================

/**
 * Parse Definition of Done checkboxes from issue content
 */
export function parseDoD(content: string): DoDStatus {
  // Find the DoD section first
  const dodSectionMatch = content.match(/## Definition of Done[\s\S]*?(?=##|$)/);
  if (!dodSectionMatch) {
    return { completed: 0, total: 0, items: [] };
  }

  const dodContent = dodSectionMatch[0];
  const checkboxPattern = /- \[([ xX])\]\s*(.+)$/gm;
  const items: DoDItem[] = [];

  let match;
  while ((match = checkboxPattern.exec(dodContent)) !== null) {
    items.push({
      checked: match[1].toLowerCase() === "x",
      text: match[2].trim(),
    });
  }

  return {
    completed: items.filter((i) => i.checked).length,
    total: items.length,
    items,
  };
}

// ============================================================================
// Issue Analysis (Folder-Based)
// ============================================================================

/**
 * Analyze a folder-based issue comprehensively
 */
export async function analyzeIssue(issue: Issue): Promise<IssueAnalysis> {
  const specManager = createSpecManager();
  const guidanceTracker = createGuidanceTracker();

  // Read main file content
  const mainContent = await readFile(issue.mainFilePath, "utf-8");
  const sections = parseSections(mainContent);
  const incompleteSections = sections.filter((s) => s.hasPlaceholders).map((s) => s.name);

  // Check description
  const descSection = sections.find((s) => s.name === "Description");
  const hasDescription = descSection
    ? !descSection.hasPlaceholders && descSection.content.length > 10
    : false;

  // Check owner
  const ownerAssigned = issue.meta.owner !== "[Name]" && issue.meta.owner.length > 0;

  // Parse DoD
  const dod = parseDoD(mainContent);

  // Get specs
  const specStatus = await specManager.getCompletionStatus(issue);

  // Get technical guidance
  const technicalGuidance = await guidanceTracker.getGuidanceMeta(issue);
  const needsGuidanceUpdate = await guidanceTracker.needsUpdate(issue, specStatus.specs);

  return {
    issue,
    specs: specStatus.specs,
    specsCompleted: specStatus.completed,
    specsTotal: specStatus.total,
    allSpecsCompleted: specStatus.allCompleted,
    technicalGuidance,
    needsGuidanceUpdate,
    dod,
    incompleteSections,
    hasDescription,
    ownerAssigned,
  };
}

// ============================================================================
// Stage Guidance (Updated for Specs)
// ============================================================================

export interface StageGuidance {
  focus: string;
  actions: string[];
  readinessChecklist: string[];
}

export const STAGE_GUIDANCE: Record<Stage, StageGuidance> = {
  "1-backlog": {
    focus: "Define the issue clearly so it can be prioritized",
    actions: [
      "Populate Description with clear context and purpose",
      "Write high-level acceptance criteria",
      "Document the goal and approach",
      "Consider initial spec breakdown using `spec suggest`",
    ],
    readinessChecklist: ["Description has no placeholder text", "Basic scope is defined"],
  },
  "2-todo": {
    focus: "Break down into specs and prepare for implementation",
    actions: [
      "Run `agile.ts spec suggest <issue>` to generate spec breakdown",
      "Review and refine suggested specs",
      "Add or remove specs as needed with `spec add/delete`",
      "Update technical guidance with initial approach",
      "Ensure owner is assigned",
    ],
    readinessChecklist: [
      "Owner is assigned",
      "Technical guidance file exists",
      "Specs are created (at least one)",
    ],
  },
  "3-in-progress": {
    focus: "Work through specs one at a time",
    actions: [
      "Start a spec with `agile.ts spec status <issue> <spec> in-progress`",
      "Complete the spec implementation",
      "Update technical guidance with discoveries",
      "Mark spec complete with `spec status <issue> <spec> completed`",
      "Repeat for remaining specs",
    ],
    readinessChecklist: [
      "All specs completed",
      "Technical guidance updated",
      "Ready for review",
    ],
  },
  "4-review": {
    focus: "Ensure quality and completeness",
    actions: [
      "Walk through Definition of Done checklist",
      "Verify all acceptance criteria are met",
      "Check tests are passing",
      "Confirm technical guidance reflects final state",
      "Update documentation as needed",
    ],
    readinessChecklist: ["All DoD items checked", "All acceptance criteria met"],
  },
  "5-done": {
    focus: "Wrap up and archive",
    actions: [
      "Celebrate completion!",
      "Archive with `agile.ts archive <issue>`",
      "Identify follow-up work for future issues",
    ],
    readinessChecklist: ["Work is complete"],
  },
};

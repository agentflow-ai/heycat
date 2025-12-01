import { readFile } from "node:fs/promises";
import {
  type Stage,
  type Issue,
  type IssueAnalysis,
  type DoDStatus,
  type DoDItem,
  type SpecInfo,
  type SpecIntegrationStatus,
  type IntegrationAnalysis,
  type DiscoveryPhase,
  DISCOVERY_PHASES,
} from "./types";
import { createSpecManager } from "./spec-manager";
import { createGuidanceTracker } from "./guidance-tracker";
import { validateBDDScenarios, hasBDDScenarios } from "./validators/bdd-validator";

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
// BDD Scenario Detection
// ============================================================================

/**
 * Placeholder patterns that indicate BDD scenarios haven't been written yet
 */
const BDD_PLACEHOLDER_PATTERNS = [
  /\[No scenarios defined yet\]/i,
  /\[Write scenarios here\]/i,
];

/**
 * Check if content has proper BDD scenarios (Given/When/Then format)
 * Returns true if:
 * - Has a "## BDD Scenarios" section (or similar)
 * - Contains at least one Given, When, and Then clause
 * - Does not contain placeholder text
 */
export function detectBDDScenarios(content: string): boolean {
  // Check for BDD Scenarios section
  const hasBDDSection = /## BDD Scenarios/i.test(content);
  if (!hasBDDSection) {
    return false;
  }

  // Extract the BDD Scenarios section content
  const bddSectionMatch = content.match(/## BDD Scenarios[\s\S]*?(?=##|$)/i);
  if (!bddSectionMatch) {
    return false;
  }

  const bddContent = bddSectionMatch[0];

  // Check for placeholder text
  for (const pattern of BDD_PLACEHOLDER_PATTERNS) {
    if (pattern.test(bddContent)) {
      return false;
    }
  }

  // Check for Given/When/Then patterns
  const hasGiven = /\bGiven\b/i.test(bddContent);
  const hasWhen = /\bWhen\b/i.test(bddContent);
  const hasThen = /\bThen\b/i.test(bddContent);

  return hasGiven && hasWhen && hasThen;
}

// ============================================================================
// Discovery Phase Parsing
// ============================================================================

/**
 * Parse discovery_phase from feature file frontmatter or content
 * Returns 'not_started' if not found
 */
export function parseDiscoveryPhase(content: string): DiscoveryPhase {
  // Check frontmatter for discovery_phase
  const frontmatterMatch = content.match(/^---\s*\n([\s\S]*?)\n---/);
  if (frontmatterMatch) {
    const frontmatter = frontmatterMatch[1];
    const phaseMatch = frontmatter.match(/discovery_phase:\s*(\w+)/i);
    if (phaseMatch) {
      const phase = phaseMatch[1].toLowerCase() as DiscoveryPhase;
      if (DISCOVERY_PHASES.includes(phase)) {
        return phase;
      }
    }
  }

  // Also check for **Discovery Phase:** format (in body)
  const bodyMatch = content.match(/\*\*Discovery Phase:\*\*\s*(\w+)/i);
  if (bodyMatch) {
    const phase = bodyMatch[1].toLowerCase() as DiscoveryPhase;
    if (DISCOVERY_PHASES.includes(phase)) {
      return phase;
    }
  }

  return "not_started";
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
// Integration Analysis
// ============================================================================

/**
 * Check if a section exists and is complete (not placeholder text or empty)
 * Returns: { exists: boolean, complete: boolean }
 */
function checkIntegrationSection(
  sections: Section[],
  sectionName: string
): { exists: boolean; complete: boolean } {
  const section = sections.find((s) => s.name === sectionName);
  if (!section) {
    return { exists: false, complete: false };
  }
  // Section exists - check if it's filled out
  // "N/A" counts as complete, placeholder text does not
  const content = section.content.trim().toLowerCase();
  const isNA = content === "n/a" || content.includes("n/a (");
  const hasContent = section.content.trim().length > 0;
  const complete = hasContent && (isNA || !section.hasPlaceholders);
  return { exists: true, complete };
}

/**
 * Analyze integration readiness for a single spec
 */
async function analyzeSpecIntegration(spec: SpecInfo): Promise<SpecIntegrationStatus> {
  const content = await readFile(spec.path, "utf-8");
  const sections = parseSections(content);

  const integrationPoints = checkIntegrationSection(sections, "Integration Points");
  const integrationTest = checkIntegrationSection(sections, "Integration Test");

  // Grandfathered if NEITHER section exists (pre-template spec)
  const isGrandfathered = !integrationPoints.exists && !integrationTest.exists;

  return {
    specName: spec.name,
    hasIntegrationPointsSection: integrationPoints.exists,
    hasIntegrationTestSection: integrationTest.exists,
    integrationPointsComplete: integrationPoints.complete,
    integrationTestComplete: integrationTest.complete,
    isGrandfathered,
  };
}

/**
 * Analyze integration readiness for all specs in an issue
 */
async function analyzeIntegration(specs: SpecInfo[]): Promise<IntegrationAnalysis> {
  const specStatuses = await Promise.all(specs.map(analyzeSpecIntegration));

  const incompleteSpecs: string[] = [];
  for (const status of specStatuses) {
    // Skip grandfathered specs
    if (status.isGrandfathered) continue;

    // Check if this spec has incomplete integration sections
    const hasIncomplete =
      (status.hasIntegrationPointsSection && !status.integrationPointsComplete) ||
      (status.hasIntegrationTestSection && !status.integrationTestComplete);

    if (hasIncomplete) {
      incompleteSpecs.push(status.specName);
    }
  }

  return {
    specs: specStatuses,
    allGrandfathered: specStatuses.every((s) => s.isGrandfathered),
    incompleteSpecs,
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

  // Analyze integration readiness
  const integration = await analyzeIntegration(specStatus.specs);

  // Parse discovery phase (features only)
  const discoveryPhase = issue.type === "feature"
    ? parseDiscoveryPhase(mainContent)
    : "complete"; // Non-features don't need discovery

  // Validate BDD scenarios (features only)
  const bddValidation = issue.type === "feature"
    ? validateBDDScenarios(mainContent)
    : null;

  // Check for BDD scenarios (backward compatibility + quick check)
  const hasBDDScenariosResult = issue.type === "feature"
    ? hasBDDScenarios(mainContent)
    : true; // Non-features don't need BDD scenarios

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
    integration,
    hasBDDScenarios: hasBDDScenariosResult,
    discoveryPhase,
    bddValidation,
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

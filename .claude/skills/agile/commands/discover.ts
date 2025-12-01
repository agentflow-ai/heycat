import { readFile, writeFile } from "node:fs/promises";
import { findProjectRoot } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import { parseDiscoveryPhase } from "../lib/analysis";
import {
  validateBDDScenarios,
  formatValidationErrors,
} from "../lib/validators/bdd-validator";
import {
  type Issue,
  type DiscoveryPhase,
  DISCOVERY_PHASE_ORDER,
  DISCOVERY_PHASE_NAMES,
} from "../lib/types";

// ============================================================================
// Discovery Context
// ============================================================================

interface DiscoveryContext {
  issue: Issue;
  phase: DiscoveryPhase;
  content: string;
}

async function loadDiscoveryContext(issueName: string): Promise<DiscoveryContext> {
  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();

  const issue = await resolver.findIssue(projectRoot, issueName);

  if (!issue) {
    console.error(`Issue not found: "${issueName}"`);
    console.error("Run 'agile.ts list' to see available issues.");
    process.exit(1);
  }

  if (issue.type !== "feature") {
    console.error(`Discovery is only for features, not ${issue.type}s.`);
    console.error("Bugs and tasks can be moved to todo without BDD scenarios.");
    process.exit(1);
  }

  const content = await readFile(issue.mainFilePath, "utf-8");
  const phase = parseDiscoveryPhase(content);

  return { issue, phase, content };
}

// ============================================================================
// Phase Navigation
// ============================================================================

function getNextPhase(current: DiscoveryPhase): DiscoveryPhase | null {
  const idx = DISCOVERY_PHASE_ORDER.indexOf(current);
  if (idx < 0 || idx >= DISCOVERY_PHASE_ORDER.length - 1) return null;
  return DISCOVERY_PHASE_ORDER[idx + 1];
}

function getPreviousPhase(current: DiscoveryPhase): DiscoveryPhase | null {
  const idx = DISCOVERY_PHASE_ORDER.indexOf(current);
  if (idx <= 0) return null;
  return DISCOVERY_PHASE_ORDER[idx - 1];
}

// ============================================================================
// Frontmatter Update
// ============================================================================

async function updateDiscoveryPhase(
  issue: Issue,
  newPhase: DiscoveryPhase
): Promise<void> {
  const content = await readFile(issue.mainFilePath, "utf-8");

  let updated: string;

  // Check if frontmatter exists
  const frontmatterMatch = content.match(/^(---\s*\n)([\s\S]*?)(\n---)/);

  if (frontmatterMatch) {
    const [, start, frontmatter, end] = frontmatterMatch;
    const afterFrontmatter = content.slice(frontmatterMatch[0].length);

    // Check if discovery_phase exists in frontmatter
    if (/discovery_phase:/i.test(frontmatter)) {
      // Update existing
      const newFrontmatter = frontmatter.replace(
        /discovery_phase:\s*\w+/i,
        `discovery_phase: ${newPhase}`
      );
      updated = start + newFrontmatter + end + afterFrontmatter;
    } else {
      // Add to frontmatter
      const newFrontmatter = frontmatter.trimEnd() + `\ndiscovery_phase: ${newPhase}`;
      updated = start + newFrontmatter + end + afterFrontmatter;
    }
  } else {
    // No frontmatter - add it
    updated = `---\ndiscovery_phase: ${newPhase}\n---\n\n${content}`;
  }

  await writeFile(issue.mainFilePath, updated);
}

// ============================================================================
// Phase Guidance Output
// ============================================================================

const PHASE_QUESTIONS: Record<DiscoveryPhase, string[]> = {
  not_started: [],
  persona: [
    "1. WHO is the primary user of this feature? (role, technical level, context)",
    "2. WHAT problem are they trying to solve? (pain point, workarounds, desired outcome)",
    "3. WHY is this important to solve now? (urgency, business impact, blockers)",
  ],
  paths: [
    "4. Walk through the IDEAL successful experience: Given (preconditions) → When (action) → Then (outcome)",
    "5. What VARIATIONS of the happy path exist? (different starting states, user types, flows)",
    "6. What could go WRONG? (errors, edge cases, invalid inputs, system failures)",
    "7. How should FAILURES be handled? (error messages, fallbacks, recovery paths)",
  ],
  scope: [
    "8. What is explicitly OUT OF SCOPE? (deferred features, adjacent functionality)",
    "9. What ASSUMPTIONS are we making? (preconditions, dependencies, system requirements)",
  ],
  synthesize: [],
  complete: [],
};

const PHASE_OUTPUT_FORMAT: Record<DiscoveryPhase, string> = {
  not_started: "",
  persona: `After gathering responses, add to ## BDD Scenarios:
### User Persona
<brief persona description>

### Problem Statement
<what problem is being solved>`,
  paths: `After gathering responses, add Gherkin scenarios:
\`\`\`gherkin
Feature: <title>

  Scenario: Happy path - <description>
    Given <context>
    When <action>
    Then <outcome>

  Scenario: Error case - <description>
    Given <context>
    When <failure condition>
    Then <error handling>
\`\`\``,
  scope: `After gathering responses, add:
### Out of Scope
- <item 1>
- <item 2>

### Assumptions
- <assumption 1>
- <assumption 2>`,
  synthesize: `Review all BDD content for:
- Consistency between persona, problem, and scenarios
- Proper Gherkin format (Given/When/Then)
- At least 2 scenarios (happy path + edge case)
- Completeness of Out of Scope and Assumptions`,
  complete: "",
};

function outputPhaseGuidance(ctx: DiscoveryContext): void {
  console.log(`\nDISCOVERY: ${ctx.issue.name}`);
  console.log(`Phase: ${DISCOVERY_PHASE_NAMES[ctx.phase]} (${ctx.phase})`);
  console.log(`File: ${ctx.issue.mainFilePath}\n`);

  if (ctx.phase === "not_started") {
    console.log("Discovery has not been started.");
    console.log("\nTo begin, run:");
    console.log(`  bun .claude/skills/agile/agile.ts discover ${ctx.issue.name} advance`);
    return;
  }

  if (ctx.phase === "complete") {
    console.log("Discovery is complete. BDD scenarios are ready.");
    console.log("\nTo restart discovery, run:");
    console.log(`  bun .claude/skills/agile/agile.ts discover ${ctx.issue.name} reset`);

    // Show validation status
    const validation = validateBDDScenarios(ctx.content);
    if (!validation.valid) {
      console.log("\nValidation issues found:");
      console.log(formatValidationErrors(ctx.issue.name, validation));
    } else {
      console.log("\nBDD scenarios are valid. Feature can move to todo stage.");
    }
    return;
  }

  // Show phase questions
  const questions = PHASE_QUESTIONS[ctx.phase];
  if (questions.length > 0) {
    console.log("Questions to ask (one at a time, wait for responses):");
    for (const q of questions) {
      console.log(`  ${q}`);
    }
    console.log();
  }

  // Show expected output
  const outputFormat = PHASE_OUTPUT_FORMAT[ctx.phase];
  if (outputFormat) {
    console.log("Expected output format:");
    console.log(outputFormat);
    console.log();
  }

  // Synthesize phase special handling
  if (ctx.phase === "synthesize") {
    const validation = validateBDDScenarios(ctx.content);
    if (validation.formatErrors.length > 0 || validation.completenessErrors.length > 0) {
      console.log("Current validation status:");
      console.log(formatValidationErrors(ctx.issue.name, validation));
    } else {
      console.log("All validation checks passing!");
    }
  }

  console.log("When phase is complete, run:");
  console.log(`  bun .claude/skills/agile/agile.ts discover ${ctx.issue.name} advance`);
}

// ============================================================================
// Subcommand Handlers
// ============================================================================

async function handleAdvance(ctx: DiscoveryContext): Promise<void> {
  const nextPhase = getNextPhase(ctx.phase);

  if (!nextPhase) {
    console.log("Discovery is already complete.");
    return;
  }

  // Special validation for synthesize -> complete transition
  if (ctx.phase === "synthesize") {
    const validation = validateBDDScenarios(ctx.content);
    if (!validation.valid) {
      console.error("Cannot complete discovery. Validation errors found:\n");
      console.error(formatValidationErrors(ctx.issue.name, validation));
      console.error("\nFix the issues above and try again.");
      process.exit(1);
    }
  }

  // Update phase
  await updateDiscoveryPhase(ctx.issue, nextPhase);

  console.log(`Advanced to phase: ${DISCOVERY_PHASE_NAMES[nextPhase]}`);

  // Output guidance for new phase
  const newCtx = { ...ctx, phase: nextPhase };
  outputPhaseGuidance(newCtx);
}

async function handleReset(ctx: DiscoveryContext): Promise<void> {
  if (ctx.phase === "not_started") {
    console.log("Discovery is already at not_started phase.");
    return;
  }

  await updateDiscoveryPhase(ctx.issue, "not_started");
  console.log("Discovery has been reset to not_started.");
  console.log("\nNote: BDD Scenarios section content was NOT cleared.");
  console.log("Manually remove content if you want to start fresh.");
}

function outputStatus(ctx: DiscoveryContext): void {
  console.log(`\nDISCOVERY STATUS: ${ctx.issue.name}`);
  console.log(`Current Phase: ${DISCOVERY_PHASE_NAMES[ctx.phase]} (${ctx.phase})`);
  console.log(`File: ${ctx.issue.mainFilePath}`);

  // Show phase progress
  const currentIdx = DISCOVERY_PHASE_ORDER.indexOf(ctx.phase);
  console.log("\nProgress:");
  for (let i = 0; i < DISCOVERY_PHASE_ORDER.length; i++) {
    const phase = DISCOVERY_PHASE_ORDER[i];
    const name = DISCOVERY_PHASE_NAMES[phase];
    const marker = i < currentIdx ? "[x]" : i === currentIdx ? "[>]" : "[ ]";
    console.log(`  ${marker} ${name}`);
  }

  // Show validation status
  const validation = validateBDDScenarios(ctx.content);
  console.log("\nValidation:");
  console.log(`  Scenarios: ${validation.scenarioCount}`);
  console.log(`  User Persona: ${validation.hasUserPersona ? "Yes" : "No"}`);
  console.log(`  Problem Statement: ${validation.hasProblemStatement ? "Yes" : "No"}`);
  console.log(`  Out of Scope: ${validation.hasOutOfScope ? "Yes" : "No"}`);
  console.log(`  Assumptions: ${validation.hasAssumptions ? "Yes" : "No"}`);

  if (validation.formatErrors.length > 0) {
    console.log(`  Format Errors: ${validation.formatErrors.length}`);
  }
  if (validation.completenessErrors.length > 0) {
    console.log(`  Completeness Errors: ${validation.completenessErrors.length}`);
  }

  console.log(`\nReady for todo: ${validation.valid && ctx.phase === "complete" ? "Yes" : "No"}`);
}

async function handleValidate(ctx: DiscoveryContext): Promise<void> {
  const validation = validateBDDScenarios(ctx.content);

  console.log(`\nBDD VALIDATION: ${ctx.issue.name}\n`);

  if (validation.valid) {
    console.log("All validation checks passed!");
    console.log(`  Scenarios: ${validation.scenarioCount}`);
    console.log("  User Persona: Present");
    console.log("  Problem Statement: Present");
    console.log("  Out of Scope: Present");
    console.log("  Assumptions: Present");
  } else {
    console.log(formatValidationErrors(ctx.issue.name, validation));
  }
}

// ============================================================================
// Main Handler
// ============================================================================

export async function handleDiscover(args: string[]): Promise<void> {
  const [issueName, subcommand] = args;

  if (!issueName) {
    console.error("Usage: agile.ts discover <issue-name> [subcommand]");
    console.error("\nSubcommands:");
    console.error("  (none)    Show current phase guidance");
    console.error("  advance   Move to next phase (validates current phase)");
    console.error("  status    Show discovery progress and validation status");
    console.error("  validate  Run BDD validation without advancing");
    console.error("  reset     Reset discovery to not_started");
    process.exit(1);
  }

  const ctx = await loadDiscoveryContext(issueName);

  switch (subcommand) {
    case "advance":
      await handleAdvance(ctx);
      break;
    case "status":
      outputStatus(ctx);
      break;
    case "validate":
      await handleValidate(ctx);
      break;
    case "reset":
      await handleReset(ctx);
      break;
    default:
      outputPhaseGuidance(ctx);
  }
}

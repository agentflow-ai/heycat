import { parseArgs } from "node:util";
import { STAGE_NAMES, VALID_TRANSITIONS, type Stage } from "../lib/types";
import { findProjectRoot } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import { analyzeIssue, STAGE_GUIDANCE } from "../lib/analysis";
import { createValidatorChain } from "../lib/validators";

export async function handleWork(args: string[]): Promise<void> {
  const { positionals } = parseArgs({
    args,
    allowPositionals: true,
  });

  const [name] = positionals;

  if (!name) {
    console.error("Usage: agile.ts work <name>");
    console.error("Analyze an issue and get stage-appropriate guidance.");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const issue = await resolver.findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  // Analyze issue
  const analysis = await analyzeIssue(issue);
  const guidance = STAGE_GUIDANCE[issue.stage];

  // Determine next stage
  const allowedTransitions = VALID_TRANSITIONS[issue.stage];
  const forwardStage = allowedTransitions.find(
    (s) => parseInt(s[0]) > parseInt(issue.stage[0])
  );

  // Validate readiness for next stage
  const validatorChain = createValidatorChain();
  let readiness = { valid: true, missing: [] as string[] };
  if (forwardStage) {
    readiness = validatorChain.validate(issue, analysis, forwardStage);
  }

  // Output structured analysis
  console.log("=".repeat(80));
  console.log(`WORK SESSION: ${name}`);
  console.log("=".repeat(80));
  console.log();

  console.log("ISSUE DETAILS");
  console.log(`  Type:     ${issue.type}`);
  console.log(`  Stage:    ${issue.stage} (${STAGE_NAMES[issue.stage]})`);
  console.log(`  Owner:    ${issue.meta.owner}`);
  console.log(`  Created:  ${issue.meta.created}`);
  console.log(`  Path:     ${issue.path}/`);
  console.log();

  // Specs status
  console.log("SPECS STATUS");
  if (analysis.specsTotal === 0) {
    console.log("  No specs created yet");
    console.log("  Run: agile.ts spec suggest " + name);
  } else {
    console.log(`  Total: ${analysis.specsTotal} specs`);
    console.log(`  Completed: ${analysis.specsCompleted} | In Progress: ${analysis.specs.filter(s => s.frontmatter.status === "in-progress").length} | Pending: ${analysis.specs.filter(s => s.frontmatter.status === "pending").length}`);
    console.log();
    console.log("  ┌─────────────────────────────┬────────────┐");
    console.log("  │ Spec                        │ Status     │");
    console.log("  ├─────────────────────────────┼────────────┤");
    for (const spec of analysis.specs) {
      const specName = spec.name.padEnd(27).slice(0, 27);
      const status = spec.frontmatter.status.padEnd(10);
      console.log(`  │ ${specName} │ ${status} │`);
    }
    console.log("  └─────────────────────────────┴────────────┘");
  }
  console.log();

  // Technical guidance status
  console.log("TECHNICAL GUIDANCE");
  if (analysis.technicalGuidance) {
    console.log(`  Last Updated: ${analysis.technicalGuidance.frontmatter.lastUpdated}`);
    console.log(`  Status: ${analysis.technicalGuidance.frontmatter.status}`);
    if (analysis.needsGuidanceUpdate) {
      console.log("  ⚠ NEEDS UPDATE (spec completed since last update)");
      console.log("  Run: agile.ts guidance update " + name);
    }
  } else {
    console.log("  ⚠ Technical guidance file not found");
  }
  console.log();

  // Analysis
  console.log("ANALYSIS");
  if (analysis.incompleteSections.length > 0) {
    console.log("  Incomplete Sections:");
    for (const section of analysis.incompleteSections) {
      console.log(`    - ${section} (has placeholder text)`);
    }
  } else {
    console.log("  All sections complete");
  }
  console.log();

  console.log(`  Definition of Done: ${analysis.dod.completed}/${analysis.dod.total} completed`);
  for (const item of analysis.dod.items) {
    console.log(`    [${item.checked ? "x" : " "}] ${item.text}`);
  }
  console.log();

  console.log(`STAGE GUIDANCE (${STAGE_NAMES[issue.stage]})`);
  console.log(`  Focus: ${guidance.focus}`);
  console.log();
  console.log("  Suggested Actions:");
  for (let i = 0; i < guidance.actions.length; i++) {
    console.log(`    ${i + 1}. ${guidance.actions[i]}`);
  }
  console.log();

  if (forwardStage) {
    console.log("READINESS TO ADVANCE");
    console.log(`  Status: ${readiness.valid ? "READY" : "NOT READY"}`);
    if (readiness.missing.length > 0) {
      console.log("  Blockers:");
      for (const m of readiness.missing) {
        console.log(`    - ${m}`);
      }
    }
    console.log();
    console.log(`  Next Stage: ${forwardStage} (${STAGE_NAMES[forwardStage]})`);
    console.log(`  Command: bun .claude/skills/agile/agile.ts move ${name} ${forwardStage}`);
  } else {
    console.log("COMPLETION");
    console.log("  This issue is in the final stage (Done).");
    console.log("  Consider archiving:");
    console.log(`  Command: bun .claude/skills/agile/agile.ts archive ${name}`);
  }

  console.log();
  console.log("=".repeat(80));
}

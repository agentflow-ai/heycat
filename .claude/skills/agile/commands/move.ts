import { parseArgs } from "node:util";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";
import {
  STAGES,
  STAGE_NAMES,
  VALID_TRANSITIONS,
  AGILE_DIR,
  type Stage,
} from "../lib/types";
import { isValidStage, findProjectRoot } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import { analyzeIssue } from "../lib/analysis";
import { createValidatorChain } from "../lib/validators";

export async function handleMove(args: string[]): Promise<void> {
  const { positionals } = parseArgs({
    args,
    allowPositionals: true,
  });

  const [name, toStage] = positionals;

  if (!name || !toStage) {
    console.error("Usage: agile.ts move <name> <stage>");
    console.error(`Stages: ${STAGES.join(", ")}`);
    process.exit(1);
  }

  if (!isValidStage(toStage)) {
    console.error(`Invalid stage: "${toStage}". Valid stages: ${STAGES.join(", ")}`);
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const issue = await resolver.findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  if (issue.stage === toStage) {
    console.log(`Issue is already in ${STAGE_NAMES[toStage]}`);
    return;
  }

  // Validate transition path
  const allowedTransitions = VALID_TRANSITIONS[issue.stage];
  if (!allowedTransitions.includes(toStage as Stage)) {
    console.error(
      `Invalid transition: ${STAGE_NAMES[issue.stage]} -> ${STAGE_NAMES[toStage as Stage]}`
    );
    console.error(
      `Allowed from ${STAGE_NAMES[issue.stage]}: ${allowedTransitions.map((s) => STAGE_NAMES[s]).join(", ")}`
    );
    process.exit(1);
  }

  // Analyze issue and validate for target stage
  const analysis = await analyzeIssue(issue);
  const validatorChain = createValidatorChain();
  const validation = validatorChain.validate(issue, analysis, toStage as Stage);

  if (!validation.valid) {
    console.error(`Cannot move to ${STAGE_NAMES[toStage as Stage]} - requirements not met:`);
    for (const missing of validation.missing) {
      console.error(`  - ${missing}`);
    }
    console.error();
    console.error(`Run "bun .claude/skills/agile/agile.ts work ${name}" for guidance.`);
    process.exit(1);
  }

  // Ensure target directory exists
  const targetDir = join(projectRoot, AGILE_DIR, toStage);
  await mkdir(targetDir, { recursive: true });

  // Move the issue folder
  const movedIssue = await resolver.moveIssue(projectRoot, issue, toStage as Stage);

  console.log(`Moved: ${movedIssue.name}`);
  console.log(`  ${STAGE_NAMES[issue.stage]} -> ${STAGE_NAMES[movedIssue.stage]}`);

  // Show spec status if relevant
  if (analysis.specsTotal > 0) {
    console.log(`  Specs: ${analysis.specsCompleted}/${analysis.specsTotal} completed`);
  }
}

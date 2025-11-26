import { parseArgs } from "node:util";
import { readFile } from "node:fs/promises";
import { GUIDANCE_STATUSES, type GuidanceStatus } from "../lib/types";
import { findProjectRoot } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import { createGuidanceTracker } from "../lib/guidance-tracker";
import { createSpecManager } from "../lib/spec-manager";

export async function handleGuidance(args: string[]): Promise<void> {
  const subcommand = args[0];
  const subArgs = args.slice(1);

  switch (subcommand) {
    case "show":
      await handleGuidanceShow(subArgs);
      break;
    case "update":
      await handleGuidanceUpdate(subArgs);
      break;
    case "validate":
      await handleGuidanceValidate(subArgs);
      break;
    case "status":
      await handleGuidanceStatus(subArgs);
      break;
    default:
      showGuidanceHelp();
      process.exit(subcommand ? 1 : 0);
  }
}

function showGuidanceHelp(): void {
  console.log("Usage: agile.ts guidance <subcommand> [options]");
  console.log();
  console.log("Subcommands:");
  console.log("  show <issue>                  Show technical guidance summary");
  console.log("  update <issue>                Mark guidance as updated (set timestamp)");
  console.log("  validate <issue>              Check if guidance is current");
  console.log("  status <issue> <status>       Set guidance status (draft/active/finalized)");
  console.log();
  console.log("Statuses: draft, active, finalized");
}

async function handleGuidanceShow(args: string[]): Promise<void> {
  const { positionals } = parseArgs({ args, allowPositionals: true });
  const [issueName] = positionals;

  if (!issueName) {
    console.error("Usage: agile.ts guidance show <issue>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const guidanceTracker = createGuidanceTracker();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  const meta = await guidanceTracker.getGuidanceMeta(issue);
  if (!meta) {
    console.error("Technical guidance file not found");
    console.error(`  Expected: ${issue.technicalGuidancePath}`);
    process.exit(1);
  }

  console.log(`\nTechnical Guidance for ${issue.name}`);
  console.log("─".repeat(60));
  console.log(`  Path:         ${issue.technicalGuidancePath}`);
  console.log(`  Last Updated: ${meta.frontmatter.lastUpdated}`);
  console.log(`  Status:       ${meta.frontmatter.status}`);
  console.log(`  Has Investigation Log: ${meta.hasInvestigationLog ? "Yes" : "No"}`);
  console.log(`  Open Questions: ${meta.openQuestionsCount}`);
  console.log();

  // Show first few lines of content
  try {
    const content = await readFile(issue.technicalGuidancePath, "utf-8");
    const lines = content.split("\n");
    const previewLines = lines.slice(0, 30);

    console.log("Preview:");
    console.log("─".repeat(60));
    for (const line of previewLines) {
      console.log(line);
    }
    if (lines.length > 30) {
      console.log(`... (${lines.length - 30} more lines)`);
    }
  } catch {
    console.log("(Could not read content)");
  }
  console.log();
}

async function handleGuidanceUpdate(args: string[]): Promise<void> {
  const { positionals } = parseArgs({ args, allowPositionals: true });
  const [issueName] = positionals;

  if (!issueName) {
    console.error("Usage: agile.ts guidance update <issue>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const guidanceTracker = createGuidanceTracker();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  const exists = await guidanceTracker.exists(issue);
  if (!exists) {
    console.error("Technical guidance file not found");
    console.error(`  Expected: ${issue.technicalGuidancePath}`);
    process.exit(1);
  }

  try {
    await guidanceTracker.markUpdated(issue);
    const meta = await guidanceTracker.getGuidanceMeta(issue);
    console.log(`Updated technical guidance timestamp`);
    console.log(`  Last Updated: ${meta?.frontmatter.lastUpdated}`);
  } catch (err) {
    console.error(`Error: ${(err as Error).message}`);
    process.exit(1);
  }
}

async function handleGuidanceValidate(args: string[]): Promise<void> {
  const { positionals } = parseArgs({ args, allowPositionals: true });
  const [issueName] = positionals;

  if (!issueName) {
    console.error("Usage: agile.ts guidance validate <issue>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const guidanceTracker = createGuidanceTracker();
  const specManager = createSpecManager();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  const meta = await guidanceTracker.getGuidanceMeta(issue);
  if (!meta) {
    console.error("Technical guidance file not found");
    process.exit(1);
  }

  const specs = await specManager.listSpecs(issue);
  const needsUpdate = await guidanceTracker.needsUpdate(issue, specs);

  console.log(`\nGuidance Validation for ${issue.name}`);
  console.log("─".repeat(60));
  console.log(`  Last Updated: ${meta.frontmatter.lastUpdated}`);
  console.log(`  Status: ${meta.frontmatter.status}`);
  console.log();

  if (needsUpdate) {
    console.log("  ⚠ NEEDS UPDATE");
    console.log();
    console.log("  Completed specs after last guidance update:");
    for (const spec of specs) {
      if (spec.frontmatter.status === "completed" && spec.frontmatter.completed) {
        if (spec.frontmatter.completed > meta.frontmatter.lastUpdated) {
          console.log(`    - ${spec.name} (completed: ${spec.frontmatter.completed})`);
        }
      }
    }
    console.log();
    console.log("  Run: agile.ts guidance update " + issueName);
  } else {
    console.log("  ✓ UP TO DATE");
    console.log("  Technical guidance is current with all completed specs.");
  }
  console.log();
}

async function handleGuidanceStatus(args: string[]): Promise<void> {
  const { positionals } = parseArgs({ args, allowPositionals: true });
  const [issueName, status] = positionals;

  if (!issueName || !status) {
    console.error("Usage: agile.ts guidance status <issue> <draft|active|finalized>");
    process.exit(1);
  }

  if (!GUIDANCE_STATUSES.includes(status as GuidanceStatus)) {
    console.error(`Invalid status: "${status}". Must be one of: ${GUIDANCE_STATUSES.join(", ")}`);
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const guidanceTracker = createGuidanceTracker();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  const exists = await guidanceTracker.exists(issue);
  if (!exists) {
    console.error("Technical guidance file not found");
    process.exit(1);
  }

  try {
    await guidanceTracker.setStatus(issue, status as GuidanceStatus);
    console.log(`Updated guidance status to: ${status}`);
  } catch (err) {
    console.error(`Error: ${(err as Error).message}`);
    process.exit(1);
  }
}

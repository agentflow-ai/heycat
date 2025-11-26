import { parseArgs } from "node:util";
import { SPEC_STATUSES, type SpecStatus } from "../lib/types";
import { findProjectRoot, toKebabCase } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import { createSpecManager } from "../lib/spec-manager";
import { createGuidanceTracker } from "../lib/guidance-tracker";

export async function handleSpec(args: string[]): Promise<void> {
  const subcommand = args[0];
  const subArgs = args.slice(1);

  switch (subcommand) {
    case "list":
      await handleSpecList(subArgs);
      break;
    case "add":
      await handleSpecAdd(subArgs);
      break;
    case "status":
      await handleSpecStatus(subArgs);
      break;
    case "delete":
      await handleSpecDelete(subArgs);
      break;
    case "suggest":
      await handleSpecSuggest(subArgs);
      break;
    default:
      showSpecHelp();
      process.exit(subcommand ? 1 : 0);
  }
}

function showSpecHelp(): void {
  console.log("Usage: agile.ts spec <subcommand> [options]");
  console.log();
  console.log("Subcommands:");
  console.log("  list <issue>                         List all specs in an issue");
  console.log("  add <issue> <name> [--title \"...\"]   Add a new spec");
  console.log("  status <issue> <spec> <status>       Update spec status");
  console.log("  delete <issue> <spec>                Delete a spec");
  console.log("  suggest <issue>                      Suggest specs from description (AI)");
  console.log();
  console.log("Statuses: pending, in-progress, completed");
}

async function handleSpecList(args: string[]): Promise<void> {
  const { positionals } = parseArgs({ args, allowPositionals: true });
  const [issueName] = positionals;

  if (!issueName) {
    console.error("Usage: agile.ts spec list <issue>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const specManager = createSpecManager();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  const specs = await specManager.listSpecs(issue);

  console.log(`\nSpecs for ${issue.name}:`);
  console.log("─".repeat(60));

  if (specs.length === 0) {
    console.log("  No specs yet. Run 'spec suggest' or 'spec add' to create specs.");
  } else {
    const status = await specManager.getCompletionStatus(issue);
    console.log(`Total: ${status.total} | Completed: ${status.completed} | In Progress: ${status.inProgress} | Pending: ${status.pending}`);
    console.log();

    for (const spec of specs) {
      const statusIcon = spec.frontmatter.status === "completed" ? "✓" :
                         spec.frontmatter.status === "in-progress" ? "→" : "○";
      const statusLabel = spec.frontmatter.status.padEnd(11);
      console.log(`  ${statusIcon} [${statusLabel}] ${spec.name}`);
      console.log(`     ${spec.title}`);
      if (spec.frontmatter.dependencies.length > 0) {
        console.log(`     deps: ${spec.frontmatter.dependencies.join(", ")}`);
      }
    }
  }
  console.log();
}

async function handleSpecAdd(args: string[]): Promise<void> {
  const { values, positionals } = parseArgs({
    args,
    options: {
      title: { type: "string", short: "t" },
    },
    allowPositionals: true,
  });

  const [issueName, specName] = positionals;

  if (!issueName || !specName) {
    console.error("Usage: agile.ts spec add <issue> <spec-name> [--title \"Title\"]");
    process.exit(1);
  }

  const slug = toKebabCase(specName);
  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const specManager = createSpecManager();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  try {
    const spec = await specManager.createSpec(projectRoot, issue, slug, values.title);
    console.log(`Created spec: ${spec.name}`);
    console.log(`  Path: ${spec.path}`);
    console.log(`  Title: ${spec.title}`);
    console.log();
    console.log("Next: Edit the spec file to fill in details");
  } catch (err) {
    console.error(`Error: ${(err as Error).message}`);
    process.exit(1);
  }
}

async function handleSpecStatus(args: string[]): Promise<void> {
  const { positionals } = parseArgs({ args, allowPositionals: true });
  const [issueName, specName, status] = positionals;

  if (!issueName || !specName || !status) {
    console.error("Usage: agile.ts spec status <issue> <spec> <pending|in-progress|completed>");
    process.exit(1);
  }

  if (!SPEC_STATUSES.includes(status as SpecStatus)) {
    console.error(`Invalid status: "${status}". Must be one of: ${SPEC_STATUSES.join(", ")}`);
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const specManager = createSpecManager();
  const guidanceTracker = createGuidanceTracker();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  const spec = await specManager.getSpec(issue, specName);
  if (!spec) {
    console.error(`Spec not found: ${specName}`);
    process.exit(1);
  }

  // If completing a spec, validate technical guidance was updated
  if (status === "completed" && spec.frontmatter.status !== "completed") {
    const validation = await guidanceTracker.validateForSpecCompletion(issue, spec);
    if (!validation.valid) {
      console.error("Cannot complete spec - technical guidance not updated:");
      console.error(`  ${validation.message}`);
      console.error();
      console.error("Run: agile.ts guidance update " + issueName);
      process.exit(1);
    }
  }

  try {
    const updated = await specManager.updateStatus(spec, status as SpecStatus);
    console.log(`Updated: ${updated.name}`);
    console.log(`  Status: ${spec.frontmatter.status} -> ${updated.frontmatter.status}`);

    if (status === "completed") {
      console.log();
      console.log("Remember to update technical guidance with any discoveries!");
    }
  } catch (err) {
    console.error(`Error: ${(err as Error).message}`);
    process.exit(1);
  }
}

async function handleSpecDelete(args: string[]): Promise<void> {
  const { positionals } = parseArgs({ args, allowPositionals: true });
  const [issueName, specName] = positionals;

  if (!issueName || !specName) {
    console.error("Usage: agile.ts spec delete <issue> <spec>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();
  const specManager = createSpecManager();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  const spec = await specManager.getSpec(issue, specName);
  if (!spec) {
    console.error(`Spec not found: ${specName}`);
    process.exit(1);
  }

  await specManager.deleteSpec(spec);
  console.log(`Deleted spec: ${specName}`);
}

async function handleSpecSuggest(args: string[]): Promise<void> {
  const { positionals } = parseArgs({ args, allowPositionals: true });
  const [issueName] = positionals;

  if (!issueName) {
    console.error("Usage: agile.ts spec suggest <issue>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();

  const issue = await resolver.findIssue(projectRoot, issueName);
  if (!issue) {
    console.error(`Issue not found: ${issueName}`);
    process.exit(1);
  }

  // This is a placeholder - actual AI suggestion would be done by the agent
  console.log(`\nSPEC SUGGESTION for ${issue.name}`);
  console.log("─".repeat(60));
  console.log();
  console.log("This command is designed for AI-assisted spec breakdown.");
  console.log("The Claude agent will:");
  console.log("  1. Read the issue description and acceptance criteria");
  console.log("  2. Identify natural breakpoints following SPS pattern");
  console.log("  3. Suggest spec names and brief descriptions");
  console.log("  4. Create specs upon user approval");
  console.log();
  console.log("To use, run: bun .claude/skills/agile/agile.ts spec suggest <issue>");
  console.log();
  console.log(`Issue path: ${issue.mainFilePath}`);
}

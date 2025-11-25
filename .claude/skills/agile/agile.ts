#!/usr/bin/env bun
import { parseArgs } from "node:util";
import { readdir, readFile, writeFile, rename, unlink, mkdir, stat } from "node:fs/promises";
import { join, dirname, basename } from "node:path";

// ============================================================================
// Constants
// ============================================================================

const STAGES = ["1-backlog", "2-todo", "3-in-progress", "4-review", "5-done"] as const;
type Stage = (typeof STAGES)[number];

const STAGE_NAMES: Record<Stage, string> = {
  "1-backlog": "Backlog",
  "2-todo": "Todo",
  "3-in-progress": "In Progress",
  "4-review": "Review",
  "5-done": "Done",
};

const TEMPLATES = ["feature", "bug", "task"] as const;
type Template = (typeof TEMPLATES)[number];

// Strict sequential transitions only
const VALID_TRANSITIONS: Record<Stage, Stage[]> = {
  "1-backlog": ["2-todo"],
  "2-todo": ["1-backlog", "3-in-progress"],
  "3-in-progress": ["2-todo", "4-review"],
  "4-review": ["3-in-progress", "5-done"],
  "5-done": ["4-review"],
};

const SLUG_PATTERN = /^[a-z0-9]+(-[a-z0-9]+)*$/;
const AGILE_DIR = "agile";
const ARCHIVE_DIR = "agile/archive";
const TEMPLATES_DIR = "agile/templates";

// ============================================================================
// Utilities
// ============================================================================

async function findProjectRoot(): Promise<string> {
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

function isValidStage(stage: string): stage is Stage {
  return STAGES.includes(stage as Stage);
}

function isValidTemplate(template: string): template is Template {
  return TEMPLATES.includes(template as Template);
}

function toKebabCase(str: string): string {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

function toTitleCase(str: string): string {
  return str
    .split("-")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}

function getCurrentDate(): string {
  return new Date().toISOString().split("T")[0];
}

function validateSlug(slug: string): void {
  if (!SLUG_PATTERN.test(slug)) {
    throw new Error(
      `Invalid name: "${slug}". Use kebab-case (lowercase letters, numbers, hyphens)`
    );
  }
}

// ============================================================================
// Issue Operations
// ============================================================================

interface IssueLocation {
  stage: Stage;
  path: string;
  filename: string;
}

interface IssueMeta {
  title: string;
  type: Template | "unknown";
  created: string;
}

async function findIssue(
  projectRoot: string,
  name: string
): Promise<IssueLocation | null> {
  const filename = name.endsWith(".md") ? name : `${name}.md`;

  for (const stage of STAGES) {
    const stagePath = join(projectRoot, AGILE_DIR, stage);
    try {
      const files = await readdir(stagePath);
      if (files.includes(filename)) {
        return {
          stage,
          path: join(stagePath, filename),
          filename,
        };
      }
    } catch {
      // Stage directory might not exist
    }
  }
  return null;
}

async function parseIssueMeta(filePath: string): Promise<IssueMeta> {
  const content = await readFile(filePath, "utf-8");
  const lines = content.split("\n");

  let title = "";
  let type: Template | "unknown" = "unknown";
  let created = "";

  // Parse first line for type and title
  const firstLine = lines[0] || "";
  const titleMatch = firstLine.match(/^#\s+(Feature|Bug|Task):\s*(.*)$/i);
  if (titleMatch) {
    type = titleMatch[1].toLowerCase() as Template;
    title = titleMatch[2].trim();
  }

  // Parse created date
  for (const line of lines) {
    const dateMatch = line.match(/\*\*Created:\*\*\s*(\d{4}-\d{2}-\d{2})/);
    if (dateMatch) {
      created = dateMatch[1];
      break;
    }
  }

  return { title, type, created };
}

// ============================================================================
// Command Handlers
// ============================================================================

async function handleCreate(args: string[]): Promise<void> {
  const { values, positionals } = parseArgs({
    args,
    options: {
      title: { type: "string", short: "t" },
      stage: { type: "string", short: "s", default: "1-backlog" },
    },
    allowPositionals: true,
  });

  const [type, name] = positionals;

  if (!type || !name) {
    console.error("Usage: agile.ts create <type> <name> [--title \"Title\"]");
    console.error("Types: feature, bug, task");
    process.exit(1);
  }

  if (!isValidTemplate(type)) {
    console.error(`Invalid type: "${type}". Valid types: ${TEMPLATES.join(", ")}`);
    process.exit(1);
  }

  const stage = values.stage as string;
  if (!isValidStage(stage)) {
    console.error(`Invalid stage: "${stage}". Valid stages: ${STAGES.join(", ")}`);
    process.exit(1);
  }

  const slug = toKebabCase(name);
  validateSlug(slug);

  const projectRoot = await findProjectRoot();

  // Check if issue already exists
  const existing = await findIssue(projectRoot, slug);
  if (existing) {
    console.error(`Issue already exists: ${existing.stage}/${existing.filename}`);
    process.exit(1);
  }

  // Read template
  const templatePath = join(projectRoot, TEMPLATES_DIR, `${type}.md`);
  let content: string;
  try {
    content = await readFile(templatePath, "utf-8");
  } catch {
    console.error(`Template not found: ${templatePath}`);
    process.exit(1);
  }

  // Replace placeholders
  const title = values.title || toTitleCase(slug);
  content = content.replace("[Title]", title);
  content = content.replace("YYYY-MM-DD", getCurrentDate());

  // Write issue
  const targetDir = join(projectRoot, AGILE_DIR, stage);
  await mkdir(targetDir, { recursive: true });
  const targetPath = join(targetDir, `${slug}.md`);
  await writeFile(targetPath, content);

  console.log(`Created: ${AGILE_DIR}/${stage}/${slug}.md (${type})`);
}

async function handleMove(args: string[]): Promise<void> {
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
  const issue = await findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  if (issue.stage === toStage) {
    console.log(`Issue is already in ${STAGE_NAMES[toStage]}`);
    return;
  }

  // Validate transition
  const allowedTransitions = VALID_TRANSITIONS[issue.stage];
  if (!allowedTransitions.includes(toStage)) {
    console.error(
      `Invalid transition: ${STAGE_NAMES[issue.stage]} -> ${STAGE_NAMES[toStage]}`
    );
    console.error(
      `Allowed from ${STAGE_NAMES[issue.stage]}: ${allowedTransitions.map((s) => STAGE_NAMES[s]).join(", ")}`
    );
    process.exit(1);
  }

  // Move the file
  const targetDir = join(projectRoot, AGILE_DIR, toStage);
  await mkdir(targetDir, { recursive: true });
  const targetPath = join(targetDir, issue.filename);
  await rename(issue.path, targetPath);

  console.log(`Moved: ${basename(issue.filename, ".md")}`);
  console.log(`  ${STAGE_NAMES[issue.stage]} -> ${STAGE_NAMES[toStage]}`);
}

async function handleList(args: string[]): Promise<void> {
  const { values } = parseArgs({
    args,
    options: {
      stage: { type: "string", short: "s" },
      format: { type: "string", short: "f", default: "table" },
    },
    allowPositionals: true,
  });

  const filterStage = values.stage;
  if (filterStage && !isValidStage(filterStage)) {
    console.error(`Invalid stage: "${filterStage}". Valid stages: ${STAGES.join(", ")}`);
    process.exit(1);
  }

  const format = values.format as string;
  if (format !== "table" && format !== "json") {
    console.error('Invalid format. Use "table" or "json"');
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const stagesToList = filterStage ? [filterStage as Stage] : STAGES;

  interface IssueInfo {
    name: string;
    stage: Stage;
    type: string;
    title: string;
    created: string;
    path: string;
  }

  const allIssues: IssueInfo[] = [];

  for (const stage of stagesToList) {
    const stagePath = join(projectRoot, AGILE_DIR, stage);
    let files: string[] = [];
    try {
      files = await readdir(stagePath);
    } catch {
      // Stage directory might not exist
    }

    for (const file of files) {
      if (file.endsWith(".md") && file !== ".gitkeep") {
        const filePath = join(stagePath, file);
        const meta = await parseIssueMeta(filePath);
        allIssues.push({
          name: basename(file, ".md"),
          stage,
          type: meta.type,
          title: meta.title || basename(file, ".md"),
          created: meta.created,
          path: `${AGILE_DIR}/${stage}/${file}`,
        });
      }
    }
  }

  if (format === "json") {
    console.log(JSON.stringify(allIssues, null, 2));
    return;
  }

  // Table format
  for (const stage of stagesToList) {
    const stageIssues = allIssues.filter((i) => i.stage === stage);
    console.log(`\n${STAGE_NAMES[stage]} (${stageIssues.length})`);
    console.log("─".repeat(50));

    if (stageIssues.length === 0) {
      console.log("  (empty)");
    } else {
      for (const issue of stageIssues) {
        const typeTag = `[${issue.type}]`.padEnd(10);
        console.log(`  ${typeTag} ${issue.name} - ${issue.title}`);
      }
    }
  }
  console.log();
}

async function handleArchive(args: string[]): Promise<void> {
  const { positionals } = parseArgs({
    args,
    allowPositionals: true,
  });

  const [name] = positionals;

  if (!name) {
    console.error("Usage: agile.ts archive <name>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const issue = await findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  // Create archive directory
  const archiveDir = join(projectRoot, ARCHIVE_DIR);
  await mkdir(archiveDir, { recursive: true });

  // Archive with timestamp to allow re-archiving
  const slug = basename(issue.filename, ".md");
  const timestamp = getCurrentDate();
  const archiveFilename = `${slug}-${timestamp}.md`;
  const archivePath = join(archiveDir, archiveFilename);

  await rename(issue.path, archivePath);

  console.log(`Archived: ${slug}`);
  console.log(`  -> ${ARCHIVE_DIR}/${archiveFilename}`);
}

async function handleDelete(args: string[]): Promise<void> {
  const { positionals } = parseArgs({
    args,
    allowPositionals: true,
  });

  const [name] = positionals;

  if (!name) {
    console.error("Usage: agile.ts delete <name>");
    process.exit(1);
  }

  const projectRoot = await findProjectRoot();
  const issue = await findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  await unlink(issue.path);

  const slug = basename(issue.filename, ".md");
  console.log(`Deleted: ${slug} (was in ${STAGE_NAMES[issue.stage]})`);
}

function handleHelp(args: string[]): void {
  const [command] = args;

  if (command) {
    switch (command) {
      case "create":
        console.log(`
Usage: agile.ts create <type> <name> [options]

Create a new issue from a template.

Arguments:
  type     Issue type: feature, bug, or task
  name     Kebab-case name for the issue (e.g., user-authentication)

Options:
  --title, -t    Human-readable title (defaults to name in Title Case)
  --stage, -s    Initial stage (default: 1-backlog)

Examples:
  agile.ts create feature user-auth --title "User Authentication"
  agile.ts create bug fix-login --stage 2-todo
`);
        break;
      case "move":
        console.log(`
Usage: agile.ts move <name> <stage>

Move an issue to a different workflow stage.

Arguments:
  name     Issue name (with or without .md extension)
  stage    Target stage: ${STAGES.join(", ")}

Workflow (only sequential transitions allowed):
  1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done

Examples:
  agile.ts move user-auth 2-todo
  agile.ts move fix-login 3-in-progress
`);
        break;
      case "list":
        console.log(`
Usage: agile.ts list [options]

List all issues or filter by stage.

Options:
  --stage, -s     Filter by stage
  --format, -f    Output format: table (default) or json

Examples:
  agile.ts list
  agile.ts list --stage 3-in-progress
  agile.ts list --format json
`);
        break;
      case "archive":
        console.log(`
Usage: agile.ts archive <name>

Archive an issue (move to agile/archive/ with timestamp).

Arguments:
  name     Issue name to archive

Examples:
  agile.ts archive completed-feature
`);
        break;
      case "delete":
        console.log(`
Usage: agile.ts delete <name>

Permanently delete an issue.

Arguments:
  name     Issue name to delete

Examples:
  agile.ts delete old-task
`);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        process.exit(1);
    }
    return;
  }

  console.log(`
Agile Workflow Manager

Usage: agile.ts <command> [options]

Commands:
  create <type> <name>    Create a new issue (feature, bug, or task)
  move <name> <stage>     Move an issue to a different stage
  list                    List all issues
  archive <name>          Archive an issue
  delete <name>           Permanently delete an issue
  help [command]          Show help for a command

Workflow Stages:
  1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done

Only sequential transitions are allowed (forward or back by one stage).

Examples:
  agile.ts create feature user-auth --title "User Authentication"
  agile.ts move user-auth 2-todo
  agile.ts list --stage 3-in-progress
  agile.ts archive completed-feature

Run "agile.ts help <command>" for more information on a command.
`);
}

// ============================================================================
// Main Entry Point
// ============================================================================

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0];
  const commandArgs = args.slice(1);

  try {
    switch (command) {
      case "create":
        await handleCreate(commandArgs);
        break;
      case "move":
        await handleMove(commandArgs);
        break;
      case "list":
        await handleList(commandArgs);
        break;
      case "archive":
        await handleArchive(commandArgs);
        break;
      case "delete":
        await handleDelete(commandArgs);
        break;
      case "help":
      case "--help":
      case "-h":
      case undefined:
        handleHelp(commandArgs);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        console.error('Run "agile.ts help" to see available commands');
        process.exit(1);
    }
  } catch (error) {
    if (error instanceof Error) {
      console.error(`Error: ${error.message}`);
    } else {
      console.error("An unexpected error occurred");
    }
    process.exit(1);
  }
}

main();

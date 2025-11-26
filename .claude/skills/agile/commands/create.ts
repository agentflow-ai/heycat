import { parseArgs } from "node:util";
import {
  STAGES,
  TEMPLATES,
  AGILE_DIR,
  type Stage,
} from "../lib/types";
import {
  isValidStage,
  isValidTemplate,
  toKebabCase,
  findProjectRoot,
} from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";

export async function handleCreate(args: string[]): Promise<void> {
  const { values, positionals } = parseArgs({
    args,
    options: {
      title: { type: "string", short: "t" },
      owner: { type: "string", short: "o" },
      stage: { type: "string", short: "s", default: "1-backlog" },
    },
    allowPositionals: true,
  });

  const [type, name] = positionals;

  if (!type || !name) {
    console.error("Usage: agile.ts create <type> <name> [--title \"Title\"] [--owner \"Name\"] [--stage <stage>]");
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
  const projectRoot = await findProjectRoot();
  const resolver = createIssueResolver();

  try {
    const issue = await resolver.createIssue(projectRoot, type, slug, {
      title: values.title,
      owner: values.owner,
      stage: stage as Stage,
    });

    console.log(`Created: ${AGILE_DIR}/${issue.stage}/${issue.name}/`);
    console.log(`  - ${issue.type}.md (main spec)`);
    console.log(`  - technical-guidance.md`);
    console.log(`\nNext steps:`);
    console.log(`  1. Edit ${issue.mainFilePath} to fill in description`);
    console.log(`  2. Run 'agile.ts spec suggest ${issue.name}' to generate specs`);
  } catch (err) {
    console.error(`Error: ${(err as Error).message}`);
    process.exit(1);
  }
}

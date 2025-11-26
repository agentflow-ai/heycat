import { parseArgs } from "node:util";
import { STAGE_NAMES } from "../lib/types";
import { findProjectRoot } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";

export async function handleDelete(args: string[]): Promise<void> {
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
  const resolver = createIssueResolver();
  const issue = await resolver.findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  await resolver.deleteIssue(issue);

  console.log(`Deleted: ${issue.name} (was in ${STAGE_NAMES[issue.stage]})`);
  console.log(`  Removed folder and all specs`);
}

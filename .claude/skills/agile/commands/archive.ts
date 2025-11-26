import { parseArgs } from "node:util";
import { ARCHIVE_DIR } from "../lib/types";
import { findProjectRoot } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";

export async function handleArchive(args: string[]): Promise<void> {
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
  const resolver = createIssueResolver();
  const issue = await resolver.findIssue(projectRoot, name);

  if (!issue) {
    console.error(`Issue not found: ${name}`);
    process.exit(1);
  }

  const archivePath = await resolver.archiveIssue(projectRoot, issue);

  console.log(`Archived: ${issue.name}`);
  console.log(`  -> ${archivePath.replace(projectRoot + "/", "")}`);
}

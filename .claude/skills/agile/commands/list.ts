import { parseArgs } from "node:util";
import {
  STAGES,
  STAGE_NAMES,
  AGILE_DIR,
  type Stage,
  type Issue,
} from "../lib/types";
import { isValidStage, findProjectRoot } from "../lib/utils";
import { createIssueResolver } from "../lib/issue-resolver";
import { createSpecManager } from "../lib/spec-manager";

interface IssueListInfo {
  name: string;
  stage: Stage;
  type: string;
  title: string;
  created: string;
  owner: string;
  path: string;
  specsCompleted: number;
  specsTotal: number;
}

export async function handleList(args: string[]): Promise<void> {
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
  const resolver = createIssueResolver();
  const specManager = createSpecManager();

  const issues = await resolver.listIssues(
    projectRoot,
    filterStage as Stage | undefined
  );

  const allIssues: IssueListInfo[] = [];

  for (const issue of issues) {
    const specStatus = await specManager.getCompletionStatus(issue);

    allIssues.push({
      name: issue.name,
      stage: issue.stage,
      type: issue.type,
      title: issue.meta.title || issue.name,
      created: issue.meta.created,
      owner: issue.meta.owner,
      path: `${AGILE_DIR}/${issue.stage}/${issue.name}/`,
      specsCompleted: specStatus.completed,
      specsTotal: specStatus.total,
    });
  }

  if (format === "json") {
    console.log(JSON.stringify(allIssues, null, 2));
    return;
  }

  // Table format
  const stagesToList = filterStage ? [filterStage as Stage] : STAGES;

  for (const stage of stagesToList) {
    const stageIssues = allIssues.filter((i) => i.stage === stage);
    console.log(`\n${STAGE_NAMES[stage]} (${stageIssues.length})`);
    console.log("─".repeat(60));

    if (stageIssues.length === 0) {
      console.log("  (empty)");
    } else {
      for (const issue of stageIssues) {
        const typeTag = `[${issue.type}]`.padEnd(10);
        const ownerTag = issue.owner && issue.owner !== "[Name]" ? ` (${issue.owner})` : "";
        const specTag = issue.specsTotal > 0
          ? ` [${issue.specsCompleted}/${issue.specsTotal} specs]`
          : "";
        console.log(`  ${typeTag} ${issue.name} - ${issue.title}${specTag}${ownerTag}`);
      }
    }
  }
  console.log();
}

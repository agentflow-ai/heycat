import { STAGES, STAGE_NAMES, SPEC_STATUSES, GUIDANCE_STATUSES } from "../lib/types";

export function handleHelp(args: string[]): void {
  const [command] = args;

  if (command) {
    switch (command) {
      case "create":
        console.log(`
Usage: agile.ts create <type> <name> [options]

Create a new folder-based issue from a template.

Arguments:
  type     Issue type: feature, bug, or task
  name     Kebab-case name for the issue (e.g., user-authentication)

Options:
  --title, -t    Human-readable title (defaults to name in Title Case)
  --owner, -o    Issue owner/assignee name
  --stage, -s    Initial stage (default: 1-backlog)

Creates:
  agile/<stage>/<name>/
    - <type>.md              Main issue spec
    - technical-guidance.md  Technical investigation document

Examples:
  agile.ts create feature user-auth --title "User Authentication" --owner "Alice"
  agile.ts create bug fix-login --stage 2-todo --owner "Bob"
`);
        break;
      case "move":
        console.log(`
Usage: agile.ts move <name> <stage>

Move an issue folder to a different workflow stage.

Arguments:
  name     Issue name
  stage    Target stage: ${STAGES.join(", ")}

Workflow (only sequential transitions allowed):
  1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done

Validation Requirements:
  - 2-todo: Description must be complete
  - 3-in-progress: Owner assigned, technical guidance exists
  - 4-review: All specs completed, guidance updated
  - 5-done: All Definition of Done items checked

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

Archive an issue folder (move to agile/archive/<name>-<date>/).

Arguments:
  name     Issue name to archive

Examples:
  agile.ts archive completed-feature
`);
        break;
      case "delete":
        console.log(`
Usage: agile.ts delete <name>

Permanently delete an issue folder and all its specs.

Arguments:
  name     Issue name to delete

Examples:
  agile.ts delete old-task
`);
        break;
      case "work":
        console.log(`
Usage: agile.ts work <name>

Analyze an issue and get stage-appropriate guidance.

Arguments:
  name     Issue name to analyze

Output includes:
  - Issue metadata (type, stage, owner, created date)
  - Specs status (pending, in-progress, completed)
  - Technical guidance status
  - Incomplete sections with placeholder text
  - Definition of Done progress
  - Stage-specific guidance and suggested actions
  - Readiness status for advancing to the next stage

Examples:
  agile.ts work user-auth
  agile.ts work dark-mode
`);
        break;
      case "spec":
        console.log(`
Usage: agile.ts spec <subcommand> [options]

Manage specs within an issue folder.

Subcommands:
  list <issue>                          List all specs in an issue
  add <issue> <name> [--title "..."]    Add a new spec
  status <issue> <spec> <status>        Update spec status
  delete <issue> <spec>                 Delete a spec
  suggest <issue>                       AI-assisted spec breakdown

Statuses: ${SPEC_STATUSES.join(", ")}

Note: Completing a spec requires technical guidance to be updated first.

Examples:
  agile.ts spec list user-auth
  agile.ts spec add user-auth login-flow --title "Implement Login Flow"
  agile.ts spec status user-auth login-flow in-progress
  agile.ts spec status user-auth login-flow completed
  agile.ts spec delete user-auth unused-spec
  agile.ts spec suggest user-auth
`);
        break;
      case "guidance":
        console.log(`
Usage: agile.ts guidance <subcommand> [options]

Manage technical guidance for an issue.

Subcommands:
  show <issue>                  Show technical guidance summary
  update <issue>                Mark guidance as updated (set timestamp)
  validate <issue>              Check if guidance is current
  status <issue> <status>       Set guidance status

Statuses: ${GUIDANCE_STATUSES.join(", ")}

Technical guidance must be updated before completing specs.

Examples:
  agile.ts guidance show user-auth
  agile.ts guidance update user-auth
  agile.ts guidance validate user-auth
  agile.ts guidance status user-auth active
`);
        break;
      default:
        console.error(`Unknown command: ${command}`);
        process.exit(1);
    }
    return;
  }

  console.log(`
Agile Workflow Manager (Folder-Based Issues with SPS Specs)

Usage: agile.ts <command> [options]

Issue Commands:
  create <type> <name>    Create a new issue folder (feature, bug, or task)
  move <name> <stage>     Move an issue to a different stage
  list                    List all issues with spec progress
  work <name>             Analyze an issue and get stage-appropriate guidance
  archive <name>          Archive an issue folder
  delete <name>           Permanently delete an issue folder

Spec Commands:
  spec list <issue>             List specs in an issue
  spec add <issue> <name>       Add a new spec
  spec status <issue> <s> <st>  Update spec status (pending/in-progress/completed)
  spec delete <issue> <spec>    Delete a spec
  spec suggest <issue>          AI-assisted spec breakdown

Guidance Commands:
  guidance show <issue>         Show technical guidance
  guidance update <issue>       Mark guidance as updated
  guidance validate <issue>     Check if guidance is current

Workflow:
  1-backlog -> 2-todo -> 3-in-progress -> 4-review -> 5-done

Issue Structure:
  agile/<stage>/<issue-name>/
    - feature.md (or bug.md/task.md)
    - technical-guidance.md
    - *.spec.md (SPS spec files)

SPS Pattern (Smallest Possible Spec):
  Each spec should be the smallest deliverable unit - roughly the size
  of one "todo" item. All specs must be completed before moving to review.

Examples:
  agile.ts create feature user-auth --title "User Authentication" --owner "Alice"
  agile.ts spec suggest user-auth
  agile.ts spec status user-auth login-flow in-progress
  agile.ts guidance update user-auth
  agile.ts move user-auth 4-review

Run "agile.ts help <command>" for more information on a command.
`);
}

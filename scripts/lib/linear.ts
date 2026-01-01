/**
 * Linear API utilities for heycat scripts.
 *
 * Provides functions to query Linear for issue information,
 * particularly the Linear identifier (e.g., HEY-123) for an issue.
 */

import { LinearClient } from "@linear/sdk";
import { resolve } from "path";

/**
 * Issue information returned by Linear queries.
 */
export interface LinearIssueInfo {
  identifier: string;
  title: string;
  id: string;
}

/**
 * Convert a title to a slug format for matching.
 * Normalizes to lowercase, replaces non-alphanumeric with dashes,
 * and removes leading/trailing dashes.
 */
export function titleToSlug(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

/**
 * Load Linear team ID from devloop.config.ts or environment variable.
 */
async function getTeamId(): Promise<string | null> {
  // First try environment variable
  if (process.env.LINEAR_TEAM_ID) {
    return process.env.LINEAR_TEAM_ID;
  }

  // Fall back to devloop.config.ts
  try {
    const configPath = resolve(process.cwd(), "devloop.config.ts");
    const config = await import(configPath);
    return config.default?.agile?.linear?.teamId ?? null;
  } catch {
    return null;
  }
}

/**
 * Find an issue by slug or identifier.
 * This is the core lookup function used by both getLinearIdentifier and validateLinearIssue.
 *
 * @param client - LinearClient instance
 * @param teamId - Team ID to search within
 * @param issueSlug - The issue slug or HEY-### identifier
 * @returns Issue info if found, null otherwise
 */
async function findIssueBySlug(
  client: LinearClient,
  teamId: string,
  issueSlug: string
): Promise<LinearIssueInfo | null> {
  // If issueSlug looks like a Linear identifier (HEY-123), look it up directly
  if (/^HEY-\d+$/i.test(issueSlug)) {
    const issue = await client.issue(issueSlug.toUpperCase());
    if (issue) {
      return {
        identifier: issue.identifier,
        title: issue.title,
        id: issue.id,
      };
    }
    return null;
  }

  // Search for issues in the team that match the slug
  // Use first: 100 to limit results and improve performance
  const issues = await client.issues({
    first: 100,
    filter: {
      team: { id: { eq: teamId } },
    },
  });

  // Find the issue with matching slug
  for (const issue of issues.nodes) {
    const issueSlugFromTitle = titleToSlug(issue.title);

    if (
      issueSlugFromTitle === issueSlug ||
      issue.title.toLowerCase().replace(/\s+/g, "-") === issueSlug
    ) {
      return {
        identifier: issue.identifier,
        title: issue.title,
        id: issue.id,
      };
    }
  }

  return null;
}

/**
 * Get the Linear identifier (e.g., "HEY-123") for an issue given its slug.
 *
 * @param issueSlug - The issue slug (e.g., "docker-development-workflow")
 * @returns The Linear identifier (e.g., "HEY-42") or null if not found
 *
 * @example
 * const identifier = await getLinearIdentifier("docker-development-workflow");
 * // Returns: "HEY-42"
 */
export async function getLinearIdentifier(issueSlug: string): Promise<string | null> {
  const apiKey = process.env.LINEAR_API_KEY;
  if (!apiKey) {
    console.error("LINEAR_API_KEY environment variable is not set");
    return null;
  }

  const teamId = await getTeamId();
  if (!teamId) {
    console.error("LINEAR_TEAM_ID not set (check env or devloop.config.ts)");
    return null;
  }

  const client = new LinearClient({ apiKey });

  try {
    const issue = await findIssueBySlug(client, teamId, issueSlug);
    return issue?.identifier ?? null;
  } catch (error) {
    console.error("Error querying Linear:", error);
    return null;
  }
}

/**
 * Validate that an issue exists in Linear.
 *
 * @param issueSlug - The issue slug or identifier
 * @returns Object with issue info if found, null otherwise
 */
export async function validateLinearIssue(issueSlug: string): Promise<LinearIssueInfo | null> {
  const apiKey = process.env.LINEAR_API_KEY;
  if (!apiKey) {
    console.error("LINEAR_API_KEY environment variable is not set");
    return null;
  }

  const teamId = await getTeamId();
  if (!teamId) {
    console.error("LINEAR_TEAM_ID not set (check env or devloop.config.ts)");
    return null;
  }

  const client = new LinearClient({ apiKey });

  try {
    return await findIssueBySlug(client, teamId, issueSlug);
  } catch (error) {
    console.error("Error querying Linear:", error);
    return null;
  }
}

// CLI usage: query Linear identifier for an issue
if (import.meta.main) {
  const slug = process.argv[2];
  if (!slug) {
    console.log("Usage: bun scripts/lib/linear.ts <issue-slug>");
    console.log("Example: bun scripts/lib/linear.ts docker-development-workflow");
    process.exit(1);
  }

  const result = await validateLinearIssue(slug);
  if (result) {
    console.log(`Issue found: ${result.identifier}`);
    console.log(`Title: ${result.title}`);
    console.log(`ID: ${result.id}`);
  } else {
    console.log(`Issue not found: ${slug}`);
    process.exit(1);
  }
}

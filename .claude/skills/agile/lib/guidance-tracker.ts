import { readFile, stat } from "node:fs/promises";
import {
  GUIDANCE_STATUSES,
  type Issue,
  type TechnicalGuidanceMeta,
  type GuidanceFrontmatter,
  type GuidanceStatus,
  type SpecInfo,
} from "./types";
import { getCurrentDate } from "./utils";

// ============================================================================
// Guidance Tracker Interface
// ============================================================================

export interface GuidanceTracker {
  getGuidanceMeta(issue: Issue): Promise<TechnicalGuidanceMeta | null>;
  exists(issue: Issue): Promise<boolean>;
  markUpdated(issue: Issue): Promise<void>;
  setStatus(issue: Issue, status: GuidanceStatus): Promise<void>;
  needsUpdate(issue: Issue, specs: SpecInfo[]): Promise<boolean>;
  validateForSpecCompletion(issue: Issue, spec: SpecInfo): Promise<GuidanceValidationResult>;
}

export interface GuidanceValidationResult {
  valid: boolean;
  message: string;
  guidanceLastUpdated: string | null;
  specStarted: string;
}

// ============================================================================
// Guidance Tracker Implementation
// ============================================================================

export class FolderGuidanceTracker implements GuidanceTracker {
  /**
   * Get technical guidance metadata for an issue
   */
  async getGuidanceMeta(issue: Issue): Promise<TechnicalGuidanceMeta | null> {
    try {
      await stat(issue.technicalGuidancePath);
      const content = await readFile(issue.technicalGuidancePath, "utf-8");

      const frontmatter = this.parseFrontmatter(content);
      const hasInvestigationLog = this.hasInvestigationLog(content);
      const openQuestionsCount = this.countOpenQuestions(content);

      return {
        path: issue.technicalGuidancePath,
        frontmatter,
        hasInvestigationLog,
        openQuestionsCount,
      };
    } catch {
      return null;
    }
  }

  /**
   * Check if technical guidance file exists
   */
  async exists(issue: Issue): Promise<boolean> {
    try {
      await stat(issue.technicalGuidancePath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Mark technical guidance as updated (sets timestamp)
   */
  async markUpdated(issue: Issue): Promise<void> {
    const content = await readFile(issue.technicalGuidancePath, "utf-8");
    const date = getCurrentDate();

    const updatedContent = content.replace(
      /^last-updated:\s*.+$/m,
      `last-updated: ${date}`
    );

    await Bun.write(issue.technicalGuidancePath, updatedContent);
  }

  /**
   * Set guidance status (draft, active, finalized)
   */
  async setStatus(issue: Issue, status: GuidanceStatus): Promise<void> {
    if (!GUIDANCE_STATUSES.includes(status)) {
      throw new Error(`Invalid status: "${status}". Must be one of: ${GUIDANCE_STATUSES.join(", ")}`);
    }

    const content = await readFile(issue.technicalGuidancePath, "utf-8");

    const updatedContent = content.replace(
      /^status:\s*.+$/m,
      `status: ${status}`
    );

    await Bun.write(issue.technicalGuidancePath, updatedContent);
  }

  /**
   * Check if guidance needs update (completed specs after last guidance update)
   */
  async needsUpdate(issue: Issue, specs: SpecInfo[]): Promise<boolean> {
    const meta = await this.getGuidanceMeta(issue);
    if (!meta) return true;

    const guidanceDate = meta.frontmatter.lastUpdated;

    // Check if any completed spec has a completion date after guidance update
    for (const spec of specs) {
      if (spec.frontmatter.status === "completed" && spec.frontmatter.completed) {
        if (spec.frontmatter.completed > guidanceDate) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Validate that guidance was updated before completing a spec
   */
  async validateForSpecCompletion(
    issue: Issue,
    spec: SpecInfo
  ): Promise<GuidanceValidationResult> {
    const meta = await this.getGuidanceMeta(issue);

    if (!meta) {
      return {
        valid: false,
        message: "Technical guidance file does not exist",
        guidanceLastUpdated: null,
        specStarted: spec.frontmatter.created,
      };
    }

    const guidanceDate = meta.frontmatter.lastUpdated;
    const specStarted = spec.frontmatter.created;

    // Guidance must be updated after the spec was started
    if (guidanceDate < specStarted) {
      return {
        valid: false,
        message: `Technical guidance must be updated after spec started. ` +
          `Guidance last updated: ${guidanceDate}, Spec started: ${specStarted}`,
        guidanceLastUpdated: guidanceDate,
        specStarted,
      };
    }

    return {
      valid: true,
      message: "Technical guidance is up to date",
      guidanceLastUpdated: guidanceDate,
      specStarted,
    };
  }

  // ============================================================================
  // Private Helpers
  // ============================================================================

  /**
   * Parse YAML frontmatter from guidance content
   */
  private parseFrontmatter(content: string): GuidanceFrontmatter {
    const defaults: GuidanceFrontmatter = {
      lastUpdated: getCurrentDate(),
      status: "draft",
    };

    // Match frontmatter block
    const fmMatch = content.match(/^---\n([\s\S]*?)\n---/);
    if (!fmMatch) return defaults;

    const fmContent = fmMatch[1];

    // Parse last-updated
    const dateMatch = fmContent.match(/^last-updated:\s*(\d{4}-\d{2}-\d{2})/m);
    if (dateMatch) {
      defaults.lastUpdated = dateMatch[1];
    }

    // Parse status
    const statusMatch = fmContent.match(/^status:\s*(.+)$/m);
    if (statusMatch) {
      const status = statusMatch[1].trim() as GuidanceStatus;
      if (GUIDANCE_STATUSES.includes(status)) {
        defaults.status = status;
      }
    }

    return defaults;
  }

  /**
   * Check if content has investigation log entries
   */
  private hasInvestigationLog(content: string): boolean {
    // Look for Investigation Log section with table entries
    const logSection = content.match(/## Investigation Log[\s\S]*?(?=##|$)/);
    if (!logSection) return false;

    // Check for table rows (lines starting with |)
    const tableRows = logSection[0].match(/^\|[^|]+\|[^|]+\|[^|]+\|$/gm);
    // Filter out header and separator rows
    const dataRows = tableRows?.filter(
      (row) => !row.includes("---") && !row.includes("Date") && !row.includes("Finding")
    );

    return (dataRows?.length || 0) > 0;
  }

  /**
   * Count open questions (unchecked checkboxes in Open Questions section)
   */
  private countOpenQuestions(content: string): number {
    const questionsSection = content.match(/## Open Questions[\s\S]*?(?=##|$)/);
    if (!questionsSection) return 0;

    const unchecked = questionsSection[0].match(/- \[ \]/g);
    return unchecked?.length || 0;
  }
}

// ============================================================================
// Default Export
// ============================================================================

export function createGuidanceTracker(): GuidanceTracker {
  return new FolderGuidanceTracker();
}

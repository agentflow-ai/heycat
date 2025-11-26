import { readdir, readFile, mkdir, rename, rm, stat, copyFile } from "node:fs/promises";
import { join, basename } from "node:path";
import {
  STAGES,
  TEMPLATES,
  AGILE_DIR,
  ARCHIVE_DIR,
  TEMPLATES_DIR,
  type Stage,
  type Template,
  type Issue,
  type IssueMeta,
  type CreateOptions,
} from "./types";
import { validateSlug, toTitleCase, getCurrentDate } from "./utils";

// ============================================================================
// Issue Resolver Interface
// ============================================================================

export interface IssueResolver {
  findIssue(projectRoot: string, name: string): Promise<Issue | null>;
  listIssues(projectRoot: string, stage?: Stage): Promise<Issue[]>;
  createIssue(projectRoot: string, type: Template, name: string, options?: CreateOptions): Promise<Issue>;
  moveIssue(projectRoot: string, issue: Issue, toStage: Stage): Promise<Issue>;
  archiveIssue(projectRoot: string, issue: Issue): Promise<string>;
  deleteIssue(issue: Issue): Promise<void>;
}

// ============================================================================
// Folder Issue Resolver Implementation
// ============================================================================

export class FolderIssueResolver implements IssueResolver {
  /**
   * Find an issue by name across all stages
   */
  async findIssue(projectRoot: string, name: string): Promise<Issue | null> {
    const slug = name.replace(/\.md$/, "").replace(/\/$/, "");

    for (const stage of STAGES) {
      const stagePath = join(projectRoot, AGILE_DIR, stage);
      const folderPath = join(stagePath, slug);

      try {
        const stats = await stat(folderPath);
        if (stats.isDirectory()) {
          const issue = await this.resolveIssueFromFolder(folderPath, stage, slug);
          if (issue) return issue;
        }
      } catch {
        // Folder doesn't exist in this stage
      }
    }

    return null;
  }

  /**
   * List all issues, optionally filtered by stage
   */
  async listIssues(projectRoot: string, stage?: Stage): Promise<Issue[]> {
    const issues: Issue[] = [];
    const stagesToSearch = stage ? [stage] : STAGES;

    for (const s of stagesToSearch) {
      const stagePath = join(projectRoot, AGILE_DIR, s);

      try {
        const entries = await readdir(stagePath, { withFileTypes: true });

        for (const entry of entries) {
          if (entry.isDirectory()) {
            const folderPath = join(stagePath, entry.name);
            const issue = await this.resolveIssueFromFolder(folderPath, s, entry.name);
            if (issue) {
              issues.push(issue);
            }
          }
        }
      } catch {
        // Stage directory might not exist
      }
    }

    return issues;
  }

  /**
   * Create a new folder-based issue
   */
  async createIssue(
    projectRoot: string,
    type: Template,
    name: string,
    options: CreateOptions = {}
  ): Promise<Issue> {
    validateSlug(name);

    // Check for duplicate across all stages
    const existing = await this.findIssue(projectRoot, name);
    if (existing) {
      throw new Error(`Issue "${name}" already exists in ${existing.stage}`);
    }

    const stage = options.stage || "1-backlog";
    const title = options.title || toTitleCase(name);
    const owner = options.owner || "[Name]";
    const date = getCurrentDate();

    // Create folder structure
    const stagePath = join(projectRoot, AGILE_DIR, stage);
    const folderPath = join(stagePath, name);

    await mkdir(folderPath, { recursive: true });

    // Read and process main template
    const mainTemplatePath = join(projectRoot, TEMPLATES_DIR, type, `${type}.md`);
    let mainContent: string;

    try {
      mainContent = await readFile(mainTemplatePath, "utf-8");
    } catch {
      throw new Error(`Template not found: ${mainTemplatePath}`);
    }

    mainContent = mainContent
      .replace(/\[Title\]/g, title)
      .replace(/YYYY-MM-DD/g, date)
      .replace(/\[Name\]/g, owner);

    const mainFilePath = join(folderPath, `${type}.md`);
    await Bun.write(mainFilePath, mainContent);

    // Read and process technical guidance template
    const guidanceTemplatePath = join(projectRoot, TEMPLATES_DIR, type, "technical-guidance.md");
    let guidanceContent: string;

    try {
      guidanceContent = await readFile(guidanceTemplatePath, "utf-8");
    } catch {
      throw new Error(`Technical guidance template not found: ${guidanceTemplatePath}`);
    }

    guidanceContent = guidanceContent
      .replace(/\[Issue Name\]/g, title)
      .replace(/YYYY-MM-DD/g, date);

    const guidancePath = join(folderPath, "technical-guidance.md");
    await Bun.write(guidancePath, guidanceContent);

    // Parse meta from created content
    const meta = await this.parseIssueMeta(mainFilePath);

    return {
      name,
      stage,
      type,
      path: folderPath,
      mainFilePath,
      technicalGuidancePath: guidancePath,
      meta,
    };
  }

  /**
   * Move an issue to a different stage
   */
  async moveIssue(projectRoot: string, issue: Issue, toStage: Stage): Promise<Issue> {
    const newFolderPath = join(projectRoot, AGILE_DIR, toStage, issue.name);

    await rename(issue.path, newFolderPath);

    return {
      ...issue,
      stage: toStage,
      path: newFolderPath,
      mainFilePath: join(newFolderPath, basename(issue.mainFilePath)),
      technicalGuidancePath: join(newFolderPath, "technical-guidance.md"),
    };
  }

  /**
   * Archive an issue with timestamp
   */
  async archiveIssue(projectRoot: string, issue: Issue): Promise<string> {
    const date = getCurrentDate();
    const archiveName = `${issue.name}-${date}`;
    const archivePath = join(projectRoot, ARCHIVE_DIR, archiveName);

    // Ensure archive directory exists
    await mkdir(join(projectRoot, ARCHIVE_DIR), { recursive: true });

    await rename(issue.path, archivePath);

    return archivePath;
  }

  /**
   * Delete an issue permanently
   */
  async deleteIssue(issue: Issue): Promise<void> {
    await rm(issue.path, { recursive: true, force: true });
  }

  // ============================================================================
  // Private Helpers
  // ============================================================================

  /**
   * Resolve an Issue object from a folder path
   */
  private async resolveIssueFromFolder(
    folderPath: string,
    stage: Stage,
    name: string
  ): Promise<Issue | null> {
    // Look for main file (feature.md, bug.md, or task.md)
    for (const template of TEMPLATES) {
      const mainFilePath = join(folderPath, `${template}.md`);
      try {
        await stat(mainFilePath);

        const meta = await this.parseIssueMeta(mainFilePath);
        const technicalGuidancePath = join(folderPath, "technical-guidance.md");

        return {
          name,
          stage,
          type: template,
          path: folderPath,
          mainFilePath,
          technicalGuidancePath,
          meta,
        };
      } catch {
        // This template type doesn't exist
      }
    }

    return null;
  }

  /**
   * Parse issue metadata from the main file
   */
  private async parseIssueMeta(filePath: string): Promise<IssueMeta> {
    const content = await readFile(filePath, "utf-8");
    const lines = content.split("\n");

    let title = "";
    let type: Template | "unknown" = "unknown";
    let created = "";
    let owner = "";

    // Parse first line for type and title
    const firstLine = lines[0] || "";
    const titleMatch = firstLine.match(/^#\s+(Feature|Bug|Task):\s*(.*)$/i);
    if (titleMatch) {
      type = titleMatch[1].toLowerCase() as Template;
      title = titleMatch[2].trim();
    }

    // Parse created date and owner
    for (const line of lines) {
      const dateMatch = line.match(/\*\*Created:\*\*\s*(\d{4}-\d{2}-\d{2})/);
      if (dateMatch) {
        created = dateMatch[1];
      }
      const ownerMatch = line.match(/\*\*Owner:\*\*\s*(.+)$/);
      if (ownerMatch) {
        owner = ownerMatch[1].trim();
      }
    }

    return { title, type, created, owner };
  }
}

// ============================================================================
// Default Export
// ============================================================================

export function createIssueResolver(): IssueResolver {
  return new FolderIssueResolver();
}

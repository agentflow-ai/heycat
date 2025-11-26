import { readdir, readFile, unlink, stat } from "node:fs/promises";
import { join, basename } from "node:path";
import {
  TEMPLATES_DIR,
  SPEC_STATUSES,
  type Issue,
  type SpecInfo,
  type SpecFrontmatter,
  type SpecStatus,
} from "./types";
import { validateSlug, toTitleCase, getCurrentDate } from "./utils";

// ============================================================================
// Spec Manager Interface
// ============================================================================

export interface SpecManager {
  listSpecs(issue: Issue): Promise<SpecInfo[]>;
  getSpec(issue: Issue, specName: string): Promise<SpecInfo | null>;
  createSpec(projectRoot: string, issue: Issue, name: string, title?: string): Promise<SpecInfo>;
  updateStatus(spec: SpecInfo, status: SpecStatus): Promise<SpecInfo>;
  deleteSpec(spec: SpecInfo): Promise<void>;
  getCompletionStatus(issue: Issue): Promise<SpecCompletionStatus>;
}

export interface SpecCompletionStatus {
  total: number;
  pending: number;
  inProgress: number;
  completed: number;
  allCompleted: boolean;
  specs: SpecInfo[];
}

// ============================================================================
// Spec Manager Implementation
// ============================================================================

export class FolderSpecManager implements SpecManager {
  /**
   * List all specs in an issue folder
   */
  async listSpecs(issue: Issue): Promise<SpecInfo[]> {
    const specs: SpecInfo[] = [];

    try {
      const entries = await readdir(issue.path);

      for (const entry of entries) {
        if (entry.endsWith(".spec.md")) {
          const specPath = join(issue.path, entry);
          const spec = await this.parseSpec(specPath);
          if (spec) {
            specs.push(spec);
          }
        }
      }
    } catch {
      // Issue folder might not exist or be accessible
    }

    // Sort by status (in-progress first, then pending, then completed) and name
    return specs.sort((a, b) => {
      const statusOrder = { "in-progress": 0, pending: 1, completed: 2 };
      const statusDiff = statusOrder[a.frontmatter.status] - statusOrder[b.frontmatter.status];
      if (statusDiff !== 0) return statusDiff;
      return a.name.localeCompare(b.name);
    });
  }

  /**
   * Get a specific spec by name
   */
  async getSpec(issue: Issue, specName: string): Promise<SpecInfo | null> {
    const slug = specName.replace(/\.spec\.md$/, "");
    const specPath = join(issue.path, `${slug}.spec.md`);

    try {
      await stat(specPath);
      return await this.parseSpec(specPath);
    } catch {
      return null;
    }
  }

  /**
   * Create a new spec in an issue folder
   */
  async createSpec(
    projectRoot: string,
    issue: Issue,
    name: string,
    title?: string
  ): Promise<SpecInfo> {
    validateSlug(name);

    const specPath = join(issue.path, `${name}.spec.md`);

    // Check if spec already exists
    try {
      await stat(specPath);
      throw new Error(`Spec "${name}" already exists in issue "${issue.name}"`);
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code !== "ENOENT") {
        throw err;
      }
    }

    // Read spec template
    const templatePath = join(projectRoot, TEMPLATES_DIR, "spec.template.md");
    let content: string;

    try {
      content = await readFile(templatePath, "utf-8");
    } catch {
      throw new Error(`Spec template not found: ${templatePath}`);
    }

    const specTitle = title || toTitleCase(name);
    const date = getCurrentDate();

    content = content
      .replace(/\[Title\]/g, specTitle)
      .replace(/YYYY-MM-DD/g, date);

    await Bun.write(specPath, content);

    const spec = await this.parseSpec(specPath);
    if (!spec) {
      throw new Error("Failed to parse newly created spec");
    }

    return spec;
  }

  /**
   * Update a spec's status
   */
  async updateStatus(spec: SpecInfo, status: SpecStatus): Promise<SpecInfo> {
    if (!SPEC_STATUSES.includes(status)) {
      throw new Error(`Invalid status: "${status}". Must be one of: ${SPEC_STATUSES.join(", ")}`);
    }

    const content = await readFile(spec.path, "utf-8");
    const date = getCurrentDate();

    // Update frontmatter
    let updatedContent = content;

    // Update status
    updatedContent = updatedContent.replace(
      /^status:\s*.+$/m,
      `status: ${status}`
    );

    // Update completed date if completing
    if (status === "completed") {
      updatedContent = updatedContent.replace(
        /^completed:\s*.+$/m,
        `completed: ${date}`
      );
    } else {
      updatedContent = updatedContent.replace(
        /^completed:\s*.+$/m,
        "completed: null"
      );
    }

    await Bun.write(spec.path, updatedContent);

    // Return updated spec info
    const updatedSpec = await this.parseSpec(spec.path);
    if (!updatedSpec) {
      throw new Error("Failed to parse updated spec");
    }

    return updatedSpec;
  }

  /**
   * Delete a spec
   */
  async deleteSpec(spec: SpecInfo): Promise<void> {
    await unlink(spec.path);
  }

  /**
   * Get completion status summary for an issue
   */
  async getCompletionStatus(issue: Issue): Promise<SpecCompletionStatus> {
    const specs = await this.listSpecs(issue);

    const pending = specs.filter((s) => s.frontmatter.status === "pending").length;
    const inProgress = specs.filter((s) => s.frontmatter.status === "in-progress").length;
    const completed = specs.filter((s) => s.frontmatter.status === "completed").length;

    return {
      total: specs.length,
      pending,
      inProgress,
      completed,
      allCompleted: specs.length > 0 && pending === 0 && inProgress === 0,
      specs,
    };
  }

  // ============================================================================
  // Private Helpers
  // ============================================================================

  /**
   * Parse a spec file into SpecInfo
   */
  private async parseSpec(specPath: string): Promise<SpecInfo | null> {
    try {
      const content = await readFile(specPath, "utf-8");
      const name = basename(specPath).replace(/\.spec\.md$/, "");

      // Parse YAML frontmatter
      const frontmatter = this.parseFrontmatter(content);

      // Parse title from content
      const titleMatch = content.match(/^#\s+Spec:\s*(.+)$/m);
      const title = titleMatch ? titleMatch[1].trim() : toTitleCase(name);

      return {
        name,
        path: specPath,
        frontmatter,
        title,
      };
    } catch {
      return null;
    }
  }

  /**
   * Parse YAML frontmatter from spec content
   */
  private parseFrontmatter(content: string): SpecFrontmatter {
    const defaults: SpecFrontmatter = {
      status: "pending",
      created: getCurrentDate(),
      completed: null,
      dependencies: [],
    };

    // Match frontmatter block
    const fmMatch = content.match(/^---\n([\s\S]*?)\n---/);
    if (!fmMatch) return defaults;

    const fmContent = fmMatch[1];

    // Parse status
    const statusMatch = fmContent.match(/^status:\s*(.+)$/m);
    if (statusMatch) {
      const status = statusMatch[1].trim() as SpecStatus;
      if (SPEC_STATUSES.includes(status)) {
        defaults.status = status;
      }
    }

    // Parse created
    const createdMatch = fmContent.match(/^created:\s*(\d{4}-\d{2}-\d{2})/m);
    if (createdMatch) {
      defaults.created = createdMatch[1];
    }

    // Parse completed
    const completedMatch = fmContent.match(/^completed:\s*(\d{4}-\d{2}-\d{2}|null)/m);
    if (completedMatch && completedMatch[1] !== "null") {
      defaults.completed = completedMatch[1];
    }

    // Parse dependencies (YAML array)
    const depsMatch = fmContent.match(/^dependencies:\s*\[(.*)\]/m);
    if (depsMatch && depsMatch[1].trim()) {
      defaults.dependencies = depsMatch[1]
        .split(",")
        .map((d) => d.trim().replace(/['"]/g, ""))
        .filter(Boolean);
    }

    return defaults;
  }
}

// ============================================================================
// Default Export
// ============================================================================

export function createSpecManager(): SpecManager {
  return new FolderSpecManager();
}

import { describe, expect, it } from "bun:test";

/**
 * Convert branch name to a valid container ID.
 * Matches the implementation in create-container.ts
 */
function branchToContainerId(branchName: string): string {
  // Order matters: truncate first, then remove leading/trailing dashes
  return branchName
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, "-")
    .replace(/-+/g, "-")
    .slice(0, 32)
    .replace(/^-|-$/g, "");
}

describe("create-container", () => {
  describe("branchToContainerId", () => {
    it("converts simple branch name to lowercase", () => {
      const id = branchToContainerId("Feature-Test");
      expect(id).toBe("feature-test");
    });

    it("replaces special characters with dashes", () => {
      const id = branchToContainerId("feature/add_new.thing");
      expect(id).toBe("feature-add-new-thing");
    });

    it("collapses multiple dashes into one", () => {
      const id = branchToContainerId("feature--double---dash");
      expect(id).toBe("feature-double-dash");
    });

    it("removes leading and trailing dashes", () => {
      const id = branchToContainerId("-feature-name-");
      expect(id).toBe("feature-name");
    });

    it("handles Linear issue format (HEY-123-description)", () => {
      const id = branchToContainerId("HEY-123-add-dark-mode");
      expect(id).toBe("hey-123-add-dark-mode");
    });

    it("truncates to 32 characters", () => {
      const longName = "this-is-a-very-long-branch-name-that-exceeds-the-limit";
      const id = branchToContainerId(longName);
      expect(id.length).toBeLessThanOrEqual(32);
      expect(id).toBe("this-is-a-very-long-branch-name");
    });

    it("handles empty string", () => {
      const id = branchToContainerId("");
      expect(id).toBe("");
    });

    it("handles string with only special characters", () => {
      const id = branchToContainerId("///___...");
      expect(id).toBe("");
    });

    it("handles numeric prefixes", () => {
      const id = branchToContainerId("123-feature");
      expect(id).toBe("123-feature");
    });
  });

  describe("container naming", () => {
    it("generates valid Docker container name format", () => {
      const branchName = "HEY-456-implement-feature";
      const devId = branchToContainerId(branchName);
      const containerName = `heycat-dev-${devId}`;

      // Docker container names must match [a-zA-Z0-9][a-zA-Z0-9_.-]*
      expect(containerName).toMatch(/^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/);
      expect(containerName).toBe("heycat-dev-hey-456-implement-feature");
    });

    it("produces consistent IDs for same branch", () => {
      const id1 = branchToContainerId("feature/my-branch");
      const id2 = branchToContainerId("feature/my-branch");
      expect(id1).toBe(id2);
    });
  });
});

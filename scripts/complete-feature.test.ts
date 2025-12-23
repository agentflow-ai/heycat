import { describe, expect, it } from "bun:test";
import { deriveCommitMessage, parseArgs } from "./complete-feature";

describe("complete-feature", () => {
  describe("parseArgs", () => {
    it("parses --continue flag", () => {
      const flags = parseArgs(["--continue"]);
      expect(flags.continue).toBe(true);
      expect(flags.dryRun).toBe(false);
      expect(flags.help).toBe(false);
    });

    it("parses --dry-run flag", () => {
      const flags = parseArgs(["--dry-run"]);
      expect(flags.dryRun).toBe(true);
      expect(flags.continue).toBe(false);
    });

    it("parses --help flag", () => {
      const flags = parseArgs(["--help"]);
      expect(flags.help).toBe(true);
    });

    it("parses -h flag", () => {
      const flags = parseArgs(["-h"]);
      expect(flags.help).toBe(true);
    });

    it("parses multiple flags", () => {
      const flags = parseArgs(["--continue", "--dry-run"]);
      expect(flags.continue).toBe(true);
      expect(flags.dryRun).toBe(true);
    });

    it("returns defaults for empty args", () => {
      const flags = parseArgs([]);
      expect(flags.continue).toBe(false);
      expect(flags.dryRun).toBe(false);
      expect(flags.help).toBe(false);
    });

    it("ignores unknown flags", () => {
      const flags = parseArgs(["--unknown", "random"]);
      expect(flags.continue).toBe(false);
      expect(flags.dryRun).toBe(false);
      expect(flags.help).toBe(false);
    });
  });

  describe("deriveCommitMessage", () => {
    it("returns default for empty commits", () => {
      const message = deriveCommitMessage([]);
      expect(message).toBe("chore: merge feature");
    });

    it("strips WIP: prefix", () => {
      const message = deriveCommitMessage(["WIP: add new feature"]);
      expect(message).toContain("add new feature");
      expect(message).not.toContain("WIP");
    });

    it("strips WIP: prefix case-insensitively", () => {
      const message = deriveCommitMessage(["wip: lowercase prefix"]);
      expect(message).toContain("lowercase prefix");
      expect(message.toLowerCase()).not.toContain("wip:");
    });

    it("detects feat type from commits", () => {
      const message = deriveCommitMessage(["Add new button", "Implement click handler"]);
      expect(message).toMatch(/^feat/);
    });

    it("detects fix type from bug-related commits", () => {
      const message = deriveCommitMessage(["Fix broken button", "Bug in handler fixed"]);
      expect(message).toMatch(/^fix/);
    });

    it("detects refactor type", () => {
      const message = deriveCommitMessage(["Refactor component structure", "Refactor utils"]);
      expect(message).toMatch(/^refactor/);
    });

    it("detects docs type", () => {
      const message = deriveCommitMessage(["Update documentation", "Document API"]);
      expect(message).toMatch(/^docs/);
    });

    it("detects test type", () => {
      const message = deriveCommitMessage(["Add tests for component", "Test coverage improved"]);
      expect(message).toMatch(/^test/);
    });

    it("handles multiple WIP commits", () => {
      const commits = [
        "WIP: Start implementation",
        "WIP: Add more logic",
        "WIP: Fix edge case",
        "Final touches",
      ];
      const message = deriveCommitMessage(commits);
      expect(message).not.toContain("WIP");
      expect(message.length).toBeGreaterThan(0);
    });

    it("extracts scope from bracket notation", () => {
      const message = deriveCommitMessage(["[hotkey] Add recording feature"]);
      expect(message).toContain("(hotkey)");
    });

    it("truncates long messages", () => {
      const longCommit = "Add a very long feature description that goes on and on describing all the details of what was implemented in this commit message that is way too long";
      const message = deriveCommitMessage([longCommit]);
      expect(message.length).toBeLessThanOrEqual(80);
    });

    it("handles commits with only WIP prefix", () => {
      const message = deriveCommitMessage(["WIP: ", "WIP:"]);
      expect(message).toBe("chore: merge feature");
    });

    it("creates meaningful message from multiple commits", () => {
      const commits = [
        "WIP: Add hotkey recorder component",
        "WIP: Implement key event handling",
        "Fix modifier key detection",
      ];
      const message = deriveCommitMessage(commits);
      expect(message.length).toBeGreaterThan(10);
      expect(message).toMatch(/^(feat|fix|refactor|chore|docs|test)/);
    });

    it("handles single conventional commit", () => {
      const message = deriveCommitMessage(["feat: add new feature"]);
      // Should preserve the essence but reformat
      expect(message).toContain("add new feature");
    });

    it("filters empty messages", () => {
      const message = deriveCommitMessage(["", "  ", "Valid commit"]);
      expect(message).not.toBe("chore: merge feature");
      expect(message.length).toBeGreaterThan(0);
    });
  });
});

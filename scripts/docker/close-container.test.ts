import { describe, expect, it } from "bun:test";
import { parseArgs } from "./close-container";

describe("close-container", () => {
  describe("parseArgs", () => {
    it("parses container ID argument", () => {
      const flags = parseArgs(["feature-test"]);
      expect(flags.containerId).toBe("feature-test");
      expect(flags.force).toBe(false);
      expect(flags.cleanVolumes).toBe(false);
    });

    it("parses --force flag", () => {
      const flags = parseArgs(["--force"]);
      expect(flags.force).toBe(true);
    });

    it("parses -f short flag", () => {
      const flags = parseArgs(["-f"]);
      expect(flags.force).toBe(true);
    });

    it("parses --clean-volumes flag", () => {
      const flags = parseArgs(["--clean-volumes"]);
      expect(flags.cleanVolumes).toBe(true);
    });

    it("parses --volumes alias", () => {
      const flags = parseArgs(["--volumes"]);
      expect(flags.cleanVolumes).toBe(true);
    });

    it("parses --help flag", () => {
      const flags = parseArgs(["--help"]);
      expect(flags.help).toBe(true);
    });

    it("parses -h short flag", () => {
      const flags = parseArgs(["-h"]);
      expect(flags.help).toBe(true);
    });

    it("parses multiple flags together", () => {
      const flags = parseArgs(["my-container", "--force", "--clean-volumes"]);
      expect(flags.containerId).toBe("my-container");
      expect(flags.force).toBe(true);
      expect(flags.cleanVolumes).toBe(true);
    });

    it("handles full container name with heycat-dev prefix", () => {
      const flags = parseArgs(["heycat-dev-feature-test"]);
      expect(flags.containerId).toBe("heycat-dev-feature-test");
    });

    it("returns null containerId when no argument provided", () => {
      const flags = parseArgs([]);
      expect(flags.containerId).toBeNull();
    });

    it("ignores unknown flags", () => {
      const flags = parseArgs(["--unknown", "container-id"]);
      expect(flags.containerId).toBe("container-id");
    });
  });

  describe("volume naming", () => {
    it("generates correct volume names for dev ID", () => {
      const devId = "feature-test";
      const expectedVolumes = [
        `heycat-bun-cache-${devId}`,
        `heycat-cargo-registry-${devId}`,
        `heycat-cargo-git-${devId}`,
      ];

      expect(expectedVolumes).toEqual([
        "heycat-bun-cache-feature-test",
        "heycat-cargo-registry-feature-test",
        "heycat-cargo-git-feature-test",
      ]);
    });
  });

  describe("container naming", () => {
    it("generates correct container name from dev ID", () => {
      const devId = "hey-123-add-feature";
      const containerName = `heycat-dev-${devId}`;
      expect(containerName).toBe("heycat-dev-hey-123-add-feature");
    });
  });
});

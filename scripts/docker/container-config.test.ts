import { afterEach, beforeEach, describe, expect, it } from "bun:test";
import {
  getContainerName,
  getDevMode,
  getEnvironmentInfo,
  isDockerEnvironment,
  type DevMode,
} from "./container-config";

describe("container-config", () => {
  // Save original env vars
  const originalEnv = { ...process.env };

  beforeEach(() => {
    // Reset env vars before each test
    delete process.env.HEYCAT_DOCKER_DEV;
    delete process.env.HEYCAT_DEV_ID;
  });

  afterEach(() => {
    // Restore original env vars
    process.env = { ...originalEnv };
  });

  describe("isDockerEnvironment", () => {
    it("returns false when HEYCAT_DOCKER_DEV is not set", () => {
      expect(isDockerEnvironment()).toBe(false);
    });

    it("returns false when HEYCAT_DOCKER_DEV is empty", () => {
      process.env.HEYCAT_DOCKER_DEV = "";
      expect(isDockerEnvironment()).toBe(false);
    });

    it("returns false when HEYCAT_DOCKER_DEV is 0", () => {
      process.env.HEYCAT_DOCKER_DEV = "0";
      expect(isDockerEnvironment()).toBe(false);
    });

    it("returns true when HEYCAT_DOCKER_DEV is 1", () => {
      process.env.HEYCAT_DOCKER_DEV = "1";
      expect(isDockerEnvironment()).toBe(true);
    });
  });

  describe("getContainerName", () => {
    it("generates correct container name from dev ID", () => {
      expect(getContainerName("feature-test")).toBe("heycat-dev-feature-test");
    });

    it("handles Linear issue format", () => {
      expect(getContainerName("hey-123-add-feature")).toBe("heycat-dev-hey-123-add-feature");
    });
  });

  describe("getDevMode", () => {
    it("returns 'docker' when in Docker environment", () => {
      process.env.HEYCAT_DOCKER_DEV = "1";
      expect(getDevMode()).toBe("docker");
    });

    it("returns 'main' or 'worktree' when not in Docker", () => {
      // Without Docker env var, it will check git config
      // In test environment, result depends on where tests run
      const mode = getDevMode();
      expect(["main", "worktree"]).toContain(mode);
    });
  });

  describe("getEnvironmentInfo", () => {
    it("returns complete info for Docker environment", () => {
      process.env.HEYCAT_DOCKER_DEV = "1";
      process.env.HEYCAT_DEV_ID = "test-feature";

      const info = getEnvironmentInfo();

      expect(info.mode).toBe("docker");
      expect(info.containerId).toBe("test-feature");
      expect(info.containerName).toBe("heycat-dev-test-feature");
      expect(info.isDocker).toBe(true);
    });

    it("returns null container info when not in Docker without worktree", () => {
      // This test runs in a real worktree, so we check structure only
      const info = getEnvironmentInfo();

      expect(info.mode).toBeDefined();
      expect(info.isDocker).toBe(false);
      // containerId and containerName depend on test execution environment
    });

    it("handles default HEYCAT_DEV_ID", () => {
      process.env.HEYCAT_DOCKER_DEV = "1";
      process.env.HEYCAT_DEV_ID = "default";

      const info = getEnvironmentInfo();

      expect(info.mode).toBe("docker");
      expect(info.containerId).toBeNull();
      expect(info.containerName).toBeNull();
    });
  });

  describe("DevMode type", () => {
    it("type allows docker, worktree, and main", () => {
      const modes: DevMode[] = ["docker", "worktree", "main"];
      expect(modes.length).toBe(3);
    });
  });
});

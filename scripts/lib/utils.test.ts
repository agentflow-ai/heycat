import { describe, expect, it, mock, afterEach } from "bun:test";
import { colors, log, success, error, info, warn } from "./utils";

describe("utils", () => {
  describe("colors", () => {
    it("has reset code", () => {
      expect(colors.reset).toBe("\x1b[0m");
    });

    it("has bold code", () => {
      expect(colors.bold).toBe("\x1b[1m");
    });

    it("has all color codes", () => {
      expect(colors.red).toBe("\x1b[31m");
      expect(colors.green).toBe("\x1b[32m");
      expect(colors.yellow).toBe("\x1b[33m");
      expect(colors.blue).toBe("\x1b[34m");
      expect(colors.cyan).toBe("\x1b[36m");
      expect(colors.dim).toBe("\x1b[2m");
    });
  });

  describe("logging functions", () => {
    // Store original console methods
    const originalLog = console.log;
    const originalError = console.error;

    afterEach(() => {
      // Restore original console methods
      console.log = originalLog;
      console.error = originalError;
    });

    it("log outputs message to console.log", () => {
      const mockLog = mock(() => {});
      console.log = mockLog;

      log("test message");

      expect(mockLog).toHaveBeenCalledWith("test message");
    });

    it("success outputs green bold message", () => {
      const mockLog = mock(() => {});
      console.log = mockLog;

      success("success message");

      expect(mockLog).toHaveBeenCalledWith(
        `${colors.green}${colors.bold}success message${colors.reset}`
      );
    });

    it("error outputs red bold message to console.error", () => {
      const mockError = mock(() => {});
      console.error = mockError;

      error("error message");

      expect(mockError).toHaveBeenCalledWith(
        `${colors.red}${colors.bold}Error: error message${colors.reset}`
      );
    });

    it("info outputs cyan message", () => {
      const mockLog = mock(() => {});
      console.log = mockLog;

      info("info message");

      expect(mockLog).toHaveBeenCalledWith(
        `${colors.cyan}info message${colors.reset}`
      );
    });

    it("warn outputs yellow message", () => {
      const mockLog = mock(() => {});
      console.log = mockLog;

      warn("warning message");

      expect(mockLog).toHaveBeenCalledWith(
        `${colors.yellow}warning message${colors.reset}`
      );
    });
  });
});

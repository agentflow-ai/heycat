import { describe, expect, it } from "bun:test";
import { parseArgs } from "./local-dev";

describe("local-dev", () => {
  describe("parseArgs", () => {
    it("parses --shell flag", () => {
      const flags = parseArgs(["--shell"]);
      expect(flags.shell).toBe(true);
      expect(flags.build).toBe(false);
      expect(flags.dev).toBe(false);
    });

    it("parses -s short flag", () => {
      const flags = parseArgs(["-s"]);
      expect(flags.shell).toBe(true);
    });

    it("parses --build flag", () => {
      const flags = parseArgs(["--build"]);
      expect(flags.build).toBe(true);
      expect(flags.shell).toBe(false);
    });

    it("parses -b short flag", () => {
      const flags = parseArgs(["-b"]);
      expect(flags.build).toBe(true);
    });

    it("parses --dev flag", () => {
      const flags = parseArgs(["--dev"]);
      expect(flags.dev).toBe(true);
      expect(flags.build).toBe(false);
    });

    it("parses -d short flag", () => {
      const flags = parseArgs(["-d"]);
      expect(flags.dev).toBe(true);
    });

    it("parses --stop flag", () => {
      const flags = parseArgs(["--stop"]);
      expect(flags.stop).toBe(true);
    });

    it("parses --help flag", () => {
      const flags = parseArgs(["--help"]);
      expect(flags.help).toBe(true);
    });

    it("parses -h short flag", () => {
      const flags = parseArgs(["-h"]);
      expect(flags.help).toBe(true);
    });

    it("returns default flags when no args", () => {
      const flags = parseArgs([]);
      expect(flags.shell).toBe(false);
      expect(flags.build).toBe(false);
      expect(flags.dev).toBe(false);
      expect(flags.stop).toBe(false);
      expect(flags.help).toBe(false);
    });

    it("parses multiple flags", () => {
      const flags = parseArgs(["--shell", "--help"]);
      expect(flags.shell).toBe(true);
      expect(flags.help).toBe(true);
    });
  });
});

import { describe, expect, it } from "bun:test";
import { parseDiscoveryPhase } from "./analysis";
import type { DiscoveryPhase } from "./types";

describe("parseDiscoveryPhase", () => {
  it("returns not_started when no discovery_phase in content", () => {
    const content = `# Feature: Test

**Created:** 2024-01-01
**Owner:** John

## Description
Some description
`;
    expect(parseDiscoveryPhase(content)).toBe("not_started");
  });

  it("parses discovery_phase from frontmatter", () => {
    const content = `---
discovery_phase: persona
---

# Feature: Test
`;
    expect(parseDiscoveryPhase(content)).toBe("persona");
  });

  it("parses discovery_phase from body **Discovery Phase:** format", () => {
    const content = `# Feature: Test

**Created:** 2024-01-01
**Owner:** John
**Discovery Phase:** paths

## Description
`;
    expect(parseDiscoveryPhase(content)).toBe("paths");
  });

  it("handles all valid phase values", () => {
    const phases: DiscoveryPhase[] = [
      "not_started",
      "persona",
      "paths",
      "scope",
      "synthesize",
      "complete",
    ];

    for (const phase of phases) {
      const content = `---
discovery_phase: ${phase}
---
`;
      expect(parseDiscoveryPhase(content)).toBe(phase);
    }
  });

  it("returns not_started for invalid phase value", () => {
    const content = `---
discovery_phase: invalid_phase
---
`;
    expect(parseDiscoveryPhase(content)).toBe("not_started");
  });

  it("is case insensitive for frontmatter key", () => {
    const content = `---
Discovery_Phase: scope
---
`;
    expect(parseDiscoveryPhase(content)).toBe("scope");
  });

  it("prefers frontmatter over body format", () => {
    const content = `---
discovery_phase: complete
---

# Feature: Test

**Discovery Phase:** persona
`;
    // Frontmatter should take precedence
    expect(parseDiscoveryPhase(content)).toBe("complete");
  });
});

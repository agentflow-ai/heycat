import { describe, expect, it } from "bun:test";
import {
  validateBDDScenarios,
  parseBDDSection,
  hasBDDScenarios,
} from "./bdd-validator";

describe("parseBDDSection", () => {
  it("returns null when no BDD section exists", () => {
    const content = "# Feature\n\n## Description\nSome text";
    const result = parseBDDSection(content);
    expect(result).toBeNull();
  });

  it("parses BDD section with all subsections", () => {
    const content = `# Feature

## BDD Scenarios

### User Persona
A developer who wants to automate tasks

### Problem Statement
Manual task management is tedious

\`\`\`gherkin
Feature: Task automation

  Scenario: Happy path - create task
    Given the user has an empty task list
    When the user creates a new task
    Then the task appears in the list
\`\`\`

### Out of Scope
- Advanced scheduling
- Multi-user support

### Assumptions
- User has internet connection
- User is authenticated
`;

    const result = parseBDDSection(content);
    expect(result).not.toBeNull();
    expect(result?.userPersona).toContain("developer");
    expect(result?.problemStatement).toContain("tedious");
    expect(result?.scenarios).toHaveLength(1);
    expect(result?.scenarios[0].name).toBe("Happy path - create task");
    expect(result?.scenarios[0].givenSteps).toHaveLength(1);
    expect(result?.scenarios[0].whenSteps).toHaveLength(1);
    expect(result?.scenarios[0].thenSteps).toHaveLength(1);
    expect(result?.outOfScope).toHaveLength(2);
    expect(result?.assumptions).toHaveLength(2);
  });

  it("parses inline Gherkin without code block", () => {
    const content = `## BDD Scenarios

Scenario: Test scenario
  Given a precondition
  When an action occurs
  Then the result is observed
`;

    const result = parseBDDSection(content);
    expect(result?.scenarios).toHaveLength(1);
    expect(result?.scenarios[0].givenSteps).toContain("a precondition");
  });

  it("parses Background section", () => {
    const content = `## BDD Scenarios

\`\`\`gherkin
Feature: With background

  Background:
    Given the system is initialized
    And the user is logged in

  Scenario: Test with background
    Given additional context
    When something happens
    Then result
\`\`\`
`;

    const result = parseBDDSection(content);
    expect(result?.background).not.toBeNull();
    expect(result?.background?.givenSteps).toHaveLength(2);
    expect(result?.scenarios).toHaveLength(1);
  });

  it("handles And/But continuation steps", () => {
    const content = `## BDD Scenarios

\`\`\`gherkin
Scenario: Multiple steps
  Given first condition
  And second condition
  When action one
  And action two
  Then outcome one
  And outcome two
  But not this outcome
\`\`\`
`;

    const result = parseBDDSection(content);
    expect(result?.scenarios[0].givenSteps).toHaveLength(2);
    expect(result?.scenarios[0].whenSteps).toHaveLength(2);
    expect(result?.scenarios[0].thenSteps).toHaveLength(3);
  });
});

describe("validateBDDScenarios", () => {
  it("fails when BDD section is missing", () => {
    const content = "# Feature\n\n## Description\nSome text";
    const result = validateBDDScenarios(content);

    expect(result.valid).toBe(false);
    expect(result.formatErrors).toHaveLength(1);
    expect(result.formatErrors[0].type).toBe("missing_section");
  });

  it("fails when no scenarios exist", () => {
    const content = `## BDD Scenarios

### User Persona
A user

### Problem Statement
A problem
`;

    const result = validateBDDScenarios(content);
    expect(result.valid).toBe(false);
    expect(result.formatErrors.some((e) => e.type === "missing_scenario")).toBe(true);
  });

  it("fails when scenario is incomplete", () => {
    const content = `## BDD Scenarios

### User Persona
A user

### Problem Statement
A problem

\`\`\`gherkin
Scenario: Incomplete scenario
  Given a precondition
  When an action
\`\`\`

### Out of Scope
- Nothing

### Assumptions
- Something
`;

    const result = validateBDDScenarios(content);
    expect(result.valid).toBe(false);
    expect(result.formatErrors.some((e) => e.type === "invalid_scenario")).toBe(true);
  });

  it("fails when placeholder text exists", () => {
    const content = `## BDD Scenarios

### User Persona
[Describe the user]

### Problem Statement
A real problem

\`\`\`gherkin
Scenario: Test
  Given context
  When action
  Then result
\`\`\`
`;

    const result = validateBDDScenarios(content);
    expect(result.valid).toBe(false);
    expect(result.formatErrors.some((e) => e.type === "placeholder_text")).toBe(true);
  });

  it("fails completeness when user persona missing", () => {
    const content = `## BDD Scenarios

### Problem Statement
A problem

\`\`\`gherkin
Scenario: Test one
  Given context
  When action
  Then result

Scenario: Test two
  Given context
  When action
  Then result
\`\`\`

### Out of Scope
- Nothing

### Assumptions
- Something
`;

    const result = validateBDDScenarios(content);
    expect(result.valid).toBe(false);
    expect(result.completenessErrors.some((e) => e.type === "missing_persona")).toBe(true);
    expect(result.hasUserPersona).toBe(false);
  });

  it("fails completeness when fewer than 2 scenarios", () => {
    const content = `## BDD Scenarios

### User Persona
A user

### Problem Statement
A problem

\`\`\`gherkin
Scenario: Only one
  Given context
  When action
  Then result
\`\`\`

### Out of Scope
- Nothing

### Assumptions
- Something
`;

    const result = validateBDDScenarios(content);
    expect(result.valid).toBe(false);
    expect(result.completenessErrors.some((e) => e.type === "few_scenarios")).toBe(true);
    expect(result.scenarioCount).toBe(1);
  });

  it("passes when all requirements met", () => {
    const content = `## BDD Scenarios

### User Persona
A developer working on automation

### Problem Statement
Manual tasks are time consuming

\`\`\`gherkin
Feature: Task automation

  Scenario: Happy path - create task
    Given an empty task list
    When creating a new task
    Then the task is added

  Scenario: Error - duplicate task
    Given a task already exists
    When creating a duplicate
    Then an error is shown
\`\`\`

### Out of Scope
- Multi-tenant support

### Assumptions
- User has permissions
`;

    const result = validateBDDScenarios(content);
    expect(result.valid).toBe(true);
    expect(result.formatErrors).toHaveLength(0);
    expect(result.completenessErrors).toHaveLength(0);
    expect(result.scenarioCount).toBe(2);
    expect(result.hasUserPersona).toBe(true);
    expect(result.hasProblemStatement).toBe(true);
    expect(result.hasOutOfScope).toBe(true);
    expect(result.hasAssumptions).toBe(true);
  });
});

describe("hasBDDScenarios", () => {
  it("returns false when no BDD section", () => {
    expect(hasBDDScenarios("# Feature")).toBe(false);
  });

  it("returns false when scenarios incomplete", () => {
    const content = `## BDD Scenarios

Scenario: Incomplete
  Given only a given
`;
    expect(hasBDDScenarios(content)).toBe(false);
  });

  it("returns true when at least one complete scenario", () => {
    const content = `## BDD Scenarios

Scenario: Complete
  Given context
  When action
  Then result
`;
    expect(hasBDDScenarios(content)).toBe(true);
  });
});

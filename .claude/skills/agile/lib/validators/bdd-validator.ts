import type {
  BDDValidationResult,
  BDDFormatError,
  BDDCompletenessError,
  ParsedBDDSection,
  ParsedScenario,
  ParsedBackground,
} from "../types";

// Placeholder patterns to detect incomplete content
const PLACEHOLDER_PATTERNS = [
  /\[.*?\]/g, // [placeholder text]
  /<.*?>/g, // <placeholder text>
  /TBD/gi,
  /TODO/gi,
  /FIXME/gi,
];

/**
 * Extract the ## BDD Scenarios section from markdown content
 */
function extractBDDSection(content: string): string | null {
  // Match ## BDD Scenarios until the next ## heading or end of file
  const match = content.match(/## BDD Scenarios[\s\S]*?(?=\n##(?!#)|$)/i);
  return match ? match[0] : null;
}

/**
 * Extract a subsection (### Header) content
 */
function extractSubsection(content: string, header: string): string | null {
  const pattern = new RegExp(
    `### ${header}[\\s\\S]*?(?=\\n###|\\n##(?!#)|$)`,
    "i"
  );
  const match = content.match(pattern);
  if (!match) return null;

  // Remove the header line and trim
  const text = match[0].replace(new RegExp(`### ${header}`, "i"), "").trim();
  return text.length > 0 ? text : null;
}

/**
 * Extract list items from a subsection
 */
function extractListItems(content: string, header: string): string[] {
  const section = extractSubsection(content, header);
  if (!section) return [];

  const items: string[] = [];
  const lines = section.split("\n");

  for (const line of lines) {
    // Match bullet points: -, *, or numbered lists
    const match = line.match(/^\s*[-*]\s+(.+)$|^\s*\d+\.\s+(.+)$/);
    if (match) {
      items.push((match[1] || match[2]).trim());
    }
  }

  return items;
}

/**
 * Extract gherkin code block from content
 */
function extractGherkinBlock(content: string): string | null {
  // Match ```gherkin ... ``` or ``` ... ``` containing Feature:
  const gherkinMatch = content.match(/```gherkin\s*([\s\S]*?)```/i);
  if (gherkinMatch) return gherkinMatch[1].trim();

  // Fallback: look for inline Gherkin (Feature: or Scenario: without code block)
  const hasFeature = /^\s*Feature:/m.test(content);
  const hasScenario = /^\s*Scenario:/m.test(content);

  if (hasFeature || hasScenario) {
    // Extract from Feature: or first Scenario: to end of section
    const start = content.search(/^\s*(Feature:|Scenario:)/m);
    if (start >= 0) {
      return content.slice(start).trim();
    }
  }

  return null;
}

/**
 * Parse Gherkin scenarios from a gherkin block
 */
function parseGherkinScenarios(gherkin: string): {
  scenarios: ParsedScenario[];
  background: ParsedBackground | null;
} {
  const scenarios: ParsedScenario[] = [];
  let background: ParsedBackground | null = null;

  const lines = gherkin.split("\n");
  let currentScenario: ParsedScenario | null = null;
  let inBackground = false;
  let backgroundSteps: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Background block
    if (/^Background:/i.test(trimmed)) {
      inBackground = true;
      backgroundSteps = [];
      continue;
    }

    // Scenario block (ends background if active)
    if (/^Scenario:/i.test(trimmed)) {
      // Save background if we were in one
      if (inBackground && backgroundSteps.length > 0) {
        background = {
          givenSteps: backgroundSteps,
          lineNumber: i - backgroundSteps.length,
        };
      }
      inBackground = false;

      // Save previous scenario
      if (currentScenario) {
        scenarios.push(currentScenario);
      }

      // Start new scenario
      const name = trimmed.replace(/^Scenario:\s*/i, "").trim();
      currentScenario = {
        name,
        givenSteps: [],
        whenSteps: [],
        thenSteps: [],
        lineNumber: i + 1,
      };
      continue;
    }

    // Step lines
    if (/^\s*(Given|When|Then|And|But)\s+/i.test(trimmed)) {
      const stepMatch = trimmed.match(/^\s*(Given|When|Then|And|But)\s+(.+)$/i);
      if (stepMatch) {
        const keyword = stepMatch[1].toLowerCase();
        const stepText = stepMatch[2];

        if (inBackground) {
          if (keyword === "given" || keyword === "and" || keyword === "but") {
            backgroundSteps.push(stepText);
          }
        } else if (currentScenario) {
          if (keyword === "given") {
            currentScenario.givenSteps.push(stepText);
          } else if (keyword === "when") {
            currentScenario.whenSteps.push(stepText);
          } else if (keyword === "then") {
            currentScenario.thenSteps.push(stepText);
          } else if (keyword === "and" || keyword === "but") {
            // And/But continue the previous step type
            // Determine which list to add to based on last non-empty
            if (currentScenario.thenSteps.length > 0) {
              currentScenario.thenSteps.push(stepText);
            } else if (currentScenario.whenSteps.length > 0) {
              currentScenario.whenSteps.push(stepText);
            } else {
              currentScenario.givenSteps.push(stepText);
            }
          }
        }
      }
    }
  }

  // Don't forget the last scenario
  if (currentScenario) {
    scenarios.push(currentScenario);
  }

  // Handle case where only background exists at end
  if (inBackground && backgroundSteps.length > 0 && !background) {
    background = {
      givenSteps: backgroundSteps,
      lineNumber: lines.length - backgroundSteps.length,
    };
  }

  return { scenarios, background };
}

/**
 * Check if content has placeholder text
 */
function hasPlaceholderText(content: string): boolean {
  for (const pattern of PLACEHOLDER_PATTERNS) {
    if (pattern.test(content)) {
      return true;
    }
    // Reset lastIndex for global regexes
    pattern.lastIndex = 0;
  }
  return false;
}

/**
 * Parse the complete BDD section from markdown content
 */
export function parseBDDSection(content: string): ParsedBDDSection | null {
  const sectionContent = extractBDDSection(content);
  if (!sectionContent) return null;

  const userPersona = extractSubsection(sectionContent, "User Persona");
  const problemStatement = extractSubsection(sectionContent, "Problem Statement");
  const outOfScope = extractListItems(sectionContent, "Out of Scope");
  const assumptions = extractListItems(sectionContent, "Assumptions");
  const gherkinBlock = extractGherkinBlock(sectionContent);

  const { scenarios, background } = gherkinBlock
    ? parseGherkinScenarios(gherkinBlock)
    : { scenarios: [], background: null };

  return {
    raw: sectionContent,
    userPersona,
    problemStatement,
    gherkinBlock,
    scenarios,
    background,
    outOfScope,
    assumptions,
  };
}

/**
 * Validate Gherkin format (format errors are blocking)
 */
function validateFormat(section: ParsedBDDSection | null): BDDFormatError[] {
  const errors: BDDFormatError[] = [];

  if (!section) {
    errors.push({
      type: "missing_section",
      message: "Missing ## BDD Scenarios section",
    });
    return errors;
  }

  // Check for placeholder text
  if (hasPlaceholderText(section.raw)) {
    errors.push({
      type: "placeholder_text",
      message: "BDD section contains placeholder text ([...], <...>, TBD, TODO)",
    });
  }

  // Must have at least one scenario
  if (section.scenarios.length === 0) {
    errors.push({
      type: "missing_scenario",
      message: "At least one Scenario with Given/When/Then is required",
    });
    return errors;
  }

  // Validate each scenario has Given, When, Then
  for (const scenario of section.scenarios) {
    if (scenario.givenSteps.length === 0) {
      errors.push({
        type: "invalid_scenario",
        message: `Scenario "${scenario.name}" is missing Given steps`,
        scenarioName: scenario.name,
        line: scenario.lineNumber,
      });
    }
    if (scenario.whenSteps.length === 0) {
      errors.push({
        type: "invalid_scenario",
        message: `Scenario "${scenario.name}" is missing When steps`,
        scenarioName: scenario.name,
        line: scenario.lineNumber,
      });
    }
    if (scenario.thenSteps.length === 0) {
      errors.push({
        type: "invalid_scenario",
        message: `Scenario "${scenario.name}" is missing Then steps`,
        scenarioName: scenario.name,
        line: scenario.lineNumber,
      });
    }
  }

  return errors;
}

/**
 * Check completeness (completeness errors are blocking by default)
 */
function validateCompleteness(section: ParsedBDDSection | null): BDDCompletenessError[] {
  const errors: BDDCompletenessError[] = [];

  if (!section) return errors;

  if (!section.userPersona) {
    errors.push({
      type: "missing_persona",
      message: "Missing ### User Persona subsection",
      suggestion: "Add a ### User Persona subsection describing who the feature is for",
    });
  }

  if (!section.problemStatement) {
    errors.push({
      type: "missing_problem",
      message: "Missing ### Problem Statement subsection",
      suggestion: "Add a ### Problem Statement subsection explaining what problem is being solved",
    });
  }

  if (section.outOfScope.length === 0) {
    errors.push({
      type: "missing_scope",
      message: "Missing ### Out of Scope subsection",
      suggestion: "Add a ### Out of Scope subsection listing what is NOT included",
    });
  }

  if (section.assumptions.length === 0) {
    errors.push({
      type: "missing_assumptions",
      message: "Missing ### Assumptions subsection",
      suggestion: "Add an ### Assumptions subsection listing key assumptions",
    });
  }

  if (section.scenarios.length < 2) {
    errors.push({
      type: "few_scenarios",
      message: `Only ${section.scenarios.length} scenario(s) defined (minimum 2 required)`,
      suggestion: "Add scenarios for edge cases, error conditions, and alternative flows",
    });
  }

  return errors;
}

/**
 * Main validation function - validates BDD scenarios in markdown content
 */
export function validateBDDScenarios(content: string): BDDValidationResult {
  const section = parseBDDSection(content);
  const formatErrors = validateFormat(section);
  const completenessErrors = validateCompleteness(section);

  return {
    valid: formatErrors.length === 0 && completenessErrors.length === 0,
    formatErrors,
    completenessErrors,
    scenarioCount: section?.scenarios.length ?? 0,
    hasUserPersona: !!section?.userPersona,
    hasProblemStatement: !!section?.problemStatement,
    hasOutOfScope: (section?.outOfScope.length ?? 0) > 0,
    hasAssumptions: (section?.assumptions.length ?? 0) > 0,
  };
}

/**
 * Quick check for BDD scenarios (backward compatibility with hasBDDScenarios)
 * Returns true if there's at least one valid scenario with Given/When/Then
 */
export function hasBDDScenarios(content: string): boolean {
  const section = parseBDDSection(content);
  if (!section) return false;

  // Must have at least one complete scenario
  return section.scenarios.some(
    (s) =>
      s.givenSteps.length > 0 && s.whenSteps.length > 0 && s.thenSteps.length > 0
  );
}

/**
 * Format validation errors for display
 */
export function formatValidationErrors(
  issueName: string,
  result: BDDValidationResult,
  includeCompleteness: boolean = true
): string {
  const lines: string[] = [];

  if (result.formatErrors.length > 0) {
    lines.push(`BDD Format Errors for "${issueName}":`);
    for (const error of result.formatErrors) {
      const location = error.line ? ` (line ${error.line})` : "";
      lines.push(`  [ERROR] ${error.message}${location}`);
    }
    lines.push("");
  }

  if (includeCompleteness && result.completenessErrors.length > 0) {
    lines.push(`BDD Completeness Errors for "${issueName}":`);
    for (const error of result.completenessErrors) {
      lines.push(`  [ERROR] ${error.message}`);
      lines.push(`          → ${error.suggestion}`);
    }
    lines.push("");
  }

  if (lines.length > 0) {
    lines.push("Run 'agile.ts discover <name>' for guided BDD scenario creation.");
  }

  return lines.join("\n");
}

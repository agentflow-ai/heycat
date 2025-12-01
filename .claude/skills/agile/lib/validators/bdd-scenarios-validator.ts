import {
  type Stage,
  type Issue,
  type IssueAnalysis,
  type ValidationResult,
} from "../types";
import type { TransitionValidator } from "./validator-chain";
import { formatValidationErrors } from "./bdd-validator";

/**
 * Validates that features have completed BDD discovery before moving to todo.
 * This ensures product research (Given/When/Then scenarios) is done before
 * breaking down into specs.
 *
 * Applies to: 2-todo (backlog -> todo transition)
 * Only enforced for: features (bugs and tasks skip this validation)
 *
 * Validation requirements:
 * 1. Discovery phase must be 'complete'
 * 2. BDD scenarios must pass format validation (Gherkin syntax)
 * 3. BDD scenarios must pass completeness validation (persona, scope, etc.)
 */
export class BDDScenariosValidator implements TransitionValidator {
  readonly name = "BDDScenariosValidator";
  readonly appliesTo: Stage[] = ["2-todo"];

  validate(issue: Issue, analysis: IssueAnalysis, _toStage: Stage): ValidationResult {
    // Only applies to features - bugs and tasks don't need BDD scenarios
    if (issue.type !== "feature") {
      return { valid: true, missing: [] };
    }

    // Check discovery phase is complete
    if (analysis.discoveryPhase !== "complete") {
      return {
        valid: false,
        missing: [
          `Discovery incomplete (phase: ${analysis.discoveryPhase}). ` +
          `Run 'agile.ts discover ${issue.name}' to continue guided discovery.`,
        ],
      };
    }

    // Check BDD validation
    const bdd = analysis.bddValidation;
    if (!bdd) {
      return {
        valid: false,
        missing: [
          "BDD Scenarios section is missing. " +
          `Run 'agile.ts discover ${issue.name}' to create scenarios.`,
        ],
      };
    }

    if (!bdd.valid) {
      const errors: string[] = [];

      // Collect format errors (blocking)
      if (bdd.formatErrors.length > 0) {
        errors.push("BDD Format Errors:");
        for (const err of bdd.formatErrors) {
          errors.push(`  - ${err.message}`);
        }
      }

      // Collect completeness errors (blocking by default)
      if (bdd.completenessErrors.length > 0) {
        errors.push("BDD Completeness Errors:");
        for (const err of bdd.completenessErrors) {
          errors.push(`  - ${err.message}`);
        }
      }

      errors.push("");
      errors.push(`Run 'agile.ts discover ${issue.name}' for guided scenario creation.`);
      errors.push("Use 'agile.ts move ${issue.name} 2-todo --force' to bypass validation.");

      return {
        valid: false,
        missing: errors,
      };
    }

    return { valid: true, missing: [] };
  }
}

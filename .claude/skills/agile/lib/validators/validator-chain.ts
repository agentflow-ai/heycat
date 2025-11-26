import {
  type Stage,
  type Issue,
  type IssueAnalysis,
  type ValidationResult,
} from "../types";

// ============================================================================
// Validator Interface
// ============================================================================

export interface TransitionValidator {
  readonly name: string;
  readonly appliesTo: Stage[];
  validate(issue: Issue, analysis: IssueAnalysis, toStage: Stage): ValidationResult;
}

// ============================================================================
// Validator Chain
// ============================================================================

export class ValidatorChain {
  private validators: TransitionValidator[] = [];

  /**
   * Register a validator in the chain
   */
  register(validator: TransitionValidator): void {
    this.validators.push(validator);
  }

  /**
   * Run all applicable validators for a transition
   */
  validate(issue: Issue, analysis: IssueAnalysis, toStage: Stage): ValidationResult {
    const allMissing: string[] = [];

    for (const validator of this.validators) {
      // Only run validators that apply to the target stage
      if (!validator.appliesTo.includes(toStage)) {
        continue;
      }

      const result = validator.validate(issue, analysis, toStage);
      if (!result.valid) {
        allMissing.push(...result.missing);
      }
    }

    return {
      valid: allMissing.length === 0,
      missing: allMissing,
    };
  }

  /**
   * Get all registered validators
   */
  getValidators(): TransitionValidator[] {
    return [...this.validators];
  }
}

// ============================================================================
// Individual Validators
// ============================================================================

/**
 * Validates that description section is complete (no placeholders)
 * Applies to: 2-todo
 */
export class DescriptionValidator implements TransitionValidator {
  readonly name = "DescriptionValidator";
  readonly appliesTo: Stage[] = ["2-todo"];

  validate(_issue: Issue, analysis: IssueAnalysis, _toStage: Stage): ValidationResult {
    if (!analysis.hasDescription) {
      return {
        valid: false,
        missing: ["Description section must be complete (no placeholder text)"],
      };
    }

    // Check for incomplete sections that might indicate placeholder text
    const descIncomplete = analysis.incompleteSections.some(
      (s) => s.toLowerCase().includes("description")
    );

    if (descIncomplete) {
      return {
        valid: false,
        missing: ["Description section contains placeholder text"],
      };
    }

    return { valid: true, missing: [] };
  }
}

/**
 * Validates that owner is assigned
 * Applies to: 3-in-progress
 */
export class OwnerValidator implements TransitionValidator {
  readonly name = "OwnerValidator";
  readonly appliesTo: Stage[] = ["3-in-progress"];

  validate(_issue: Issue, analysis: IssueAnalysis, _toStage: Stage): ValidationResult {
    if (!analysis.ownerAssigned) {
      return {
        valid: false,
        missing: ["Owner must be assigned (not [Name])"],
      };
    }

    return { valid: true, missing: [] };
  }
}

/**
 * Validates that technical guidance exists
 * Applies to: 3-in-progress
 */
export class TechnicalGuidanceExistsValidator implements TransitionValidator {
  readonly name = "TechnicalGuidanceExistsValidator";
  readonly appliesTo: Stage[] = ["3-in-progress"];

  validate(_issue: Issue, analysis: IssueAnalysis, _toStage: Stage): ValidationResult {
    if (!analysis.technicalGuidance) {
      return {
        valid: false,
        missing: ["Technical guidance file must exist"],
      };
    }

    return { valid: true, missing: [] };
  }
}

/**
 * Validates that all specs are completed
 * Applies to: 4-review, 5-done
 */
export class AllSpecsCompletedValidator implements TransitionValidator {
  readonly name = "AllSpecsCompletedValidator";
  readonly appliesTo: Stage[] = ["4-review", "5-done"];

  validate(_issue: Issue, analysis: IssueAnalysis, _toStage: Stage): ValidationResult {
    if (analysis.specsTotal === 0) {
      return {
        valid: false,
        missing: ["At least one spec must be created"],
      };
    }

    if (!analysis.allSpecsCompleted) {
      const pending = analysis.specsTotal - analysis.specsCompleted;
      return {
        valid: false,
        missing: [
          `All specs must be completed (${analysis.specsCompleted}/${analysis.specsTotal} done, ${pending} remaining)`,
        ],
      };
    }

    return { valid: true, missing: [] };
  }
}

/**
 * Validates that technical guidance was updated after last spec completion
 * Applies to: 4-review
 */
export class TechnicalGuidanceUpdatedValidator implements TransitionValidator {
  readonly name = "TechnicalGuidanceUpdatedValidator";
  readonly appliesTo: Stage[] = ["4-review"];

  validate(_issue: Issue, analysis: IssueAnalysis, _toStage: Stage): ValidationResult {
    if (analysis.needsGuidanceUpdate) {
      return {
        valid: false,
        missing: [
          "Technical guidance must be updated after completing specs. " +
            "Run `agile.ts guidance update <issue>` to mark as updated.",
        ],
      };
    }

    return { valid: true, missing: [] };
  }
}

/**
 * Validates Definition of Done progress
 * Applies to: 4-review (>=1 checked), 5-done (all checked)
 */
export class DoDValidator implements TransitionValidator {
  readonly name = "DoDValidator";
  readonly appliesTo: Stage[] = ["4-review", "5-done"];

  validate(_issue: Issue, analysis: IssueAnalysis, toStage: Stage): ValidationResult {
    const { dod } = analysis;

    if (toStage === "4-review") {
      if (dod.completed === 0) {
        return {
          valid: false,
          missing: ["At least one Definition of Done item must be checked"],
        };
      }
    }

    if (toStage === "5-done") {
      if (dod.completed < dod.total) {
        const remaining = dod.total - dod.completed;
        return {
          valid: false,
          missing: [
            `All Definition of Done items must be checked (${dod.completed}/${dod.total} done, ${remaining} remaining)`,
          ],
        };
      }
    }

    return { valid: true, missing: [] };
  }
}

// ============================================================================
// Factory Function
// ============================================================================

/**
 * Create a validator chain with all standard validators
 */
export function createValidatorChain(): ValidatorChain {
  const chain = new ValidatorChain();

  chain.register(new DescriptionValidator());
  chain.register(new OwnerValidator());
  chain.register(new TechnicalGuidanceExistsValidator());
  chain.register(new AllSpecsCompletedValidator());
  chain.register(new TechnicalGuidanceUpdatedValidator());
  chain.register(new DoDValidator());

  return chain;
}

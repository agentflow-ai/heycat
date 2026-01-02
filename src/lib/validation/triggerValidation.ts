/**
 * Trigger validation utilities for dictionary entries.
 */

/**
 * Validates that a trigger phrase is not empty.
 * @param trigger - The trigger phrase to validate
 * @returns Error message if invalid, null if valid
 */
export function validateTrigger(trigger: string): string | null {
  if (!trigger.trim()) {
    return "Trigger is required";
  }
  return null;
}

/**
 * Checks if a trigger phrase is empty or whitespace-only.
 * @param trigger - The trigger phrase to check
 * @returns True if the trigger is empty or whitespace-only
 */
export function isEmptyTrigger(trigger: string): boolean {
  return !trigger.trim();
}

/**
 * Checks if a trigger already exists in the list of existing triggers.
 * @param trigger - The trigger to check
 * @param existingTriggers - Array of existing triggers (lowercase)
 * @returns True if the trigger is a duplicate
 */
export function isDuplicateTrigger(
  trigger: string,
  existingTriggers: string[]
): boolean {
  const normalizedTrigger = trigger.toLowerCase();
  return existingTriggers.includes(normalizedTrigger);
}

/**
 * Get duplicate trigger error message if trigger already exists.
 * @param trigger - The trigger to check
 * @param existingTriggers - Array of existing triggers (lowercase)
 * @returns Error message if duplicate, null otherwise
 */
export function getDuplicateTriggerError(
  trigger: string,
  existingTriggers: string[]
): string | null {
  if (isDuplicateTrigger(trigger, existingTriggers)) {
    return "This trigger already exists";
  }
  return null;
}

/**
 * Regular expression validation utilities.
 */

/**
 * Validates that a string is a valid regular expression.
 * Empty strings are considered valid (optional patterns).
 *
 * @param pattern - The pattern to validate
 * @returns Error message if invalid, null if valid
 */
export function validateRegexPattern(pattern: string): string | null {
  if (!pattern.trim()) {
    return null; // Empty patterns are valid
  }

  try {
    new RegExp(pattern);
    return null;
  } catch (e) {
    return `Invalid regex: ${e instanceof Error ? e.message : String(e)}`;
  }
}

/**
 * Checks if a string is a valid regular expression.
 * Empty strings are considered valid (optional patterns).
 *
 * @param pattern - The pattern to check
 * @returns True if the pattern is a valid regex
 */
export function isValidRegex(pattern: string): boolean {
  return validateRegexPattern(pattern) === null;
}

/**
 * Tests for validation utilities.
 *
 * These tests verify user-visible validation behaviors:
 * - Trigger validation for dictionary entries
 * - Suffix length validation
 * - Duplicate detection
 */

import { describe, it, expect } from "vitest";
import {
  validateTrigger,
  isEmptyTrigger,
  isDuplicateTrigger,
  getDuplicateTriggerError,
  validateSuffix,
  isSuffixTooLong,
  MAX_SUFFIX_LENGTH,
  validateRegexPattern,
  isValidRegex,
} from "../validation";

describe("trigger validation", () => {
  it("returns error for empty trigger", () => {
    expect(validateTrigger("")).toBe("Trigger is required");
    expect(validateTrigger("   ")).toBe("Trigger is required");
  });

  it("returns null for valid trigger", () => {
    expect(validateTrigger("brb")).toBeNull();
    expect(validateTrigger("  brb  ")).toBeNull(); // with whitespace
  });

  it("isEmptyTrigger returns true for empty/whitespace", () => {
    expect(isEmptyTrigger("")).toBe(true);
    expect(isEmptyTrigger("   ")).toBe(true);
    expect(isEmptyTrigger("brb")).toBe(false);
  });
});

describe("duplicate trigger detection", () => {
  const existingTriggers = ["brb", "omw", "ttyl"];

  it("detects duplicate triggers (case-insensitive)", () => {
    expect(isDuplicateTrigger("brb", existingTriggers)).toBe(true);
    expect(isDuplicateTrigger("BRB", existingTriggers)).toBe(true);
    expect(isDuplicateTrigger("Brb", existingTriggers)).toBe(true);
  });

  it("returns false for unique triggers", () => {
    expect(isDuplicateTrigger("lol", existingTriggers)).toBe(false);
  });

  it("getDuplicateTriggerError returns appropriate message", () => {
    expect(getDuplicateTriggerError("brb", existingTriggers)).toBe(
      "This trigger already exists"
    );
    expect(getDuplicateTriggerError("lol", existingTriggers)).toBeNull();
  });
});

describe("suffix validation", () => {
  it("accepts suffixes within length limit", () => {
    expect(validateSuffix("")).toBeNull();
    expect(validateSuffix(".")).toBeNull();
    expect(validateSuffix("...")).toBeNull();
    expect(validateSuffix("12345")).toBeNull(); // exactly 5 chars
  });

  it("rejects suffixes exceeding length limit", () => {
    expect(validateSuffix("123456")).toBe(
      `Suffix must be ${MAX_SUFFIX_LENGTH} characters or less`
    );
  });

  it("isSuffixTooLong returns correct boolean", () => {
    expect(isSuffixTooLong("12345")).toBe(false);
    expect(isSuffixTooLong("123456")).toBe(true);
  });

  it("MAX_SUFFIX_LENGTH is 5", () => {
    expect(MAX_SUFFIX_LENGTH).toBe(5);
  });
});

describe("regex pattern validation", () => {
  it("accepts empty patterns", () => {
    expect(validateRegexPattern("")).toBeNull();
    expect(validateRegexPattern("   ")).toBeNull();
    expect(isValidRegex("")).toBe(true);
  });

  it("accepts valid regex patterns", () => {
    expect(validateRegexPattern(".*")).toBeNull();
    expect(validateRegexPattern("^test$")).toBeNull();
    expect(validateRegexPattern("\\d+")).toBeNull();
    expect(isValidRegex("^[a-z]+$")).toBe(true);
  });

  it("rejects invalid regex patterns", () => {
    expect(validateRegexPattern("[")).toContain("Invalid regex");
    expect(validateRegexPattern("(unclosed")).toContain("Invalid regex");
    expect(isValidRegex("[")).toBe(false);
  });
});

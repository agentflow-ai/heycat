/**
 * Validation utilities for form inputs and data.
 */

export {
  validateTrigger,
  isEmptyTrigger,
  isDuplicateTrigger,
  getDuplicateTriggerError,
} from "./triggerValidation";

export {
  MAX_SUFFIX_LENGTH,
  validateSuffix,
  isSuffixTooLong,
} from "./suffixValidation";

export {
  validateRegexPattern,
  isValidRegex,
} from "./regexValidation";

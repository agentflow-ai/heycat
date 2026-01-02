import { useCallback, useState, useMemo } from "react";
import { useFormState } from "./useFormState";
import { validateTrigger, validateSuffix, isDuplicateTrigger } from "../lib/validation";

/**
 * Form values for dictionary entry creation/editing.
 */
export interface DictionaryFormValues {
  trigger: string;
  expansion: string;
  suffix: string;
  autoEnter: boolean;
  disableSuffix: boolean;
  completeMatchOnly: boolean;
  contextIds: string[];
  // Index signature for useFormState compatibility
  [key: string]: string | boolean | string[] | undefined;
}

/**
 * Configuration options for useDictionaryForm hook.
 */
export interface UseDictionaryFormOptions {
  /** Existing triggers for duplicate detection (lowercase) */
  existingTriggers: string[];
  /** ID to exclude from duplicate check (for editing) */
  excludeId?: string;
  /** Initial values for editing mode */
  initialValues?: Partial<DictionaryFormValues>;
  /** Callback when form is submitted successfully */
  onSubmit?: (values: DictionaryFormValues) => Promise<void>;
}

/**
 * Return type of the useDictionaryForm hook.
 */
export interface UseDictionaryFormReturn {
  /** Current form values */
  values: DictionaryFormValues;
  /** Trigger validation error */
  triggerError: string | null;
  /** Suffix validation error */
  suffixError: string | null;
  /** Whether the form is submitting */
  isSubmitting: boolean;
  /** Whether settings panel is open */
  isSettingsOpen: boolean;
  /** Set a form value */
  setValue: <K extends keyof DictionaryFormValues>(
    field: K,
    value: DictionaryFormValues[K]
  ) => void;
  /** Toggle settings panel */
  toggleSettings: () => void;
  /** Handle form submission */
  handleSubmit: (e?: React.FormEvent) => Promise<boolean>;
  /** Reset form to initial state */
  reset: () => void;
  /** Check if form has non-default settings */
  hasSettings: boolean;
  /** Whether form has unsaved changes */
  isDirty: boolean;
}

const DEFAULT_VALUES: DictionaryFormValues = {
  trigger: "",
  expansion: "",
  suffix: "",
  autoEnter: false,
  disableSuffix: false,
  completeMatchOnly: false,
  contextIds: [],
};

/**
 * Hook for managing dictionary entry form state with validation.
 *
 * Composes with useFormState for base form state management and adds
 * dictionary-specific validation and settings panel functionality.
 *
 * @example
 * const form = useDictionaryForm({
 *   existingTriggers: entries.map(e => e.trigger.toLowerCase()),
 *   onSubmit: async (values) => {
 *     await addEntry.mutateAsync(values);
 *   },
 * });
 */
export function useDictionaryForm(
  options: UseDictionaryFormOptions
): UseDictionaryFormReturn {
  const { existingTriggers, excludeId, initialValues, onSubmit } = options;

  const mergedInitialValues = useMemo(
    () => ({ ...DEFAULT_VALUES, ...initialValues }),
    [initialValues]
  );

  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  // Create validation function that checks trigger and suffix
  const validate = useCallback(
    (values: DictionaryFormValues): Partial<Record<keyof DictionaryFormValues, string>> => {
      const errors: Partial<Record<keyof DictionaryFormValues, string>> = {};

      // Validate trigger
      const triggerValidation = validateTrigger(values.trigger);
      if (triggerValidation) {
        errors.trigger = triggerValidation;
      } else {
        // Check for duplicates (excluding current entry if editing)
        const triggersToCheck = excludeId
          ? existingTriggers.filter((t) => t !== values.trigger.toLowerCase())
          : existingTriggers;

        if (isDuplicateTrigger(values.trigger, triggersToCheck)) {
          errors.trigger = "This trigger already exists";
        }
      }

      // Validate suffix
      const suffixValidation = validateSuffix(values.suffix);
      if (suffixValidation) {
        errors.suffix = suffixValidation;
      }

      return errors;
    },
    [existingTriggers, excludeId]
  );

  // Track suffix error separately for real-time validation
  const [suffixErrorState, setSuffixErrorState] = useState<string | null>(null);

  // Use the generic useFormState hook
  const form = useFormState<DictionaryFormValues>({
    initialValues: mergedInitialValues,
    validate,
    onSubmit,
  });

  // Override setValue to validate suffix on change
  const setValue = useCallback(
    <K extends keyof DictionaryFormValues>(field: K, value: DictionaryFormValues[K]) => {
      form.setValue(field, value);
      // Real-time validation for suffix field
      if (field === "suffix" && typeof value === "string") {
        const error = validateSuffix(value);
        setSuffixErrorState(error);
      }
    },
    [form]
  );

  const toggleSettings = useCallback(() => {
    setIsSettingsOpen((prev) => !prev);
  }, []);

  const reset = useCallback(() => {
    form.reset();
    setIsSettingsOpen(false);
    setSuffixErrorState(null);
  }, [form]);

  const hasSettings =
    form.values.suffix !== "" ||
    form.values.autoEnter ||
    form.values.disableSuffix ||
    form.values.completeMatchOnly;

  // Use real-time suffix error if available, otherwise use form error
  const suffixError = suffixErrorState ?? form.errors.suffix ?? null;

  return {
    values: form.values,
    triggerError: form.errors.trigger ?? null,
    suffixError,
    isSubmitting: form.isSubmitting,
    isSettingsOpen,
    setValue,
    toggleSettings,
    handleSubmit: form.handleSubmit,
    reset,
    hasSettings,
    isDirty: form.isDirty,
  };
}

import { useCallback, useMemo } from "react";
import { useFormState } from "./useFormState";
import { validateRegexPattern } from "../lib/validation";
import type { OverrideMode } from "../types/windowContext";

/**
 * Form values for window context creation/editing.
 */
export interface WindowContextFormValues {
  name: string;
  appName: string;
  bundleId?: string;
  titlePattern: string;
  commandMode: OverrideMode;
  dictionaryMode: OverrideMode;
  dictionaryEntryIds: string[];
  priority: number;
  enabled: boolean;
}

/**
 * Configuration options for useWindowContextForm hook.
 */
export interface UseWindowContextFormOptions {
  /** Initial values for editing mode */
  initialValues?: Partial<WindowContextFormValues>;
  /** Callback when form is submitted successfully */
  onSubmit?: (values: WindowContextFormValues) => Promise<void>;
}

/**
 * Return type of the useWindowContextForm hook.
 */
export interface UseWindowContextFormReturn {
  /** Current form values */
  values: WindowContextFormValues;
  /** Name validation error */
  nameError: string | null;
  /** App name validation error */
  appNameError: string | null;
  /** Pattern validation error */
  patternError: string | null;
  /** Whether the form is submitting */
  isSubmitting: boolean;
  /** Set a form value */
  setValue: <K extends keyof WindowContextFormValues>(
    field: K,
    value: WindowContextFormValues[K]
  ) => void;
  /** Handle app selection from combobox */
  handleAppSelect: (appName: string, bundleId?: string) => void;
  /** Handle form submission */
  handleSubmit: (e?: React.FormEvent) => Promise<boolean>;
  /** Reset form to initial state */
  reset: () => void;
  /** Whether form has unsaved changes */
  isDirty: boolean;
}

const DEFAULT_VALUES: WindowContextFormValues = {
  name: "",
  appName: "",
  bundleId: undefined,
  titlePattern: "",
  commandMode: "merge",
  dictionaryMode: "merge",
  dictionaryEntryIds: [],
  priority: 0,
  enabled: true,
};

/**
 * Hook for managing window context form state with validation.
 *
 * Composes with useFormState for base form state management and adds
 * window context-specific validation and app selection functionality.
 *
 * @example
 * const form = useWindowContextForm({
 *   onSubmit: async (values) => {
 *     await createContext.mutateAsync(values);
 *   },
 * });
 */
export function useWindowContextForm(
  options: UseWindowContextFormOptions = {}
): UseWindowContextFormReturn {
  const { initialValues, onSubmit } = options;

  const mergedInitialValues = useMemo(
    () => ({ ...DEFAULT_VALUES, ...initialValues }),
    [initialValues]
  );

  // Create validation function
  const validate = useCallback(
    (values: WindowContextFormValues): Partial<Record<keyof WindowContextFormValues, string>> => {
      const errors: Partial<Record<keyof WindowContextFormValues, string>> = {};

      // Validate name
      if (!values.name.trim()) {
        errors.name = "Name is required";
      }

      // Validate app name
      if (!values.appName.trim()) {
        errors.appName = "App name is required";
      }

      // Validate pattern
      const patternValidation = validateRegexPattern(values.titlePattern);
      if (patternValidation) {
        errors.titlePattern = patternValidation;
      }

      return errors;
    },
    []
  );

  // Use the generic useFormState hook
  const form = useFormState<WindowContextFormValues>({
    initialValues: mergedInitialValues,
    validate,
    onSubmit,
  });

  const handleAppSelect = useCallback(
    (appName: string, bundleId?: string) => {
      form.setValues({ appName, bundleId });
    },
    [form]
  );

  // Override setValue to clear bundleId when appName is manually typed
  const setValue = useCallback(
    <K extends keyof WindowContextFormValues>(
      field: K,
      value: WindowContextFormValues[K]
    ) => {
      if (field === "appName") {
        form.setValues({ appName: value as string, bundleId: undefined });
      } else {
        form.setValue(field, value);
      }
    },
    [form]
  );

  return {
    values: form.values,
    nameError: form.errors.name ?? null,
    appNameError: form.errors.appName ?? null,
    patternError: form.errors.titlePattern ?? null,
    isSubmitting: form.isSubmitting,
    setValue,
    handleAppSelect,
    handleSubmit: form.handleSubmit,
    reset: form.reset,
    isDirty: form.isDirty,
  };
}

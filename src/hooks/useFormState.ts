import { useState, useCallback, useMemo } from "react";

/**
 * Configuration options for useFormState hook.
 */
export interface UseFormStateOptions<T extends Record<string, unknown>> {
  /** Initial values for the form */
  initialValues: T;
  /** Optional validation function that returns errors */
  validate?: (values: T) => Partial<Record<keyof T, string>>;
  /** Callback when form is submitted successfully */
  onSubmit?: (values: T) => Promise<void> | void;
}

/**
 * Return type of the useFormState hook.
 */
export interface UseFormStateReturn<T> {
  /** Current form values */
  values: T;
  /** Current validation errors by field */
  errors: Partial<Record<keyof T, string>>;
  /** Whether the form is currently submitting */
  isSubmitting: boolean;
  /** Set a single field value */
  setValue: <K extends keyof T>(field: K, value: T[K]) => void;
  /** Set multiple field values at once */
  setValues: (values: Partial<T>) => void;
  /** Set an error for a field */
  setError: (field: keyof T, error: string | null) => void;
  /** Clear error for a field */
  clearError: (field: keyof T) => void;
  /** Clear all errors */
  clearErrors: () => void;
  /** Handle form submission */
  handleSubmit: (e?: React.FormEvent) => Promise<boolean>;
  /** Reset form to initial values */
  reset: () => void;
  /** Whether any values have changed from initial */
  isDirty: boolean;
  /** Validate all fields and return whether valid */
  validateAll: () => boolean;
}

/**
 * Generic hook for managing form state, validation, and submission.
 *
 * Provides a consistent pattern for form handling across the application,
 * reducing boilerplate and ensuring consistent behavior.
 *
 * @example
 * const form = useFormState({
 *   initialValues: { name: "", email: "" },
 *   validate: (values) => ({
 *     name: values.name ? null : "Name is required",
 *     email: values.email.includes("@") ? null : "Invalid email",
 *   }),
 *   onSubmit: async (values) => {
 *     await saveUser(values);
 *   },
 * });
 */
export function useFormState<T extends Record<string, unknown>>(
  options: UseFormStateOptions<T>
): UseFormStateReturn<T> {
  const { initialValues, validate, onSubmit } = options;

  const [values, setValuesState] = useState<T>(initialValues);
  const [errors, setErrors] = useState<Partial<Record<keyof T, string>>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);

  const setValue = useCallback(<K extends keyof T>(field: K, value: T[K]) => {
    setValuesState((prev) => ({ ...prev, [field]: value }));
    // Clear error when field is modified
    setErrors((prev) => {
      if (prev[field]) {
        const { [field]: _, ...rest } = prev;
        return rest as Partial<Record<keyof T, string>>;
      }
      return prev;
    });
  }, []);

  const setValues = useCallback((newValues: Partial<T>) => {
    setValuesState((prev) => ({ ...prev, ...newValues }));
    // Clear errors for modified fields
    setErrors((prev) => {
      const newErrors = { ...prev };
      for (const key of Object.keys(newValues)) {
        delete newErrors[key as keyof T];
      }
      return newErrors;
    });
  }, []);

  const setError = useCallback((field: keyof T, error: string | null) => {
    setErrors((prev) => {
      if (error === null) {
        const { [field]: _, ...rest } = prev;
        return rest as Partial<Record<keyof T, string>>;
      }
      return { ...prev, [field]: error };
    });
  }, []);

  const clearError = useCallback((field: keyof T) => {
    setErrors((prev) => {
      const { [field]: _, ...rest } = prev;
      return rest as Partial<Record<keyof T, string>>;
    });
  }, []);

  const clearErrors = useCallback(() => {
    setErrors({});
  }, []);

  const validateAll = useCallback((): boolean => {
    if (!validate) return true;

    const validationErrors = validate(values);
    const filteredErrors: Record<string, string> = {};

    for (const [key, value] of Object.entries(validationErrors)) {
      if (value) {
        filteredErrors[key] = value;
      }
    }

    setErrors(filteredErrors as Partial<Record<keyof T, string>>);
    return Object.keys(filteredErrors).length === 0;
  }, [validate, values]);

  const handleSubmit = useCallback(
    async (e?: React.FormEvent): Promise<boolean> => {
      if (e) {
        e.preventDefault();
      }

      if (!validateAll()) {
        return false;
      }

      if (!onSubmit) {
        return true;
      }

      setIsSubmitting(true);
      try {
        await onSubmit(values);
        return true;
      } finally {
        setIsSubmitting(false);
      }
    },
    [validateAll, onSubmit, values]
  );

  const reset = useCallback(() => {
    setValuesState(initialValues);
    setErrors({});
  }, [initialValues]);

  const isDirty = useMemo(() => {
    return JSON.stringify(values) !== JSON.stringify(initialValues);
  }, [values, initialValues]);

  return {
    values,
    errors,
    isSubmitting,
    setValue,
    setValues,
    setError,
    clearError,
    clearErrors,
    handleSubmit,
    reset,
    isDirty,
    validateAll,
  };
}

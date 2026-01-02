---
paths: "src/**/*.ts, src/**/*.tsx"
---

# Frontend Form State Pattern

## useFormState<T> Hook

Use the generic `useFormState` hook for form management:

```typescript
import { useFormState } from "./useFormState";

const form = useFormState({
  initialValues: { name: "", email: "" },
  validate: (values) => ({
    name: values.name ? null : "Name is required",
    email: values.email.includes("@") ? null : "Invalid email",
  }),
  onSubmit: async (values) => {
    await saveUser(values);
  },
});
```

## Hook Configuration

```typescript
interface UseFormStateOptions<T extends Record<string, unknown>> {
  /** Initial values for the form */
  initialValues: T;
  /** Optional validation function that returns errors */
  validate?: (values: T) => Partial<Record<keyof T, string>>;
  /** Callback when form is submitted successfully */
  onSubmit?: (values: T) => Promise<void> | void;
}
```

## Return Value

```typescript
interface UseFormStateReturn<T> {
  values: T;                                    // Current form values
  errors: Partial<Record<keyof T, string>>;     // Validation errors by field
  isSubmitting: boolean;                        // Submission in progress
  setValue: <K extends keyof T>(field: K, value: T[K]) => void;
  setValues: (values: Partial<T>) => void;
  setError: (field: keyof T, error: string | null) => void;
  clearError: (field: keyof T) => void;
  clearErrors: () => void;
  handleSubmit: (e?: React.FormEvent) => Promise<boolean>;
  reset: () => void;
  isDirty: boolean;                             // Values changed from initial
  validateAll: () => boolean;
}
```

## Validation Function Signature

Return `null` or `undefined` for valid fields, string error message for invalid:

```typescript
validate: (values: T) => Partial<Record<keyof T, string>>

// Example
validate: (values) => ({
  trigger: values.trigger ? null : "Trigger is required",
  suffix: values.suffix.length > 10 ? "Suffix too long" : null,
})
```

## Domain-Specific Form Hooks

Compose `useFormState` with domain-specific logic:

```typescript
// useDictionaryForm.ts
export function useDictionaryForm(options: UseDictionaryFormOptions): UseDictionaryFormReturn {
  const { existingTriggers, excludeId, initialValues, onSubmit } = options;

  // Custom validation with duplicate detection
  const validate = useCallback(
    (values: DictionaryFormValues) => {
      const errors: Partial<Record<keyof DictionaryFormValues, string>> = {};

      const triggerValidation = validateTrigger(values.trigger);
      if (triggerValidation) {
        errors.trigger = triggerValidation;
      } else if (isDuplicateTrigger(values.trigger, existingTriggers)) {
        errors.trigger = "This trigger already exists";
      }

      return errors;
    },
    [existingTriggers, excludeId]
  );

  // Base form state
  const form = useFormState<DictionaryFormValues>({
    initialValues: mergedInitialValues,
    validate,
    onSubmit,
  });

  // Domain-specific additions
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  return {
    values: form.values,
    triggerError: form.errors.trigger ?? null,
    // ... other domain-specific return values
  };
}
```

## Form Usage in Components

```tsx
function DictionaryEntryForm() {
  const form = useDictionaryForm({
    existingTriggers: entries.map(e => e.trigger.toLowerCase()),
    onSubmit: async (values) => {
      await addEntry.mutateAsync(values);
    },
  });

  return (
    <form onSubmit={form.handleSubmit}>
      <input
        value={form.values.trigger}
        onChange={(e) => form.setValue("trigger", e.target.value)}
      />
      {form.triggerError && <span className="error">{form.triggerError}</span>}
      <button type="submit" disabled={form.isSubmitting}>
        Save
      </button>
    </form>
  );
}
```

## Anti-Patterns

### Direct useState for form fields

```typescript
// BAD: Multiple useState calls, manual validation
function MyForm() {
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [nameError, setNameError] = useState("");
  const [emailError, setEmailError] = useState("");

  const handleSubmit = () => {
    if (!name) setNameError("Required");
    if (!email.includes("@")) setEmailError("Invalid");
    // ...
  };
}

// GOOD: useFormState handles all state
function MyForm() {
  const form = useFormState({
    initialValues: { name: "", email: "" },
    validate: (values) => ({
      name: values.name ? null : "Required",
      email: values.email.includes("@") ? null : "Invalid",
    }),
  });
}
```

### Validation in component body

```typescript
// BAD: Validation logic scattered in component
function MyForm() {
  const [values, setValues] = useState({ name: "" });

  const nameError = values.name.length < 3 ? "Too short" : null;

  return <input onChange={...} />;
}

// GOOD: Centralized validation function
const form = useFormState({
  initialValues: { name: "" },
  validate: (values) => ({
    name: values.name.length < 3 ? "Too short" : null,
  }),
});
```

### Missing isDirty check before navigation

```typescript
// BAD: No unsaved changes warning
const handleNavigate = () => navigate("/other");

// GOOD: Check for unsaved changes
const handleNavigate = () => {
  if (form.isDirty && !confirm("Discard changes?")) return;
  navigate("/other");
};
```

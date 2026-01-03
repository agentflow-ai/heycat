---
paths: "src/**/*.ts, src/**/*.tsx"
---

# Form State Pattern

```
                            FORM DATA FLOW
───────────────────────────────────────────────────────────────────────
User Input
    │
    ▼
form.setValue() ──▶ form.values ──▶ validate()
                                        │
                         ┌──────────────┴──────────────┐
                         ▼                             ▼
                   form.errors                   no errors
                   (show in UI)                        │
                                                       ▼
                                              form.handleSubmit()
                                                       │
                                                       ▼
                                               onSubmit() → invoke()
                                                       │
                                                       ▼
                                                   Backend
```

Use `useFormState<T>` hook for all forms (never raw useState for form fields).

```typescript
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

## Validation

Return `null` for valid, string error message for invalid:

```typescript
validate: (values: T) => Partial<Record<keyof T, string>>
```

## Domain-Specific Forms

Compose `useFormState` with domain logic (e.g., `useDictionaryForm` adds duplicate detection).

## Usage

```tsx
<form onSubmit={form.handleSubmit}>
  <input
    value={form.values.trigger}
    onChange={(e) => form.setValue("trigger", e.target.value)}
  />
  {form.errors.trigger && <span className="error">{form.errors.trigger}</span>}
  <button disabled={form.isSubmitting}>Save</button>
</form>
```

Check `form.isDirty` before navigation to warn about unsaved changes.

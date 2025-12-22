---
status: completed
created: 2025-12-22
completed: 2025-12-22
dependencies: ["settings-panel-ui"]
review_round: 1
---

# Spec: Add frontend validation for suffix field (max 5 chars)

## Description

Add validation to the suffix text field in the settings panel to prevent users from entering more than 5 characters. Display a clear error message when validation fails and prevent saving until the error is fixed.

## Acceptance Criteria

- [ ] Suffix field has `maxLength={5}` attribute as first line of defense
- [ ] Validation error shown if suffix exceeds 5 characters (edge case: paste)
- [ ] Error message: "Suffix must be 5 characters or less"
- [ ] Save button disabled while validation error exists
- [ ] Error clears when suffix is corrected to ≤5 characters

## Test Cases

- [ ] Type 5 characters → no error, save enabled
- [ ] Type 6+ characters → error shown, save disabled
- [ ] Paste long text → error shown, save disabled
- [ ] Delete characters to ≤5 → error clears, save enabled
- [ ] Empty suffix → no error (valid, means no suffix)

## Dependencies

- `settings-panel-ui` - Settings panel must exist to add validation to it

## Preconditions

- Settings panel with suffix input field exists

## Implementation Notes

### Data Flow Position
```
Settings Panel
       ↓
Suffix Input ← This spec (validation)
       ↓
Validation state
       ↓
Save button (disabled if invalid)
```

### Validation Logic

```tsx
// src/pages/Dictionary.tsx

function SettingsPanel({ suffix, setSuffix, onSave, ... }) {
  const [suffixError, setSuffixError] = useState<string | null>(null);

  const validateSuffix = (value: string) => {
    if (value.length > 5) {
      setSuffixError("Suffix must be 5 characters or less");
      return false;
    }
    setSuffixError(null);
    return true;
  };

  const handleSuffixChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setSuffix(value);
    validateSuffix(value);
  };

  const handleSave = () => {
    if (validateSuffix(suffix)) {
      onSave();
    }
  };

  return (
    <div className="settings-panel">
      <div className="setting-row">
        <label>Suffix</label>
        <div className="input-with-error">
          <input
            type="text"
            value={suffix}
            onChange={handleSuffixChange}
            placeholder="e.g., . or ?"
            maxLength={5}
            className={suffixError ? "input-error" : ""}
          />
          {suffixError && (
            <span className="error-message">{suffixError}</span>
          )}
        </div>
      </div>

      {/* Auto-enter toggle */}

      <button
        onClick={handleSave}
        disabled={!!suffixError}
      >
        Save
      </button>
    </div>
  );
}
```

### CSS Additions (`src/pages/Dictionary.css`)

```css
.input-with-error {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.input-error {
  border-color: var(--error) !important;
}

.error-message {
  color: var(--error);
  font-size: 12px;
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

### Testing Strategy

**Frontend (TypeScript/React):**

```typescript
// src/pages/__tests__/Dictionary.test.tsx
describe("Suffix Validation", () => {
  beforeEach(() => {
    // Setup mock entries
    mockInvoke.mockResolvedValue([
      { id: "1", trigger: "brb", expansion: "be right back" },
    ]);
  });

  it("shows error when suffix exceeds 5 characters", async () => {
    render(<Dictionary />);
    await waitFor(() => screen.getByText("brb"));

    // Open settings panel
    await userEvent.click(screen.getByRole("button", { name: /settings/i }));

    // Type long suffix (simulate paste by setting value directly)
    const suffixInput = screen.getByPlaceholderText("e.g., . or ?");

    // Use fireEvent to bypass maxLength for paste simulation
    fireEvent.change(suffixInput, { target: { value: "123456" } });

    // Error should be shown
    expect(screen.getByText("Suffix must be 5 characters or less")).toBeInTheDocument();

    // Save button should be disabled
    const saveButton = screen.getByRole("button", { name: /save/i });
    expect(saveButton).toBeDisabled();
  });

  it("allows exactly 5 characters without error", async () => {
    render(<Dictionary />);
    await waitFor(() => screen.getByText("brb"));

    await userEvent.click(screen.getByRole("button", { name: /settings/i }));

    const suffixInput = screen.getByPlaceholderText("e.g., . or ?");
    await userEvent.type(suffixInput, "12345");

    // No error should be shown
    expect(screen.queryByText("Suffix must be 5 characters or less")).not.toBeInTheDocument();

    // Save button should be enabled
    const saveButton = screen.getByRole("button", { name: /save/i });
    expect(saveButton).toBeEnabled();
  });

  it("clears error when suffix is corrected", async () => {
    render(<Dictionary />);
    await waitFor(() => screen.getByText("brb"));

    await userEvent.click(screen.getByRole("button", { name: /settings/i }));

    const suffixInput = screen.getByPlaceholderText("e.g., . or ?");

    // Set invalid value
    fireEvent.change(suffixInput, { target: { value: "123456" } });
    expect(screen.getByText("Suffix must be 5 characters or less")).toBeInTheDocument();

    // Correct to valid value
    fireEvent.change(suffixInput, { target: { value: "12345" } });

    // Error should be cleared
    expect(screen.queryByText("Suffix must be 5 characters or less")).not.toBeInTheDocument();
  });

  it("allows empty suffix", async () => {
    render(<Dictionary />);
    await waitFor(() => screen.getByText("brb"));

    await userEvent.click(screen.getByRole("button", { name: /settings/i }));

    const suffixInput = screen.getByPlaceholderText("e.g., . or ?");
    // Input is empty by default

    // No error should be shown
    expect(screen.queryByText("Suffix must be 5 characters or less")).not.toBeInTheDocument();

    // Save button should be enabled
    const saveButton = screen.getByRole("button", { name: /save/i });
    expect(saveButton).toBeEnabled();
  });
});
```

### Edge Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| Paste long text | Error shown, save disabled |
| Emoji (multi-byte) | Counts as 1+ chars depending on emoji |
| Special chars | Allowed (., !, ?, etc.) |
| Whitespace only | Allowed (user's choice) |
| Leading/trailing spaces | Allowed (counts toward limit) |

## Related Specs

- [settings-panel-ui.spec.md](./settings-panel-ui.spec.md) - Provides the settings panel to validate

## Integration Points

- Production call site: `src/pages/Dictionary.tsx` - SettingsPanel component
- Connects to: Settings panel UI, save functionality

## Integration Test

- Test location: `src/pages/__tests__/Dictionary.test.tsx`
- Verification: [ ] Integration test passes

## Review

**Reviewed:** 2025-12-22
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Suffix field has `maxLength={5}` attribute as first line of defense | PASS | Dictionary.tsx:48 - `maxLength={5}` on Input component |
| Validation error shown if suffix exceeds 5 characters (edge case: paste) | PASS | Dictionary.tsx:98-105 `validateSuffix` function, line 54 error display |
| Error message: "Suffix must be 5 characters or less" | PASS | Dictionary.tsx:100 exact message |
| Save button disabled while validation error exists | PASS | Dictionary.tsx:196 `disabled={isSubmitting \|\| !!suffixError}`, line 292 `disabled={!!editSuffixError}` |
| Error clears when suffix is corrected to <=5 characters | PASS | Dictionary.tsx:102-104 clears error when validation passes |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Type 5 characters -> no error, save enabled | PASS | Dictionary.test.tsx:569-591 |
| Type 6+ characters -> error shown, save disabled | PASS | Dictionary.test.tsx:593-621 (via paste simulation) |
| Paste long text -> error shown, save disabled | PASS | Dictionary.test.tsx:593-621 uses fireEvent.change to bypass maxLength |
| Delete characters to <=5 -> error clears, save enabled | PASS | Dictionary.test.tsx:623-652 |
| Empty suffix -> no error (valid) | PASS | Dictionary.test.tsx:654-673 |
| Edit mode: suffix validation | PASS | Dictionary.test.tsx:675-737 |

### Pre-Review Gate Results

**Build Warning Check:**
```
warning: method `get` is never used (pre-existing, not related to this spec)
```
Result: PASS - No new warnings from this spec.

**Command Registration Check:** N/A - Frontend-only spec, no new Tauri commands.

**Event Subscription Check:** N/A - No new events added.

### Code Quality

**Strengths:**
- Clean separation of validation logic in dedicated `validateSuffix` function
- Validation applied consistently in both Add form and Edit mode
- Proper use of existing project CSS utilities (`border-error`, `text-error`)
- Tests cover both typing and paste edge cases using `fireEvent.change` to bypass maxLength
- Reusable `SettingsPanel` component accepts `suffixError` prop for error display
- Proper aria attributes for accessibility (`aria-invalid`)

**Concerns:**
- None identified

### Integration Verification

**Data Flow (Frontend-Only):**
```
User types in suffix field
       |
       v
handleSuffixChange (Dictionary.tsx:107-110)
       | setSuffix(value), validateSuffix(value)
       v
validateSuffix (Dictionary.tsx:98-105)
       | length > 5 ? setSuffixError(msg) : setSuffixError(null)
       v
suffixError state
       |
       v
SettingsPanel receives suffixError prop (Dictionary.tsx:208)
       | Displays error message, applies border-error class
       v
Add/Save button disabled={!!suffixError} (Dictionary.tsx:196, 292)
```

**Production Call Sites:**
| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| validateSuffix | fn | Dictionary.tsx:109, 128 | YES (via form handlers) |
| suffixError state | state | Dictionary.tsx:93 | YES (Add form) |
| editSuffixError state | state | Dictionary.tsx:431 | YES (Edit mode) |
| SettingsPanel.suffixError prop | prop | Dictionary.tsx:18, 208, 314 | YES |

### Verdict

**APPROVED** - All acceptance criteria are met with comprehensive test coverage. The implementation correctly validates suffix length in both add and edit modes, displays appropriate error messages, and disables save buttons when validation fails. Tests properly simulate paste behavior to bypass maxLength attribute.

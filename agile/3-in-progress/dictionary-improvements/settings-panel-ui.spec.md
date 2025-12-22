---
status: in-progress
created: 2025-12-22
completed: null
dependencies: ["backend-storage-update"]
---

# Spec: Add expandable settings panel to dictionary entry UI

## Description

Add a collapsible settings panel to each dictionary entry in the Dictionary page. The panel is toggled by clicking a settings/gear icon and contains:
- Suffix text field (freeform, with placeholder examples like "." or "?")
- Auto-enter toggle switch

The panel appears both on existing entries (edit mode) and in the add entry form.

## Acceptance Criteria

- [ ] Each dictionary entry has a settings icon (gear/cog)
- [ ] Clicking the icon toggles a collapsible settings panel
- [ ] Settings panel contains a "Suffix" text input with placeholder "e.g., . or ?"
- [ ] Settings panel contains an "Auto-enter" toggle switch
- [ ] AddEntryForm also has the settings panel
- [ ] Saving an entry persists suffix and autoEnter values
- [ ] Loading entries displays correct suffix and autoEnter values
- [ ] Settings icon indicates when settings are configured (visual indicator)

## Test Cases

- [ ] Click settings icon → panel expands
- [ ] Click settings icon again → panel collapses
- [ ] Enter suffix value → value shown in input
- [ ] Toggle auto-enter on → toggle shows on state
- [ ] Save entry with settings → invoke called with suffix/autoEnter params
- [ ] Load entry with settings → settings panel shows correct values when expanded
- [ ] Entry with no settings → settings panel shows empty/default values

## Dependencies

- `backend-storage-update` - Tauri commands must accept suffix/autoEnter params

## Preconditions

- useDictionary hook updated to pass suffix/autoEnter to mutations
- DictionaryEntry TypeScript interface has suffix and autoEnter fields

## Implementation Notes

### Data Flow Position
```
Dictionary Page
       ↓
EntryItem / AddEntryForm ← This spec
       ↓
Settings Panel (collapsible) ← This spec
       ↓
useDictionary hook → invoke() → Tauri commands
```

### Component Structure

```tsx
// src/pages/Dictionary.tsx

function EntryItem({ entry }: { entry: DictionaryEntry }) {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [suffix, setSuffix] = useState(entry.suffix || "");
  const [autoEnter, setAutoEnter] = useState(entry.autoEnter || false);

  const hasSettings = entry.suffix || entry.autoEnter;

  return (
    <div className="entry-item">
      <div className="entry-header">
        <span>{entry.trigger} → {entry.expansion}</span>
        <div className="entry-actions">
          <button onClick={() => setIsSettingsOpen(!isSettingsOpen)}>
            <SettingsIcon hasIndicator={hasSettings} />
          </button>
          {/* Edit, Delete buttons */}
        </div>
      </div>

      {isSettingsOpen && (
        <div className="settings-panel">
          <div className="setting-row">
            <label>Suffix</label>
            <input
              type="text"
              value={suffix}
              onChange={(e) => setSuffix(e.target.value)}
              placeholder="e.g., . or ?"
              maxLength={5}
            />
          </div>
          <div className="setting-row">
            <label>Auto-enter</label>
            <Toggle
              checked={autoEnter}
              onChange={setAutoEnter}
            />
          </div>
        </div>
      )}
    </div>
  );
}
```

### CSS Additions (`src/pages/Dictionary.css`)

```css
.settings-panel {
  padding: 12px;
  background: var(--surface-secondary);
  border-radius: 8px;
  margin-top: 8px;
  animation: slideDown 0.2s ease-out;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.setting-row label {
  font-weight: 500;
}

.setting-row input {
  width: 80px;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 4px;
}

.settings-icon.has-settings {
  color: var(--accent);
}
```

### Testing Strategy

**Frontend (TypeScript/React):**

```typescript
// src/pages/__tests__/Dictionary.test.tsx
describe("Dictionary Settings Panel", () => {
  it("toggles settings panel on icon click", async () => {
    render(<Dictionary />);

    // Wait for entries to load
    await waitFor(() => screen.getByText("brb"));

    // Settings panel should be hidden initially
    expect(screen.queryByText("Suffix")).not.toBeInTheDocument();

    // Click settings icon
    const settingsIcon = screen.getByRole("button", { name: /settings/i });
    await userEvent.click(settingsIcon);

    // Settings panel should now be visible
    expect(screen.getByText("Suffix")).toBeInTheDocument();
    expect(screen.getByText("Auto-enter")).toBeInTheDocument();
  });

  it("saves entry with suffix and autoEnter", async () => {
    render(<Dictionary />);
    await waitFor(() => screen.getByText("brb"));

    // Open settings
    await userEvent.click(screen.getByRole("button", { name: /settings/i }));

    // Enter suffix
    const suffixInput = screen.getByPlaceholderText("e.g., . or ?");
    await userEvent.type(suffixInput, ".");

    // Toggle auto-enter
    await userEvent.click(screen.getByRole("switch"));

    // Save
    await userEvent.click(screen.getByRole("button", { name: /save/i }));

    // Verify invoke was called with new fields
    expect(mockInvoke).toHaveBeenCalledWith("update_dictionary_entry", {
      id: expect.any(String),
      trigger: "brb",
      expansion: "be right back",
      suffix: ".",
      autoEnter: true,
    });
  });

  it("shows indicator when entry has settings", async () => {
    // Mock entry with settings
    mockInvoke.mockResolvedValueOnce([
      { id: "1", trigger: "brb", expansion: "be right back", suffix: ".", autoEnter: true },
    ]);

    render(<Dictionary />);
    await waitFor(() => screen.getByText("brb"));

    const settingsIcon = screen.getByRole("button", { name: /settings/i });
    expect(settingsIcon).toHaveClass("has-settings");
  });
});
```

### UI Mockup

```
┌────────────────────────────────────────────────────────┐
│  brb → be right back                    [✎] [🗑] [⚙️•] │
├────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐  │
│  │  Suffix:      [.____]  (placeholder: e.g., . ?) │  │
│  │  Auto-enter:  [○━━━]                            │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘

• The dot on ⚙️ indicates settings are configured
• Panel slides down when settings icon is clicked
```

## Related Specs

- [backend-storage-update.spec.md](./backend-storage-update.spec.md) - Commands accept new fields
- [suffix-validation.spec.md](./suffix-validation.spec.md) - Validates suffix length
- [data-model-update.spec.md](./data-model-update.spec.md) - TypeScript interface

## Integration Points

- Production call site: `src/pages/Dictionary.tsx` - Main dictionary page
- Connects to: useDictionary hook (mutations), DictionaryEntry type

## Integration Test

- Test location: `src/pages/__tests__/Dictionary.test.tsx`
- Verification: [ ] Integration test passes

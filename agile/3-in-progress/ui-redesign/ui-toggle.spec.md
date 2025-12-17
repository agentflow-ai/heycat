---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - layout-shell
---

# Spec: UI Toggle (Dev Preview)

## Description

Add a development toggle to switch between the old UI and new UI during the redesign process. This allows easy A/B comparison and incremental review of new components.

**Note:** This is a temporary dev feature that will be removed in the `integration-and-cleanup` spec.

## Acceptance Criteria

- [ ] Toggle switch in a fixed position (bottom-left corner, above footer)
- [ ] Toggle persists preference to localStorage
- [ ] When "New UI" is selected, render the new AppShell layout
- [ ] When "Old UI" is selected, render the existing Sidebar-based layout
- [ ] Visual indicator shows which UI mode is active
- [ ] Keyboard shortcut to toggle (e.g., Ctrl+Shift+U)
- [ ] Only visible in development mode (not in production builds)

## Test Cases

- [ ] Toggle switches between old and new UI
- [ ] Preference persists across page reloads
- [ ] Keyboard shortcut works
- [ ] Toggle is hidden in production builds
- [ ] Both UIs render without errors when toggled

## Dependencies

- layout-shell (new UI to toggle to)

## Preconditions

- Layout shell spec completed
- Old UI still functional

## Implementation Notes

**Files to create/modify:**
```
src/components/dev/
├── UIToggle.tsx        # Toggle component
└── index.ts

src/App.tsx             # Conditional rendering based on toggle
src/hooks/useUIMode.ts  # Hook to manage UI mode state
```

**Implementation approach:**
```tsx
// useUIMode.ts
const UI_MODE_KEY = 'heycat-ui-mode';
type UIMode = 'old' | 'new';

function useUIMode() {
  const [mode, setMode] = useState<UIMode>(() =>
    localStorage.getItem(UI_MODE_KEY) as UIMode || 'old'
  );
  // ...
}

// App.tsx
function App() {
  const { mode } = useUIMode();

  if (mode === 'new') {
    return <AppShell>...</AppShell>;
  }
  return <OldApp />;
}
```

**Toggle styling:**
- Fixed position: bottom-left, 16px from edges
- Small pill with "Old UI" / "New UI" labels
- Semi-transparent background
- Z-index above content but below modals

**Visibility control:**
- Use `import.meta.env.DEV` to conditionally render
- Or use a feature flag in settings

## Related Specs

- layout-shell (dependency)
- integration-and-cleanup (removes this toggle)

## Integration Points

- Production call site: `src/App.tsx`
- Connects to: Old UI components, new AppShell

## Integration Test

- Test location: `src/components/dev/__tests__/UIToggle.test.tsx`
- Verification: [ ] Integration test passes

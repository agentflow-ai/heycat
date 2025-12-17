---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - design-system-foundation
---

# Spec: Base UI Components

## Description

Build foundational UI primitives using Radix UI and Tailwind CSS. These are the reusable building blocks for all pages and features.

**Source of Truth:** `ui.md` - Part 3: Component Library (sections 3.1-3.4)

## Acceptance Criteria

### Buttons (ui.md 3.1)
- [ ] Primary button with gradient background, hover elevation, press states
- [ ] Secondary button with orange border, white background
- [ ] Ghost button with transparent background
- [ ] Danger button with red background
- [ ] All buttons support disabled state and loading spinner

### Cards (ui.md 3.2)
- [ ] Standard card with shadow, border, hover elevation
- [ ] Interactive card with cursor pointer and enhanced hover
- [ ] Status card with colored left border and icon

### Inputs (ui.md 3.3)
- [ ] Text input with focus ring (teal), placeholder styling
- [ ] Select/dropdown using Radix Select with custom styling
- [ ] Toggle switch (pill-shaped, orange when active)

### Status Indicators (ui.md 3.4)
- [ ] Recording dot with pulse animation (red, 1.5s interval)
- [ ] Listening glow effect (teal, 2s breathing)
- [ ] Audio level meter (horizontal bar with gradient zones)

## Test Cases

- [ ] Button variants render with correct styles
- [ ] Button hover/press states animate correctly
- [ ] Card hover elevation works
- [ ] Input focus ring appears on focus
- [ ] Toggle switch animates on click
- [ ] Recording dot pulses at correct interval
- [ ] Audio level meter responds to value changes

## Dependencies

- design-system-foundation (uses CSS tokens)

## Preconditions

- Design system foundation spec completed
- Radix UI packages installed (@radix-ui/react-select, @radix-ui/react-switch, etc.)
- Framer Motion installed for animations

## Implementation Notes

**Files to create:**
```
src/components/ui/
├── Button.tsx
├── Card.tsx
├── Input.tsx
├── Select.tsx
├── Toggle.tsx
├── StatusIndicator.tsx
├── AudioLevelMeter.tsx
└── index.ts
```

**Radix UI components to use:**
- `@radix-ui/react-select` for Select
- `@radix-ui/react-switch` for Toggle
- `@radix-ui/react-slot` for Button asChild pattern

**Animation specs from ui.md:**
- Hover: scale 1.02, shadow elevation
- Press: scale 0.98, reduced shadow
- Recording pulse: 1.5s ease-in-out infinite
- Listening breathe: 2s ease-in-out infinite

## Related Specs

- design-system-foundation (dependency)
- layout-shell, status-pill-states, all pages (dependents)

## Integration Points

- Production call site: Used by all page components
- Connects to: Layout shell, all page specs

## Integration Test

- Test location: Component unit tests + Storybook stories
- Verification: [ ] Integration test passes

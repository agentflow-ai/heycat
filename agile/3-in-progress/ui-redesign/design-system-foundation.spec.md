---
status: in-progress
created: 2025-12-17
completed: null
dependencies: []
---

# Spec: Design System Foundation

## Description

Establish CSS variables, Tailwind theme configuration, and base styles for the HeyCat design system. This creates the foundational design tokens that all other UI components will use.

**Source of Truth:** `ui.md` - Part 1: Style Guide (sections 1.1-1.6)

## Acceptance Criteria

- [ ] Create `src/styles/globals.css` with all CSS custom properties from ui.md section 1.2-1.6
- [ ] Brand colors defined (orange, cream, teal, purple variants)
- [ ] Neutral color scale (50-900) defined
- [ ] Semantic colors defined (success, warning, error, info)
- [ ] State colors defined (recording, listening, processing)
- [ ] Typography scale defined (xs through 2xl)
- [ ] Font families configured (Inter for sans, JetBrains Mono for mono)
- [ ] Spacing scale defined (1-12 based on 4px)
- [ ] Border radius scale defined (sm, md, lg, xl, full)
- [ ] Shadow tokens defined (sm, md, lg, xl, glow, window)
- [ ] Animation timing functions and durations defined
- [ ] Configure Tailwind theme to extend with HeyCat design tokens
- [ ] Dark mode color overrides defined (ui.md section 6.1)

## Test Cases

- [ ] CSS variables are accessible in browser dev tools
- [ ] Tailwind classes using custom theme work correctly (e.g., `bg-heycat-orange`)
- [ ] Dark mode toggle switches color scheme appropriately
- [ ] Font families load correctly (Inter, JetBrains Mono)

## Dependencies

None - this is the foundational spec.

## Preconditions

- Tailwind CSS installed and configured in the project
- Project builds successfully

## Implementation Notes

**Files to create/modify:**
- `src/styles/globals.css` - CSS custom properties
- `tailwind.config.js` - Theme extension with HeyCat tokens
- `src/styles/tailwind.css` - Tailwind directives

**Key design tokens from ui.md:**
```
Primary: #E8945A (orange), #F4C89A (light), #FDF6E8 (cream)
Accent: #5BB5B5 (teal), #3D8B8B (dark teal), #9B7BB5 (purple)
States: #EF4444 (recording), #5BB5B5 (listening), #F59E0B (processing)
```

## Related Specs

- base-ui-components (depends on this)
- All other specs transitively depend on this

## Integration Points

- Production call site: `src/main.tsx` imports globals.css
- Connects to: All React components via Tailwind classes

## Integration Test

- Test location: Visual inspection + Storybook stories
- Verification: [ ] Integration test passes

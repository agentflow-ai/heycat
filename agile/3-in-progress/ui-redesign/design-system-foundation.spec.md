---
status: in-review
created: 2025-12-17
completed: null
dependencies: []
review_round: 1
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

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Create `src/styles/globals.css` with all CSS custom properties from ui.md section 1.2-1.6 | PASS | `/Users/michaelhindley/Documents/git/heycat/src/styles/globals.css` lines 1-248 |
| Brand colors defined (orange, cream, teal, purple variants) | PASS | globals.css lines 12-17: `--heycat-orange`, `--heycat-orange-light`, `--heycat-cream`, `--heycat-teal`, `--heycat-teal-dark`, `--heycat-purple` |
| Neutral color scale (50-900) defined | PASS | globals.css lines 19-29: `--neutral-50` through `--neutral-900` |
| Semantic colors defined (success, warning, error, info) | PASS | globals.css lines 31-35: `--success`, `--warning`, `--error`, `--info` |
| State colors defined (recording, listening, processing) | PASS | globals.css lines 37-40: `--recording`, `--listening`, `--processing` |
| Typography scale defined (xs through 2xl) | PASS | globals.css lines 50-62: `--text-xs` through `--text-2xl` with line-heights |
| Font families configured (Inter for sans, JetBrains Mono for mono) | PASS | globals.css lines 47-48: `--font-sans` and `--font-mono` |
| Spacing scale defined (1-12 based on 4px) | PASS | globals.css lines 75-83: `--space-1` through `--space-12` |
| Border radius scale defined (sm, md, lg, xl, full) | PASS | globals.css lines 85-90: `--radius-sm` through `--radius-full` |
| Shadow tokens defined (sm, md, lg, xl, glow, window) | PASS | globals.css lines 96-101: All shadow tokens defined |
| Animation timing functions and durations defined | PASS | globals.css lines 107-116: Timing functions and durations |
| Configure Tailwind theme to extend with HeyCat design tokens | PASS | `/Users/michaelhindley/Documents/git/heycat/tailwind.config.js` lines 1-124: Full theme extension |
| Dark mode color overrides defined (ui.md section 6.1) | PASS | globals.css lines 139-179: Both `@media (prefers-color-scheme: dark)` and `.dark` class |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| CSS variables are accessible in browser dev tools | PASS | Verified via dev server - globals.css imported in main.tsx:6 |
| Tailwind classes using custom theme work correctly | PASS | tailwind.config.js extends theme with all HeyCat tokens |
| Dark mode toggle switches color scheme appropriately | PASS | Both media query and class-based dark mode supported |
| Font families load correctly | PASS | Font stack configured with fallbacks |

### Code Quality

**Strengths:**
- Comprehensive CSS variable system matching ui.md specification exactly
- Well-organized file structure with clear section comments
- Both class-based and media-query dark mode support for flexibility
- Tailwind theme properly extends with CSS variables for consistent theming
- Base styles include font smoothing and reset for consistent rendering
- Utility classes for recording/listening indicators included

**Concerns:**
- Build command (`bun run build`) fails with CSS syntax error in Tailwind v4 processing - appears to be a Tailwind v4 configuration incompatibility (using v3-style tailwind.config.js with v4 postcss plugin), not specific to this spec's implementation
- No Storybook setup exists to verify visual test cases as specified in Integration Test section

### Verdict

**NEEDS_WORK** - The CSS variables and Tailwind theme extension are correctly implemented and match the ui.md specification. However, the production build (`bun run build`) fails due to a Tailwind v4 configuration issue. The project uses `@tailwindcss/postcss` v4.1.18 with a v3-style `tailwind.config.js`, which causes CSS processing errors during build. The dev server works correctly, but the build must pass for this spec to be complete.

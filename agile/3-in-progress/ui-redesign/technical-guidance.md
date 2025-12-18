---
last-updated: 2025-12-18
status: draft
---

# Technical Guidance: UI Redesign

## Architecture Overview

### Design Philosophy

This UI redesign follows a **build alongside, then swap** migration strategy. New components are built in isolation in new directories, with a dev toggle allowing comparison between old and new UIs. Once complete, legacy code is removed.

**Source of Truth:** All visual design specifications are in `ui.md` (866 lines covering design system, layout, components, and screens).

### New Directory Structure

```
src/
├── components/
│   ├── ui/              # NEW: Base UI primitives (Button, Card, Input, etc.)
│   ├── layout/          # NEW: App shell components (Header, Sidebar, Footer)
│   ├── overlays/        # NEW: Modals, command palette, toasts
│   ├── dev/             # NEW: Dev-only components (UIToggle)
│   │
│   ├── Sidebar/         # LEGACY: Will be deleted
│   ├── RecordingsView/  # LEGACY: Will be deleted
│   ├── CommandSettings/ # LEGACY: Will be deleted
│   └── ...              # Other legacy components
│
├── pages/               # NEW: Page components
│   ├── Dashboard.tsx
│   ├── Recordings.tsx
│   ├── Commands.tsx
│   └── Settings.tsx
│
├── styles/              # NEW: Tailwind + CSS variables
│   ├── globals.css      # CSS custom properties (design tokens)
│   └── tailwind.css     # Tailwind directives
│
├── hooks/               # PRESERVED: All existing hooks
├── lib/                 # PRESERVED: Utilities
├── types/               # PRESERVED: TypeScript types
└── assets/              # PRESERVED: Images, videos
```

### Technology Stack Changes

| Layer | Current | New |
|-------|---------|-----|
| Styling | Custom CSS (BEM), ~2,400 lines | Tailwind CSS + CSS variables |
| Components | Custom HTML elements | Radix UI primitives |
| Animations | CSS animations | Framer Motion |
| Icons | None/inline SVG | Lucide React |
| State | React hooks | React hooks (preserved) |
| Navigation | Sidebar tabs | React Router (or equivalent) |

### Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        App.tsx                              │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    ToastProvider                     │   │
│  │  ┌───────────────────────────────────────────────┐  │   │
│  │  │                  AppShell                      │  │   │
│  │  │  ┌─────────────────────────────────────────┐  │  │   │
│  │  │  │ Header (StatusPill, CommandPalette)     │  │  │   │
│  │  │  ├─────────┬───────────────────────────────┤  │  │   │
│  │  │  │         │                               │  │  │   │
│  │  │  │ Sidebar │     MainContent               │  │  │   │
│  │  │  │  (Nav)  │   ┌─────────────────────┐     │  │  │   │
│  │  │  │         │   │   Page Component    │     │  │  │   │
│  │  │  │         │   │  (Dashboard, etc.)  │     │  │  │   │
│  │  │  │         │   └─────────────────────┘     │  │  │   │
│  │  │  │         ├───────────────────────────────┤  │  │   │
│  │  │  │         │ Footer (state, actions)       │  │  │   │
│  │  │  └─────────┴───────────────────────────────┘  │  │   │
│  │  └───────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

Existing hooks are preserved and integrated into new components:

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│   useRecording   │     │  useTranscription │     │   useListening   │
│                  │     │                   │     │                  │
│ • isRecording    │     │ • isTranscribing  │     │ • isListening    │
│ • duration       │     │ • result          │     │ • toggle()       │
│ • start/stop     │     │ • transcribe()    │     │                  │
└────────┬─────────┘     └────────┬──────────┘     └────────┬─────────┘
         │                        │                         │
         └────────────────────────┼─────────────────────────┘
                                  │
                                  ▼
                    ┌─────────────────────────┐
                    │       StatusPill        │
                    │  (derives state from    │
                    │   all hooks)            │
                    └─────────────────────────┘
```

### Routing Strategy

New UI uses page-based routing:

| Route | Page Component | Description |
|-------|----------------|-------------|
| `/` | Dashboard | Home with overview |
| `/recordings` | Recordings | Recording history |
| `/commands` | Commands | Voice command management |
| `/settings` | Settings | App configuration |
| `/settings/:tab` | Settings | Direct link to settings tab |

**Implementation Options:**
1. **React Router** - Full routing library (recommended if app grows)
2. **Simple state routing** - useState for current page (simpler, no extra deps)

For this redesign, recommend simple state routing to minimize dependencies, can upgrade later if needed.

### Integration Points

| Component | Integrates With | Notes |
|-----------|-----------------|-------|
| StatusPill | useRecording, useTranscription, useListening | Derives combined state |
| Dashboard | useListening, useRecordings, useCommands | Shows counts and recent |
| Recordings | useRecordings, useTranscription | List, play, transcribe |
| Commands | useCommands | CRUD operations |
| Settings | useSettings, useAudioDevices | All settings hooks |
| CommandPalette | All navigation, useRecording | Actions and navigation |
| Toasts | useTranscription (results), error handlers | Feedback display |

## Key Decisions

| Decision | Rationale | Date |
|----------|-----------|------|
| Tailwind CSS over styled-components | Faster development, smaller bundle, matches ui.md specs | 2025-12-17 |
| Radix UI for primitives | Accessible by default, unstyled (Tailwind-friendly), well-maintained | 2025-12-17 |
| Build alongside strategy | Lower risk, allows incremental review, easy rollback | 2025-12-17 |
| Dev toggle for preview | Enables comparison during development, removes need for feature branch | 2025-12-17 |
| Simple state routing | Avoids React Router dependency for MVP, can upgrade later | 2025-12-17 |
| Preserve all existing hooks | Hooks are well-tested, only UI layer needs replacement | 2025-12-17 |

## Investigation Log

| Date | Finding | Impact |
|------|---------|--------|
| 2025-12-17 | Current UI has ~2,400 lines of custom CSS across 15 files | Significant cleanup needed in final spec |
| 2025-12-17 | No Tailwind/Radix/Framer/Lucide installed yet | Need to install dependencies in first spec |
| 2025-12-17 | Existing hooks (useRecording, useSettings, etc.) are solid | Can reuse all hook logic, only replace UI |
| 2025-12-17 | Current sidebar has 4 tabs matching new 4 pages | Navigation concept similar, layout changes |

## Open Questions

- [x] Migration strategy (build alongside vs incremental) → Build alongside
- [x] Routing approach (React Router vs state) → Simple state routing for MVP
- [ ] Dark mode implementation timing - implement in design-system-foundation or separate spec?
- [ ] Storybook - add for component development or skip for speed?

## Dependencies to Install

```bash
# Styling
bun add tailwindcss postcss autoprefixer
bun add -D @tailwindcss/forms  # Optional: form styling

# UI Primitives (install as needed per spec)
bun add @radix-ui/react-select
bun add @radix-ui/react-switch
bun add @radix-ui/react-dialog
bun add @radix-ui/react-toast
bun add @radix-ui/react-tabs
bun add @radix-ui/react-slot

# Animations
bun add framer-motion

# Icons
bun add lucide-react

# Command Palette (optional - can build custom)
bun add cmdk
```

## Files to Modify

### New Files (by spec)

**design-system-foundation:**
- `src/styles/globals.css` - CSS custom properties
- `tailwind.config.js` - Theme configuration
- `postcss.config.js` - PostCSS setup

**base-ui-components:**
- `src/components/ui/Button.tsx`
- `src/components/ui/Card.tsx`
- `src/components/ui/Input.tsx`
- `src/components/ui/Select.tsx`
- `src/components/ui/Toggle.tsx`
- `src/components/ui/index.ts`

**layout-shell:**
- `src/components/layout/AppShell.tsx`
- `src/components/layout/Header.tsx`
- `src/components/layout/Sidebar.tsx`
- `src/components/layout/Footer.tsx`
- `src/components/layout/MainContent.tsx`

**Pages:**
- `src/pages/Dashboard.tsx`
- `src/pages/Recordings.tsx`
- `src/pages/Commands.tsx`
- `src/pages/Settings.tsx`

### Files to Delete (integration-and-cleanup)

See `integration-and-cleanup.spec.md` for full list (~15 CSS files, ~10 component directories).

## References

- `ui.md` - Complete design specifications (source of truth)
- [Tailwind CSS Docs](https://tailwindcss.com/docs)
- [Radix UI Docs](https://www.radix-ui.com/docs/primitives)
- [Framer Motion Docs](https://www.framer.com/motion/)
- [Lucide Icons](https://lucide.dev/icons/)
- [cmdk](https://cmdk.paco.me/) - Command palette library

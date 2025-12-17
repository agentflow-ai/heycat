---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - design-system-foundation
  - base-ui-components
---

# Spec: Toast Notifications

## Description

Build the notification/toast system for displaying transcription results, errors, and other feedback to users.

**Source of Truth:** `ui.md` - Part 3.7 (Notifications & Toasts)

## Acceptance Criteria

### Toast Container
- [ ] Fixed position: bottom-right of viewport
- [ ] Stacks multiple toasts vertically (newest on top)
- [ ] Z-index above content but below modals
- [ ] Max 3 visible toasts, older ones dismissed

### Toast Component (ui.md 3.7)
- [ ] Icon on left (color-coded by type)
- [ ] Title text (bold)
- [ ] Description text (optional, truncated if long)
- [ ] Close X button on right
- [ ] Action buttons (optional, e.g., "Copy to Clipboard")

### Toast Types
- [ ] **Success**: Green icon, success styling (transcription complete)
- [ ] **Error**: Red icon, error styling (device errors, failures)
- [ ] **Warning**: Amber icon, warning styling (caution states)
- [ ] **Info**: Blue icon, info styling (general information)

### Animations
- [ ] Slide-in from right on appear
- [ ] Slide-out to right on dismiss
- [ ] Smooth stacking animation when multiple

### Auto-dismiss
- [ ] Default: 5 seconds
- [ ] Configurable per toast
- [ ] Pause timer on hover
- [ ] No auto-dismiss for errors (require manual close)

### Transcription Toast (specific use case)
- [ ] Shows transcription result preview
- [ ] "Copy to Clipboard" action button
- [ ] Click to expand/view full text

## Test Cases

- [ ] Toast appears with slide-in animation
- [ ] Toast auto-dismisses after timeout
- [ ] Hover pauses auto-dismiss timer
- [ ] Close button dismisses immediately
- [ ] Multiple toasts stack correctly
- [ ] Action buttons work (e.g., copy to clipboard)
- [ ] Error toasts don't auto-dismiss

## Dependencies

- design-system-foundation (colors, animations)
- base-ui-components (Button, icons)

## Preconditions

- Design system completed
- React context or state management for toast queue

## Implementation Notes

**Files to create:**
```
src/components/overlays/
├── Toast.tsx
├── ToastContainer.tsx
├── ToastProvider.tsx       # Context provider
├── useToast.ts            # Hook to trigger toasts
└── toast.test.tsx
```

**Toast layout from ui.md 3.7:**
```
+------------------------------------------+
| [Check icon]  Transcription complete     |
|               "Hello, this is..."   [X]  |
|               [Copy to Clipboard]        |
+------------------------------------------+
```

**Consider using:**
- Radix Toast primitives (@radix-ui/react-toast)
- Or react-hot-toast / sonner for simpler implementation

**Toast context API:**
```ts
const { toast } = useToast();

toast({
  type: 'success',
  title: 'Transcription complete',
  description: 'Hello, this is...',
  action: {
    label: 'Copy to Clipboard',
    onClick: () => copyToClipboard(text)
  }
});
```

**Styling notes:**
- Width: 360px
- Border radius: var(--radius-lg)
- Shadow: var(--shadow-lg)
- Colored left border (4px) matching type

## Related Specs

- design-system-foundation, base-ui-components (dependencies)
- page-recordings, page-commands, page-settings (use toasts)

## Integration Points

- Production call site: Wrapped at app root in ToastProvider
- Connects to: useTranscription (transcription results), error handlers

## Integration Test

- Test location: `src/components/overlays/__tests__/Toast.test.tsx`
- Verification: [ ] Integration test passes

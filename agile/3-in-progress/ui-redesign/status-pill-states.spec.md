---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - design-system-foundation
  - base-ui-components
---

# Spec: Status Pill States

## Description

Create the status pill component that displays in the header, showing the current app state with appropriate colors and animations.

**Source of Truth:** `ui.md` - Part 2.2 (Header Bar), Part 3.4 (Status Indicators), Part 5.3 (State Transitions)

## Acceptance Criteria

### Status Pill Component
- [ ] Pill-shaped container with rounded ends
- [ ] Text label showing current state
- [ ] Icon appropriate to state (optional)
- [ ] Smooth transitions between states

### State Visualizations (ui.md 2.2, 5.3)
- [ ] **Idle**: Neutral gray background, "Ready" text
- [ ] **Listening**: Teal background with pulse/glow animation, "Listening..." text
- [ ] **Recording**: Red background with pulse animation, "Recording" text, duration timer
- [ ] **Processing**: Amber background with spinner, "Processing..." text

### Animations (ui.md 1.6, 3.4)
- [ ] Recording pulse: Red glow pulsing at 1.5s interval
- [ ] Listening glow: Soft teal ambient glow, breathing at 2s interval
- [ ] Processing: Rotating spinner or animated dots
- [ ] State transitions: Smooth color/size transitions (200ms)

### Integration
- [ ] Connects to app state (useRecording, useTranscription hooks)
- [ ] Updates in real-time as app state changes

## Test Cases

- [ ] Idle state renders gray pill with "Ready"
- [ ] Listening state shows teal with animation
- [ ] Recording state shows red with pulse and timer
- [ ] Processing state shows amber with spinner
- [ ] Transitions animate smoothly between states
- [ ] Component responds to hook state changes

## Dependencies

- design-system-foundation (uses state colors, animations)
- base-ui-components (may use StatusIndicator primitives)

## Preconditions

- Design system with state colors defined
- App state hooks available (useRecording, useTranscription, etc.)

## Implementation Notes

**Files to create:**
```
src/components/ui/
├── StatusPill.tsx
└── StatusPill.test.tsx
```

**State colors from ui.md:**
```css
--recording:    #EF4444    /* Red pulse */
--listening:    #5BB5B5    /* Teal glow */
--processing:   #F59E0B    /* Amber */
--idle:         #737373    /* Neutral gray */
```

**Animation keyframes:**
```css
@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.7; transform: scale(1.1); }
}

@keyframes breathe {
  0%, 100% { box-shadow: 0 0 10px var(--listening); }
  50% { box-shadow: 0 0 20px var(--listening); }
}
```

**State machine mapping:**
- Idle → status: 'idle'
- Listening (wake word active) → status: 'listening'
- Recording → status: 'recording'
- Transcribing → status: 'processing'

## Related Specs

- design-system-foundation, base-ui-components (dependencies)
- layout-shell (renders StatusPill in header)
- page-dashboard (uses status info)

## Integration Points

- Production call site: `src/components/layout/Header.tsx`
- Connects to: useRecording, useTranscription, useListening hooks

## Integration Test

- Test location: `src/components/ui/__tests__/StatusPill.test.tsx`
- Verification: [ ] Integration test passes

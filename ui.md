# HeyCat UI Redesign Instructions

## Overview

Complete UI redesign for HeyCat, a Tauri v2 desktop voice assistant app. These instructions provide everything needed to create a world-class, modern desktop interface.

---

## Part 1: Style Guide - "HeyCat Design System"

### 1.1 Brand Foundation

The design is derived from the HeyCat mascot - a friendly orange cat with teal accents and expressive purple eyes.

**Brand Personality:**
- Friendly & Approachable
- Warm & Inviting
- Professional but Playful
- Trustworthy & Reliable

### 1.2 Color Palette

#### Primary Colors (from mascot)
```
--heycat-orange:        #E8945A    /* Primary - warm orange from cat fur */
--heycat-orange-light:  #F4C89A    /* Lighter fur tones */
--heycat-cream:         #FDF6E8    /* Cream belly - backgrounds */
--heycat-teal:          #5BB5B5    /* Accent - from ears/tail tip */
--heycat-teal-dark:     #3D8B8B    /* Darker teal for hover states */
--heycat-purple:        #9B7BB5    /* From cat eyes - special accents */
```

#### Neutral Colors
```
--neutral-50:   #FAFAFA
--neutral-100:  #F5F5F5
--neutral-200:  #E5E5E5
--neutral-300:  #D4D4D4
--neutral-400:  #A3A3A3
--neutral-500:  #737373
--neutral-600:  #525252
--neutral-700:  #404040
--neutral-800:  #262626
--neutral-900:  #171717
```

#### Semantic Colors
```
--success:      #22C55E    /* Green - transcription complete, commands executed */
--warning:      #F59E0B    /* Amber - download in progress, caution states */
--error:        #EF4444    /* Red - errors, failed actions */
--info:         #3B82F6    /* Blue - informational states */
```

#### State Colors
```
--recording:    #EF4444    /* Red pulse for active recording */
--listening:    #5BB5B5    /* Teal glow for listening mode */
--processing:   #F59E0B    /* Amber for transcription in progress */
```

### 1.3 Typography

**Font Stack:**
```css
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
--font-mono: 'JetBrains Mono', 'SF Mono', 'Fira Code', monospace;
```

**Type Scale:**
```
--text-xs:    0.75rem / 1rem      /* 12px - labels, badges */
--text-sm:    0.875rem / 1.25rem  /* 14px - secondary text, sidebar */
--text-base:  1rem / 1.5rem       /* 16px - body text */
--text-lg:    1.125rem / 1.75rem  /* 18px - section headers */
--text-xl:    1.25rem / 1.75rem   /* 20px - page titles */
--text-2xl:   1.5rem / 2rem       /* 24px - main headers */
```

**Font Weights:**
```
--font-normal:    400
--font-medium:    500
--font-semibold:  600
--font-bold:      700
```

### 1.4 Spacing & Layout

**Spacing Scale (4px base):**
```
--space-1:  0.25rem   /* 4px */
--space-2:  0.5rem    /* 8px */
--space-3:  0.75rem   /* 12px */
--space-4:  1rem      /* 16px */
--space-5:  1.25rem   /* 20px */
--space-6:  1.5rem    /* 24px */
--space-8:  2rem      /* 32px */
--space-10: 2.5rem    /* 40px */
--space-12: 3rem      /* 48px */
```

**Border Radius (soft, friendly feel):**
```
--radius-sm:   0.375rem  /* 6px - buttons, inputs */
--radius-md:   0.5rem    /* 8px - cards */
--radius-lg:   0.75rem   /* 12px - modals, panels */
--radius-xl:   1rem      /* 16px - large containers */
--radius-full: 9999px    /* Pills, avatars */
```

### 1.5 Shadows & Elevation

```css
--shadow-sm:   0 1px 2px 0 rgb(0 0 0 / 0.05);
--shadow-md:   0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
--shadow-lg:   0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
--shadow-xl:   0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);
--shadow-glow: 0 0 20px var(--heycat-teal) / 0.3;  /* For listening state */
--shadow-window: 0 0 60px var(--heycat-orange) / 0.4;  /* Warm window glow */
```

**Window Glow Effect:**
The application window has a warm orange ambient glow around its edges, creating a friendly, inviting presence on the desktop. This is achieved with a large blur radius box-shadow using the brand orange color.

### 1.6 Motion & Animation

**Timing Functions:**
```css
--ease-default: cubic-bezier(0.4, 0, 0.2, 1);
--ease-in:      cubic-bezier(0.4, 0, 1, 1);
--ease-out:     cubic-bezier(0, 0, 0.2, 1);
--ease-bounce:  cubic-bezier(0.34, 1.56, 0.64, 1);
```

**Durations:**
```css
--duration-fast:   150ms
--duration-normal: 200ms
--duration-slow:   300ms
```

**Key Animations:**
- Recording pulse: Subtle red glow pulsing at 1.5s interval
- Listening glow: Soft teal ambient glow, breathing 2s interval
- Processing: Rotating spinner or progress wave
- Hover: Scale 1.02 with shadow elevation
- Press: Scale 0.98 with reduced shadow

---

## Part 2: Layout Architecture

### 2.1 Main Layout Structure

```
+------------------------------------------------------------------+
|  [Logo] HeyCat           [Status Pill]      [⌘K] [Settings] [?] |  <- Header (48px)
+------------------------------------------------------------------+
|         |                                                        |
|         |                                                        |
|  S      |                    MAIN CONTENT                        |
|  I      |                       AREA                             |
|  D      |                                                        |
|  E      |              (changes based on nav)                    |
|  B      |                                                        |
|  A      |                                                        |
|  R      |                                                        |
|         |                                                        |
| (220px) |                                                        |
|         +--------------------------------------------------------+
|         |  [Context Bar - shows current state, quick actions]    |  <- Footer (44px)
+---------+--------------------------------------------------------+
```

### 2.2 Header Bar (48px height)

**Left Section:**
- HeyCat logo (small cat icon + "HeyCat" text)
- Subtle branding, not dominant

**Center Section:**
- Status Pill showing current state:
  - `Idle` - neutral gray
  - `Listening...` - teal with pulse animation
  - `Recording` - red with pulse
  - `Processing` - amber with spinner

**Right Section:**
- Command Palette trigger (`⌘K` pill)
- Settings gear icon
- Help icon

### 2.3 Sidebar Navigation (220px width)

**Visual Style:**
- Light cream background (`--heycat-cream`)
- Subtle inner shadow on right edge
- Navigation items with rounded corners

**Navigation Items:**
```
[icon] Dashboard        <- Overview/home
[icon] Recordings       <- History view
[icon] Commands         <- Voice command management
[icon] Settings         <- App configuration
```

**Active State:**
- Full orange/cream background fill (warm highlight)
- Text remains dark for contrast
- No left border accent needed

**Section Dividers:**
- Thin line with label "FEATURES" etc.

### 2.4 Main Content Area

**Container:**
- Max-width: 900px centered
- Padding: 32px
- Clean white/cream background

**Page Structure:**
```
Page Title (text-2xl, semibold)
Page description (text-sm, neutral-500)

[Content Cards / Sections]
```

### 2.5 Context Footer Bar (44px)

Shows contextual information and quick actions:
- Left: Current state description ("Ready for your command." when idle)
- Center: Audio level mini-meter (when listening)
- Right: Quick action buttons (Start Recording, Stop, etc.)
- Decorative cat paw icon on the right side

---

## Part 3: Component Library

### 3.1 Buttons

**Primary Button:**
```css
background: linear-gradient(135deg, var(--heycat-orange), var(--heycat-orange-light));
color: white;
padding: 10px 20px;
border-radius: var(--radius-sm);
font-weight: var(--font-medium);
box-shadow: var(--shadow-sm);
transition: all var(--duration-fast) var(--ease-default);

&:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

&:active {
  transform: translateY(0);
}
```

**Secondary Button:**
- White background
- Orange border
- Orange text
- Hover: light orange background

**Ghost Button:**
- Transparent background
- Neutral text
- Hover: subtle gray background

**Danger Button:**
- Red background
- White text
- For destructive actions

### 3.2 Cards

**Standard Card:**
```css
background: white;
border-radius: var(--radius-lg);
padding: var(--space-5);
box-shadow: var(--shadow-sm);
border: 1px solid var(--neutral-200);
transition: all var(--duration-normal) var(--ease-default);

&:hover {
  box-shadow: var(--shadow-md);
  border-color: var(--neutral-300);
}
```

**Interactive Card (clickable):**
- Cursor pointer
- More pronounced hover elevation
- Optional orange left border on hover

**Status Card:**
- Colored left border indicating status
- Icon in matching color
- Used for recording items, commands, etc.

### 3.3 Inputs

**Text Input:**
```css
background: white;
border: 1px solid var(--neutral-300);
border-radius: var(--radius-sm);
padding: 10px 14px;
font-size: var(--text-base);

&:focus {
  outline: none;
  border-color: var(--heycat-teal);
  box-shadow: 0 0 0 3px var(--heycat-teal) / 0.1;
}

&::placeholder {
  color: var(--neutral-400);
}
```

**Select/Dropdown:**
- Same styling as text input
- Custom chevron icon
- Animated dropdown panel

**Toggle Switch:**
- Pill-shaped track
- Orange when active (matches brand primary)
- Smooth sliding animation

### 3.4 Status Indicators

**Recording Indicator:**
```css
/* Pulsing red dot */
.recording-dot {
  width: 12px;
  height: 12px;
  background: var(--recording);
  border-radius: 50%;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.7; transform: scale(1.1); }
}
```

**Listening Indicator:**
```css
/* Teal glow effect */
.listening-glow {
  box-shadow: 0 0 20px var(--heycat-teal) / 0.4;
  animation: breathe 2s ease-in-out infinite;
}
```

**Audio Level Meter:**
- Horizontal bar with gradient zones
- Green (safe) -> Yellow (optimal) -> Red (clipping)
- Smooth animated fill

### 3.5 Lists

**Recording List Item (collapsed):**
```
+------------------------------------------------------------------+
| [Play]  Recording_2024-01-15_143022.wav                          |
|         Sep 25, 2022 • 00:00:28 • 3.6 MB                         |
+------------------------------------------------------------------+
```

**Recording List Item (expanded):**
```
+------------------------------------------------------------------+
| [Play]  Recording_2024-01-15_143022.wav                          |
|         Sep 25, 2022 • 00:00:28 • 3.6 MB                         |
|------------------------------------------------------------------|
|  TRANSCRIPTION                                                    |
|  Hello, this is a test recording for the HeyCat application.     |
|  I'm testing the voice transcription feature.                    |
|                                                                   |
|  [Copy Text]  [Open File]  [Delete]                              |
+------------------------------------------------------------------+
```

**Command List Item:**
```
+------------------------------------------------------------------+
| [Toggle]  "open slack"                    [Open App]  [Edit] [X]  |
|           Opens Slack application                                 |
+------------------------------------------------------------------+
```

### 3.6 Modals & Dialogs

**Standard Modal:**
- Centered, max-width 480px
- Dark overlay (50% black)
- Rounded corners
- Header, body, footer sections
- Close X button top-right
- Smooth scale-in animation

**Command Palette (⌘K):**
```
+------------------------------------------+
|  [Search icon] Search commands...        |
+------------------------------------------+
|  > Start Recording          ⌘⇧R         |
|  > Stop Recording           Esc          |
|  > Open Settings            ⌘,           |
|  > Download Model                        |
|  > View Recordings                       |
+------------------------------------------+
```

- Top-center positioned (20% from top)
- Width: 560px
- Keyboard navigation (↑↓ Enter)
- Fuzzy search filtering
- Category groupings
- Recent items at top

### 3.7 Notifications & Toasts

**Toast Notification:**
```
+------------------------------------------+
| [Check icon]  Transcription complete     |
|               "Hello, this is..."   [X]  |
|               [Copy to Clipboard]        |
+------------------------------------------+
```

- Bottom-right positioned
- Auto-dismiss after 5s
- Stacking for multiple
- Slide-in from right animation
- Color-coded by type (success green, error red, etc.)

---

## Part 4: Screen Specifications

### 4.1 Dashboard (Home)

**Purpose:** At-a-glance overview and quick actions

**Layout:**
```
Dashboard
Welcome back! Here's your HeyCat status.

+------------------+  +------------------+  +------------------+
|  LISTENING       |  |  RECORDINGS      |  |  COMMANDS        |
|  [Toggle=====ON] |  |  12 recordings   |  |  8 active        |
|  'Hey Cat' ready.|  |                  |  |                  |
+------------------+  +------------------+  +------------------+

[  Start Recording  ]  [  Train Command  ]  [  Download Model  ]

RECENT ACTIVITY
+------------------------------------------------------------------+
| [Play] Recording_2024-01-15.wav     Sep 25, 2022   [Transcribed] |
| [Play] Recording_2024-01-14.wav     Aug 24, 2022      [Pending]  |
+------------------------------------------------------------------+
```

**Features:**
- Status cards with live state
- Quick action buttons (context-aware)
- Recent recordings list (last 5)
- Model download status if needed

### 4.2 Recordings

**Purpose:** Browse and manage all recordings

**Layout:**
```
Recordings
Manage your voice recordings and transcriptions.

[Search recordings...]  [Filter: All ▾]  [Sort: Newest ▾]

+------------------------------------------------------------------+
| [Play]  Recording_2024-01-15_143022.wav               [Transcript]|
|         January 15, 2024 at 2:30 PM • 3:42 • 2.3 MB      [More ▾]|
+------------------------------------------------------------------+
| [Play]  Recording_2024-01-14_091547.wav               [Transcribe]|
|         January 14, 2024 at 9:15 AM • 1:23 • 845 KB      [More ▾]|
+------------------------------------------------------------------+

[Load more...]
```

**Expanded Recording View (inline accordion):**
```
+------------------------------------------------------------------+
| [Play]  Recording_2024-01-15_143022.wav                          |
|         Sep 25, 2022 • 00:00:28 • 3.6 MB                         |
|------------------------------------------------------------------|
|  TRANSCRIPTION                                                    |
|  Hello, this is a test recording for the HeyCat application.     |
|  I'm testing the voice transcription feature.                    |
|                                                                   |
|  [Copy Text]  [Open File]  [Delete]                              |
+------------------------------------------------------------------+
```

**Empty State:**
- Friendly illustration
- "No recordings yet"
- "Press ⌘⇧R or say 'Hey Cat' to start"
- Primary button: "Start Recording"

### 4.3 Commands

**Purpose:** Create and manage voice commands

**Layout:**
```
Voice Commands
Create custom voice commands to control your Mac.

[+ New Command]                            [Search commands...]

YOUR COMMANDS (8)
+------------------------------------------------------------------+
| [ON ]  "open slack"                                    [Open App] |
|        Opens /Applications/Slack.app              [Edit] [Delete] |
+------------------------------------------------------------------+
| [ON ]  "type my email"                                [Type Text] |
|        Types: hello@example.com                   [Edit] [Delete] |
+------------------------------------------------------------------+
| [OFF]  "volume up"                               [System Control] |
|        Increases system volume by 10%             [Edit] [Delete] |
+------------------------------------------------------------------+
```

**New/Edit Command Modal:**
```
+------------------------------------------+
|  Create Voice Command                [X] |
|------------------------------------------|
|  Trigger Phrase                          |
|  [open spotify                      ]    |
|                                          |
|  Action Type                             |
|  [ Open Application            ▾]        |
|                                          |
|  Application                             |
|  [ Select application...       ▾]        |
|  OR  [Browse...]                         |
|                                          |
|  [Cancel]              [Save Command]    |
+------------------------------------------+
```

**Progressive Disclosure:**
- Basic: Trigger phrase + action type
- Advanced (revealed on click): Custom parameters, conditions

### 4.4 Settings

**Purpose:** Configure app behavior and preferences

**Layout (tabs within settings):**
```
Settings

[General] [Audio] [Transcription] [About]

GENERAL
+------------------------------------------------------------------+
|  Launch at Login                                          [ON ]  |
|  Start HeyCat when you log in to your Mac                        |
+------------------------------------------------------------------+
|  Auto-start Listening                                     [OFF]  |
|  Begin listening for wake word on app launch                     |
+------------------------------------------------------------------+
|  Notifications                                            [ON ]  |
|  Show notifications for transcription results                    |
+------------------------------------------------------------------+

KEYBOARD SHORTCUTS
+------------------------------------------------------------------+
|  Toggle Recording                              ⌘⇧R    [Change]   |
|  Cancel Recording                              Esc Esc           |
|  Open Command Palette                          ⌘K                |
+------------------------------------------------------------------+
```

**Audio Settings Tab:**
```
AUDIO INPUT
+------------------------------------------------------------------+
|  Input Device                                                     |
|  [ MacBook Pro Microphone           ▾]         [Refresh]         |
|                                                                   |
|  Audio Level  [=========--------------------]  Good               |
|               Test your microphone input                          |
+------------------------------------------------------------------+

WAKE WORD
+------------------------------------------------------------------+
|  Wake Phrase: "Hey Cat"                                          |
|  Sensitivity  [====●=====]  Medium                               |
+------------------------------------------------------------------+
```

**Transcription Settings Tab:**
```
TRANSCRIPTION MODEL
+------------------------------------------------------------------+
|  Batch Model (TDT)                                     [Ready ●] |
|  High-accuracy transcription for recordings                      |
|  Model size: 1.2 GB • Last updated: Jan 15, 2024                 |
|                                                                   |
|  [Check for Updates]                                             |
+------------------------------------------------------------------+

-- OR if not downloaded --

+------------------------------------------------------------------+
|  Batch Model (TDT)                              [Not Installed]  |
|  High-accuracy transcription for recordings                      |
|  Required for transcription features                             |
|                                                                   |
|  [Download Model (1.2 GB)]                                       |
|                                                                   |
|  Downloading... 45%                                              |
|  [============================---------------]  540 MB / 1.2 GB  |
+------------------------------------------------------------------+
```

---

## Part 5: Interaction Patterns

### 5.1 Command Palette (⌘K)

**Trigger:** ⌘K keyboard shortcut or click header icon

**Behavior:**
1. Overlay appears with dark backdrop
2. Focus immediately in search input
3. Show recent/frequent commands first
4. Type to fuzzy-filter results
5. Use ↑↓ to navigate, Enter to select
6. Escape to close

**Command Categories:**
- **Actions:** Start Recording, Stop, Transcribe
- **Navigation:** Go to Recordings, Settings, Commands
- **Settings:** Toggle Listening, Change Device
- **Help:** Documentation, Shortcuts, About

### 5.2 Progressive Disclosure

**Principle:** Show simple defaults, reveal complexity on demand

**Examples:**
1. **Command Editor:** Basic fields shown, "Advanced Options" expandable
2. **Recording Details:** Collapsed by default, click to expand
3. **Settings:** Grouped into tabs, advanced options at bottom
4. **Audio Devices:** Show recommended, "Show all devices" link

### 5.3 State Transitions

**Visual Feedback for States:**

```
IDLE → LISTENING
- Status pill: Gray → Teal with glow
- Footer: "Ready" → "Listening for 'Hey Cat'..."
- Subtle teal ambient glow on main area border

LISTENING → RECORDING
- Status pill: Teal → Red with pulse
- Footer: Shows recording duration timer
- Red accent border on content area

RECORDING → PROCESSING
- Status pill: Red → Amber with spinner
- Footer: "Transcribing your recording..."
- Disable recording button

PROCESSING → IDLE
- Toast notification with result
- Status returns to appropriate state
- Recording added to history
```

### 5.4 Error Handling

**Audio Device Errors:**
```
+------------------------------------------+
|  [!] Microphone Not Available        [X] |
|------------------------------------------|
|  HeyCat couldn't access your microphone. |
|  This might be because:                  |
|  • No microphone is connected            |
|  • Permission was denied                 |
|                                          |
|  [Open System Preferences]  [Try Again]  |
+------------------------------------------+
```

**Model Download Errors:**
- Inline error message in settings
- Retry button
- Link to manual download instructions

### 5.5 Keyboard Navigation

**Global Shortcuts:**
- `⌘K` - Command palette
- `⌘⇧R` - Toggle recording
- `Esc Esc` - Cancel recording
- `⌘,` - Open settings
- `⌘1-4` - Navigate to sidebar items

**Within Lists:**
- `↑↓` - Navigate items
- `Enter` - Select/expand
- `Delete` - Delete (with confirmation)
- `Space` - Toggle (for commands)

---

## Part 6: Dark Mode

### 6.1 Dark Mode Colors

```css
/* Dark mode overrides */
--background:           #1A1A1A
--surface:              #262626
--surface-elevated:     #303030
--text-primary:         #FAFAFA
--text-secondary:       #A3A3A3
--border:               #404040

/* Brand colors remain but adjusted */
--heycat-orange:        #F4A66A    /* Slightly lighter for dark bg */
--heycat-teal:          #6BC5C5    /* Slightly lighter */
```

### 6.2 Dark Mode Considerations

- Preserve warmth and friendliness
- Softer shadows (subtle elevation)
- Reduced contrast for comfortable viewing
- Status colors remain vibrant for visibility

---

## Part 7: Implementation Notes

### 7.1 Recommended Tech Stack

**Note:** Always use the latest stable version of each dependency.

- **Styling:** Tailwind CSS with custom theme
- **Components:** Radix UI primitives for accessibility
- **Animations:** Framer Motion for complex animations
- **Icons:** Lucide React (consistent, clean)
- **State:** React Context + Zustand for global state

### 7.2 File Structure

```
src/
├── components/
│   ├── ui/           # Base components (Button, Card, Input, etc.)
│   ├── layout/       # Header, Sidebar, Footer, PageContainer
│   ├── features/     # Feature-specific (RecordingsList, CommandEditor)
│   └── overlays/     # Modals, CommandPalette, Toasts
├── styles/
│   ├── globals.css   # CSS variables, base styles
│   └── tailwind.css  # Tailwind directives
├── pages/            # Main views (Dashboard, Recordings, Commands, Settings)
├── hooks/            # Custom hooks for Tauri integration
└── lib/              # Utilities, constants
```

### 7.3 Accessibility Requirements

- All interactive elements keyboard accessible
- ARIA labels on icons and status indicators
- Focus rings visible in both themes
- Color not sole indicator of state (use icons too)
- Screen reader announcements for state changes
- Minimum touch targets: 44x44px

### 7.4 Performance Considerations

- Virtualized lists for 100+ recordings
- Debounced search inputs
- Lazy load settings tabs
- Skeleton loaders for async content
- Optimistic UI updates

---

## Part 8: Features Reference

The UI must support these 28 features:

### Core Recording & Transcription
1. **Recording Control** - Start/stop/cancel recording (⌘⇧R, double-Esc)
2. **Transcription** - Auto-transcribe after recording, manual transcribe from history
3. **Recording History** - List, play, delete, transcribe historical recordings

### Listening & Wake Word
4. **Always-On Listening** - Toggle listening mode, wake word detection
5. **Wake Word Events** - Detect "Hey Cat", show confidence, auto-start recording

### Audio Management
6. **Device Selection** - List/select audio input devices, save preference
7. **Audio Level Monitoring** - Real-time level meter for device testing

### Model Management
8. **Model Download** - Download TDT model with progress, show status

### Voice Commands
9. **Command Management** - CRUD operations for voice commands
10. **Command Execution** - Match and execute spoken commands
11. **Disambiguation** - Handle multiple matches with selection UI

### Notifications & Feedback
12. **Transcription Notifications** - Toast with results and copy action
13. **Error Notifications** - Device errors, permission errors, download errors
14. **Recording Indicators** - Visual state feedback

### Settings & State
15. **Persistent Settings** - Save/load user preferences
16. **State Machine** - Idle → Listening → Recording → Processing states

---

## Summary

This redesign creates a warm, professional desktop app that:

1. **Leverages the HeyCat brand** through color palette and subtle mascot presence
2. **Prioritizes efficiency** with command palette and keyboard shortcuts
3. **Maintains simplicity** through progressive disclosure
4. **Provides clear feedback** for all states and actions
5. **Follows modern patterns** inspired by tools like Figma

The sidebar + main content layout provides clear navigation while the persistent header status ensures users always know what HeyCat is doing.

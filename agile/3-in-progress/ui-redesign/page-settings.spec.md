---
status: pending
created: 2025-12-17
completed: null
dependencies:
  - layout-shell
  - base-ui-components
  - toast-notifications
---

# Spec: Settings Page

## Description

Implement the Settings page with tabbed interface for General, Audio, Transcription, and About settings.

**Source of Truth:** `ui.md` - Part 4.4 (Settings)

## Acceptance Criteria

### Page Header
- [ ] Title: "Settings"
- [ ] No subtitle needed

### Tab Navigation
- [ ] Tabs: General, Audio, Transcription, About
- [ ] Active tab highlighted
- [ ] Content switches on tab click
- [ ] URL updates with tab (e.g., /settings/audio)

### General Tab (ui.md 4.4)
- [ ] **Launch at Login**: Toggle with description
- [ ] **Auto-start Listening**: Toggle with description
- [ ] **Notifications**: Toggle with description
- [ ] **Keyboard Shortcuts** section:
  - Toggle Recording: ⌘⇧R [Change] button
  - Cancel Recording: Esc Esc (display only)
  - Open Command Palette: ⌘K (display only)

### Audio Tab (ui.md 4.4)
- [ ] **Input Device** section:
  - Dropdown with available audio devices
  - Refresh button to rescan devices
  - Audio level meter showing live input
  - "Good" / "Low" / "High" indicator
- [ ] **Wake Word** section:
  - Display current wake phrase ("Hey Cat")
  - Sensitivity slider (Low - Medium - High)

### Transcription Tab (ui.md 4.4)
- [ ] **Model Status** card showing:
  - Model name (e.g., "Batch Model (TDT)")
  - Description
  - Status: Ready (green), Not Installed, Downloading
  - Model size, last updated date
- [ ] If installed: "Check for Updates" button
- [ ] If not installed: "Download Model" button
- [ ] If downloading: Progress bar with percentage, bytes downloaded

### About Tab
- [ ] App name and version
- [ ] Brief description
- [ ] Links: GitHub, Documentation, Report Issue
- [ ] Credits/acknowledgments

### Persistence
- [ ] All settings save to persistent storage
- [ ] Changes apply immediately
- [ ] Show toast on successful save

## Test Cases

- [ ] Tabs switch content correctly
- [ ] Toggle settings persist
- [ ] Audio device dropdown populates
- [ ] Audio level meter responds to input
- [ ] Model download button triggers download
- [ ] Progress bar updates during download
- [ ] Shortcut change modal works
- [ ] Settings persist across app restart

## Dependencies

- layout-shell (renders inside AppShell)
- base-ui-components (Card, Button, Input, Toggle, Select, Slider)
- toast-notifications (for save feedback)

## Preconditions

- Layout shell and toast system completed
- useSettings hook available
- useAudioDevices hook available
- Model download API available

## Implementation Notes

**Files to create:**
```
src/pages/
├── Settings.tsx
├── Settings.test.tsx
└── components/
    ├── GeneralSettings.tsx
    ├── AudioSettings.tsx
    ├── TranscriptionSettings.tsx
    ├── AboutSettings.tsx
    └── ShortcutEditor.tsx
```

**General settings layout from ui.md 4.4:**
```
GENERAL
+------------------------------------------------------------------+
|  Launch at Login                                          [ON ]  |
|  Start HeyCat when you log in to your Mac                        |
+------------------------------------------------------------------+
|  Auto-start Listening                                     [OFF]  |
|  Begin listening for wake word on app launch                     |
+------------------------------------------------------------------+
```

**Audio settings from ui.md 4.4:**
```
AUDIO INPUT
+------------------------------------------------------------------+
|  Input Device                                                     |
|  [ MacBook Pro Microphone           ▾]         [Refresh]         |
|                                                                   |
|  Audio Level  [=========--------------------]  Good               |
+------------------------------------------------------------------+
```

**Model download states:**
```
// Not installed
[Download Model (1.2 GB)]

// Downloading
Downloading... 45%
[============================---------------]  540 MB / 1.2 GB

// Installed
[Ready ●]  |  [Check for Updates]
```

**Reuse existing components:**
- AudioDeviceSelector from ListeningSettings
- AudioLevelMeter from ListeningSettings
- ModelDownloadCard from TranscriptionSettings

## Related Specs

- layout-shell, base-ui-components, toast-notifications (dependencies)
- command-palette (can navigate to settings)

## Integration Points

- Production call site: `src/App.tsx` routes to Settings
- Connects to: useSettings, useAudioDevices, useModelDownload hooks

## Integration Test

- Test location: `src/pages/__tests__/Settings.test.tsx`
- Verification: [ ] Integration test passes

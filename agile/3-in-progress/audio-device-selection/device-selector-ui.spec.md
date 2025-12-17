---
status: completed
created: 2025-12-15
completed: 2025-12-17
dependencies:
  - device-enumeration
  - device-settings-persistence
---

# Spec: Device Selector UI Component

## Description

Create the device selection UI component that displays available audio input devices in a dropdown and allows users to select their preferred microphone. The component integrates into the existing Listening Settings tab and persists the user's selection. This is the primary user-facing component for the audio device selection feature.

## Acceptance Criteria

- [ ] `useAudioDevices` hook fetches device list via `invoke('list_audio_devices')`
- [ ] `AudioDeviceSelector` component renders dropdown with all available devices
- [ ] Dropdown shows device names with "(Default)" indicator for system default device
- [ ] Current selection is highlighted/shown in dropdown
- [ ] Selecting a device updates settings via `useSettings` hook
- [ ] "System Default" option available to clear explicit selection
- [ ] Component shows loading state while fetching devices
- [ ] Component integrated into `ListeningSettings.tsx`
- [ ] Styling matches existing settings UI patterns
- [ ] Component exported from `ListeningSettings/index.ts`

## Test Cases

- [ ] `test_hook_fetches_devices` - useAudioDevices calls invoke on mount
- [ ] `test_hook_returns_loading_state` - Loading true while fetching
- [ ] `test_hook_returns_devices` - Returns device array after fetch
- [ ] `test_selector_renders_devices` - All devices appear in dropdown
- [ ] `test_selector_shows_current_selection` - Selected device is highlighted
- [ ] `test_selector_updates_settings` - Selection change calls updateSettings
- [ ] `test_selector_shows_default_indicator` - Default device shows "(Default)"
- [ ] `test_system_default_option` - "System Default" option clears selection

## Dependencies

- `device-enumeration` - Backend command to list devices
- `device-settings-persistence` - Settings hook and types

## Preconditions

- `list_audio_devices` Tauri command exists and works
- `useSettings` hook includes audio settings section
- `AudioInputDevice` TypeScript type defined
- Existing `ListeningSettings` component to integrate with

## Implementation Notes

**Files to create:**

1. **`src/hooks/useAudioDevices.ts`** - Device enumeration hook:
   ```typescript
   import { useState, useEffect } from 'react';
   import { invoke } from '@tauri-apps/api/core';
   import { AudioInputDevice } from '../types/audio';

   interface UseAudioDevicesResult {
     devices: AudioInputDevice[];
     isLoading: boolean;
     error: Error | null;
     refresh: () => void;
   }

   export function useAudioDevices(): UseAudioDevicesResult {
     const [devices, setDevices] = useState<AudioInputDevice[]>([]);
     const [isLoading, setIsLoading] = useState(true);
     const [error, setError] = useState<Error | null>(null);

     const fetchDevices = async () => {
       setIsLoading(true);
       setError(null);
       try {
         const result = await invoke<AudioInputDevice[]>('list_audio_devices');
         setDevices(result);
       } catch (e) {
         setError(e instanceof Error ? e : new Error(String(e)));
       } finally {
         setIsLoading(false);
       }
     };

     useEffect(() => {
       fetchDevices();
     }, []);

     return { devices, isLoading, error, refresh: fetchDevices };
   }
   ```

2. **`src/components/ListeningSettings/AudioDeviceSelector.tsx`** - Main component:
   ```typescript
   import React from 'react';
   import { useAudioDevices } from '../../hooks/useAudioDevices';
   import { useSettings } from '../../hooks/useSettings';
   import './AudioDeviceSelector.css';

   export function AudioDeviceSelector() {
     const { devices, isLoading, error, refresh } = useAudioDevices();
     const { settings, updateSettings } = useSettings();

     const selectedDevice = settings.audio.selectedDevice;

     const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
       const value = event.target.value;
       updateSettings({
         audio: {
           ...settings.audio,
           selectedDevice: value === '' ? null : value,
         },
       });
     };

     if (isLoading) {
       return <div className="audio-device-selector loading">Loading devices...</div>;
     }

     if (error) {
       return (
         <div className="audio-device-selector error">
           <span>Failed to load devices</span>
           <button onClick={refresh}>Retry</button>
         </div>
       );
     }

     return (
       <div className="audio-device-selector">
         <label htmlFor="audio-device-select">Microphone</label>
         <select
           id="audio-device-select"
           value={selectedDevice ?? ''}
           onChange={handleChange}
         >
           <option value="">System Default</option>
           {devices.map((device) => (
             <option key={device.name} value={device.name}>
               {device.name}
               {device.isDefault ? ' (Default)' : ''}
             </option>
           ))}
         </select>
       </div>
     );
   }
   ```

3. **`src/components/ListeningSettings/AudioDeviceSelector.css`** - Styling:
   ```css
   .audio-device-selector {
     display: flex;
     flex-direction: column;
     gap: 8px;
     margin-bottom: 16px;
   }

   .audio-device-selector label {
     font-weight: 500;
     font-size: 14px;
   }

   .audio-device-selector select {
     padding: 8px 12px;
     border-radius: 6px;
     border: 1px solid var(--border-color, #ccc);
     background: var(--input-bg, #fff);
     font-size: 14px;
     cursor: pointer;
   }

   .audio-device-selector select:focus {
     outline: none;
     border-color: var(--primary-color, #007bff);
   }

   .audio-device-selector.loading,
   .audio-device-selector.error {
     padding: 12px;
     background: var(--surface-color, #f5f5f5);
     border-radius: 6px;
   }

   .audio-device-selector.error {
     color: var(--error-color, #dc3545);
     display: flex;
     justify-content: space-between;
     align-items: center;
   }

   .audio-device-selector.error button {
     padding: 4px 12px;
     cursor: pointer;
   }
   ```

**Files to modify:**

4. **`src/components/ListeningSettings/ListeningSettings.tsx`** - Add selector:
   ```typescript
   import { AudioDeviceSelector } from './AudioDeviceSelector';

   export function ListeningSettings() {
     return (
       <div className="listening-settings">
         <h2>Listening Settings</h2>

         {/* Add device selector at the top or in appropriate section */}
         <section className="settings-section">
           <h3>Audio Input</h3>
           <AudioDeviceSelector />
         </section>

         {/* ... existing settings content */}
       </div>
     );
   }
   ```

5. **`src/components/ListeningSettings/index.ts`** - Export component:
   ```typescript
   export { ListeningSettings } from './ListeningSettings';
   export { AudioDeviceSelector } from './AudioDeviceSelector';
   ```

**UX Considerations:**
- "System Default" at top of dropdown (empty string value = null in settings)
- Default device marked with "(Default)" text for clarity
- Loading state prevents interaction during fetch
- Error state provides retry option
- Match existing settings UI styling exactly

**Key Behaviors:**
- On mount: fetch device list automatically
- On selection: immediately persist to settings
- No "Save" button needed - changes are instant

## Related Specs

- `device-enumeration.spec.md` - Backend device listing (dependency)
- `device-settings-persistence.spec.md` - Settings storage (dependency)
- `audio-level-meter.spec.md` - Will be added to this component later

## Integration Points

- Production call site: `src/components/ListeningSettings/ListeningSettings.tsx`
- Connects to: `useAudioDevices` hook, `useSettings` hook, Tauri invoke

## Integration Test

- Test location: `src/components/ListeningSettings/AudioDeviceSelector.test.tsx`
- Verification: [ ] Integration test passes

**Test Implementation:**
```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { AudioDeviceSelector } from './AudioDeviceSelector';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([
    { name: 'Built-in Microphone', isDefault: true },
    { name: 'USB Microphone', isDefault: false },
  ]),
}));

describe('AudioDeviceSelector', () => {
  it('renders device list after loading', async () => {
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(screen.getByText('Built-in Microphone (Default)')).toBeInTheDocument();
      expect(screen.getByText('USB Microphone')).toBeInTheDocument();
    });
  });

  it('shows System Default option', async () => {
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      expect(screen.getByText('System Default')).toBeInTheDocument();
    });
  });

  it('updates settings on selection', async () => {
    render(<AudioDeviceSelector />);

    await waitFor(() => {
      const select = screen.getByRole('combobox');
      fireEvent.change(select, { target: { value: 'USB Microphone' } });
    });

    // Verify updateSettings was called with correct value
  });
});
```

## Review

**Reviewed:** 2025-12-17
**Reviewer:** Claude

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `useAudioDevices` hook fetches device list via `invoke('list_audio_devices')` | PASS | src/hooks/useAudioDevices.ts:25 |
| `AudioDeviceSelector` component renders dropdown with all available devices | PASS | src/components/ListeningSettings/AudioDeviceSelector.tsx:54-59 |
| Dropdown shows device names with "(Default)" indicator for system default device | PASS | src/components/ListeningSettings/AudioDeviceSelector.tsx:57 |
| Current selection is highlighted/shown in dropdown | PASS | src/components/ListeningSettings/AudioDeviceSelector.tsx:50 |
| Selecting a device updates settings via `useSettings` hook | PASS | src/components/ListeningSettings/AudioDeviceSelector.tsx:11-14 |
| "System Default" option available to clear explicit selection | PASS | src/components/ListeningSettings/AudioDeviceSelector.tsx:53 |
| Component shows loading state while fetching devices | PASS | src/components/ListeningSettings/AudioDeviceSelector.tsx:16-22 |
| Component integrated into `ListeningSettings.tsx` | PASS | src/components/ListeningSettings/ListeningSettings.tsx:3,66 |
| Styling matches existing settings UI patterns | PASS | src/components/ListeningSettings/AudioDeviceSelector.css |
| Component exported from `ListeningSettings/index.ts` | PASS | src/components/ListeningSettings/index.ts:2 |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| test_hook_fetches_devices | PASS | AudioDeviceSelector.test.tsx:44 "renders device list after loading" |
| test_hook_returns_loading_state | PASS | AudioDeviceSelector.test.tsx:37 "shows loading state initially" |
| test_hook_returns_devices | PASS | AudioDeviceSelector.test.tsx:44 (covered in "renders device list") |
| test_selector_renders_devices | PASS | AudioDeviceSelector.test.tsx:44 "renders device list after loading" |
| test_selector_shows_current_selection | PASS | AudioDeviceSelector.test.tsx:82 "shows current selection from settings" |
| test_selector_updates_settings | PASS | AudioDeviceSelector.test.tsx:97 "updates settings when selection changes" |
| test_selector_shows_default_indicator | PASS | AudioDeviceSelector.test.tsx:72 "marks default device with (Default) indicator" |
| test_system_default_option | PASS | AudioDeviceSelector.test.tsx:60,114 (both "shows System Default option" and "clears selection when System Default is chosen") |

### Data Flow Verification

```
[UI Action] User selects device in dropdown
     |
     v
[Component] src/components/ListeningSettings/AudioDeviceSelector.tsx:11 handleChange()
     | calls updateAudioDevice()
     v
[Hook] src/hooks/useSettings.ts:135 updateAudioDevice()
     | store.set("audio.selectedDevice", deviceName)
     v
[Store] @tauri-apps/plugin-store persists to settings.json
     |
     v
[State Update] setSettings() at useSettings.ts:141
     |
     v
[UI Re-render] Dropdown reflects new selection
```

```
[Component Mount] AudioDeviceSelector renders
     |
     v
[Hook] src/hooks/useAudioDevices.ts:34 useEffect()
     | invoke("list_audio_devices")
     v
[Command] src-tauri/src/commands/mod.rs:585 list_audio_devices()
     | registered in lib.rs:240
     v
[Backend] src-tauri/src/audio/mod.rs list_input_devices()
     |
     v
[State Update] setDevices() at useAudioDevices.ts:26
     |
     v
[UI Re-render] Dropdown populated with devices
```

### Code Quality

**Strengths:**
- Clean separation of concerns: hook handles data fetching, component handles rendering
- Proper loading and error states with user-friendly retry option
- Dark mode support via CSS media query
- All tests pass (9/9)
- Good use of useCallback for stable function references
- Well-structured BEM-style CSS classes

**Concerns:**
- Minor React warning about act() wrapping in "shows loading state initially" test (does not affect functionality)

### Automated Check Results

```
Build warnings: warning about unused imports VAD_CHUNK_SIZE_16KHZ and VAD_CHUNK_SIZE_8KHZ (pre-existing, unrelated to this spec)
Command registration: list_audio_devices is properly registered (src-tauri/src/lib.rs:240)
Deferrals: None found in implementation files
```

### Verdict

**APPROVED** - All acceptance criteria are met, tests pass, data flow is complete end-to-end, and the component is properly integrated into the ListeningSettings UI. The implementation follows the spec closely with minor improvements (BEM CSS naming, useCallback optimization).

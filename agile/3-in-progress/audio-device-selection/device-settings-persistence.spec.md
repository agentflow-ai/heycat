---
status: in-progress
created: 2025-12-15
completed: null
dependencies: []
---

# Spec: Device Settings Persistence

## Description

Extend the frontend settings infrastructure to store the user's preferred audio input device. The device preference persists across app restarts using the existing Tauri store plugin. This spec creates the TypeScript types and hooks needed for device selection without any UI components.

## Acceptance Criteria

- [ ] `AudioInputDevice` TypeScript type defined with `name: string` and `isDefault: boolean`
- [ ] `AudioSettings` type defined with `selectedDevice: string | null` field
- [ ] `useSettings` hook extended to include `audio` section in settings schema
- [ ] `getAudioDevice()` function returns stored device name or null
- [ ] `setAudioDevice(deviceName: string | null)` function persists selection
- [ ] Settings default to `null` (use system default) on fresh install
- [ ] Device selection persists after app restart (verified in test)
- [ ] TypeScript types exported from `src/types/audio.ts`

## Test Cases

- [ ] `test_audio_settings_default_null` - Fresh settings return null for device
- [ ] `test_set_audio_device_persists` - Setting device saves to store
- [ ] `test_get_audio_device_retrieves` - Get returns previously set device
- [ ] `test_clear_audio_device` - Setting null clears selection
- [ ] `test_settings_survive_reload` - Device persists after simulated reload

## Dependencies

None - this is independent of backend specs and can run in parallel.

## Preconditions

- Existing `useSettings` hook in `src/hooks/useSettings.ts`
- Tauri store plugin configured and working
- Existing settings pattern to follow

## Implementation Notes

**Files to create:**

1. **`src/types/audio.ts`** - TypeScript types:
   ```typescript
   /**
    * Represents an audio input device returned from the backend
    */
   export interface AudioInputDevice {
     name: string;
     isDefault: boolean;
   }

   /**
    * Audio-related settings stored in the frontend
    */
   export interface AudioSettings {
     /** Name of the selected device, or null for system default */
     selectedDevice: string | null;
   }

   /**
    * Default audio settings for fresh installs
    */
   export const DEFAULT_AUDIO_SETTINGS: AudioSettings = {
     selectedDevice: null,
   };
   ```

**Files to modify:**

2. **`src/hooks/useSettings.ts`** - Extend settings schema:
   ```typescript
   import { AudioSettings, DEFAULT_AUDIO_SETTINGS } from '../types/audio';

   // Add to Settings interface
   interface Settings {
     // ... existing fields
     audio: AudioSettings;
   }

   // Add to defaults
   const DEFAULT_SETTINGS: Settings = {
     // ... existing defaults
     audio: DEFAULT_AUDIO_SETTINGS,
   };

   // The existing useSettings hook should already handle nested objects
   // via the Tauri store plugin - verify this works correctly
   ```

3. **`src/types/index.ts`** (if exists) - Re-export audio types:
   ```typescript
   export * from './audio';
   ```

**Usage Pattern:**
```typescript
// In a component or hook
const { settings, updateSettings } = useSettings();

// Read current device
const currentDevice = settings.audio.selectedDevice;

// Update device selection
const selectDevice = (deviceName: string | null) => {
  updateSettings({
    audio: {
      ...settings.audio,
      selectedDevice: deviceName,
    },
  });
};
```

**Key Considerations:**
- Store device by name (string), not by index or ID (devices can be reordered)
- `null` means "use system default" - important distinction from "no device"
- Follow existing `useSettings` patterns exactly for consistency
- Ensure the store plugin auto-persists changes (verify existing behavior)

**Verification Steps:**
1. Set device in settings
2. Check store file contains audio section
3. Restart app
4. Verify device selection is restored

## Related Specs

- `device-enumeration.spec.md` - Types should match backend struct
- `device-selector-ui.spec.md` - UI component consumes these settings

## Integration Points

- Production call site: `src/components/ListeningSettings/AudioDeviceSelector.tsx` (future)
- Connects to: `useSettings` hook, Tauri store plugin

## Integration Test

- Test location: `src/hooks/useSettings.test.ts` (extend existing tests)
- Verification: [ ] Integration test passes

**Test Implementation:**
```typescript
describe('audio settings', () => {
  it('defaults to null device selection', async () => {
    const { result } = renderHook(() => useSettings());
    expect(result.current.settings.audio.selectedDevice).toBeNull();
  });

  it('persists device selection', async () => {
    const { result } = renderHook(() => useSettings());

    act(() => {
      result.current.updateSettings({
        audio: { selectedDevice: 'USB Microphone' },
      });
    });

    // Re-render hook to simulate reload
    const { result: reloaded } = renderHook(() => useSettings());
    expect(reloaded.current.settings.audio.selectedDevice).toBe('USB Microphone');
  });
});
```

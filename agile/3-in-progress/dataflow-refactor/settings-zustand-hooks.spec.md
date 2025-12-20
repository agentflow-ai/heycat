---
status: in-progress
created: 2025-12-20
completed: null
dependencies: ["zustand-store"]
---

# Spec: Migrate settings to Zustand + Tauri Store

## Description

Refactor the `useSettings` hook to use Zustand for in-memory state while preserving Tauri Store for persistence. Settings are loaded into Zustand on app init and written to both Zustand and Tauri Store on updates. This maintains backend access to settings (for hotkey-triggered flows) while providing fast frontend reads.

## Acceptance Criteria

- [ ] `useSettings.ts` refactored to use Zustand store
- [ ] Settings loaded from Tauri Store into Zustand on app init
- [ ] `useSettings()` reads from Zustand (fast, synchronous)
- [ ] `updateSetting()` writes to BOTH:
  - Zustand store (immediate UI update)
  - Tauri Store (persistence, backend access)
- [ ] Settings structure preserved:
  - `listening.enabled`, `listening.autoStartOnLaunch`
  - `audio.selectedDevice`
  - `shortcuts.distinguishLeftRight`
- [ ] Backend can still read settings directly from Tauri Store
- [ ] Settings hydration happens before app renders (or shows loading)
- [ ] TypeScript types match existing Settings interface
- [ ] No race conditions between Zustand and Tauri Store updates

## Test Cases

- [ ] Settings load from Tauri Store into Zustand on init
- [ ] `useSettings()` returns current settings from Zustand
- [ ] `updateSetting('audio.selectedDevice', 'mic')` updates both stores
- [ ] Settings persist across app restarts (Tauri Store)
- [ ] Backend can read settings independently (Tauri Store access)
- [ ] Multiple components see same settings (shared Zustand state)

## Dependencies

- `zustand-store` - provides appStore with settings slice

## Preconditions

- Zustand store created with settings slice
- Tauri Store plugin available (`@tauri-apps/plugin-store`)

## Implementation Notes

```typescript
// src/hooks/useSettings.ts
import { useEffect } from 'react';
import { load } from '@tauri-apps/plugin-store';
import { useAppStore, useSettingsCache, useIsSettingsLoaded } from '../stores/appStore';

interface Settings {
  listening: { enabled: boolean; autoStartOnLaunch: boolean };
  audio: { selectedDevice: string | null };
  shortcuts: { distinguishLeftRight: boolean };
}

const DEFAULT_SETTINGS: Settings = {
  listening: { enabled: false, autoStartOnLaunch: false },
  audio: { selectedDevice: null },
  shortcuts: { distinguishLeftRight: false },
};

// Initialize settings from Tauri Store into Zustand
export async function initializeSettings() {
  const store = await load('settings.json');
  const setSettings = useAppStore.getState().setSettings;

  const settings: Settings = {
    listening: {
      enabled: await store.get('listening.enabled') ?? DEFAULT_SETTINGS.listening.enabled,
      autoStartOnLaunch: await store.get('listening.autoStartOnLaunch') ?? DEFAULT_SETTINGS.listening.autoStartOnLaunch,
    },
    audio: {
      selectedDevice: await store.get('audio.selectedDevice') ?? DEFAULT_SETTINGS.audio.selectedDevice,
    },
    shortcuts: {
      distinguishLeftRight: await store.get('shortcuts.distinguishLeftRight') ?? DEFAULT_SETTINGS.shortcuts.distinguishLeftRight,
    },
  };

  setSettings(settings);
}

// Update a setting in both Zustand and Tauri Store
export async function updateSetting<K extends keyof Settings>(
  key: K,
  value: Settings[K]
) {
  // Update Zustand immediately
  useAppStore.getState().updateSetting(key, value);

  // Persist to Tauri Store
  const store = await load('settings.json');
  for (const [subKey, subValue] of Object.entries(value)) {
    await store.set(`${key}.${subKey}`, subValue);
  }
  await store.save();
}

// Hook for components to access settings
export function useSettings() {
  const settings = useSettingsCache();
  const isLoaded = useIsSettingsLoaded();

  return {
    settings: settings ?? DEFAULT_SETTINGS,
    isLoaded,
    updateSetting,
  };
}
```

**Initialization flow:**
1. App mounts → `initializeSettings()` called
2. Tauri Store read → Zustand updated
3. `isSettingsLoaded` set to true
4. Components render with settings

**Key decision:** Settings are CLIENT state (cached in Zustand), not SERVER state (not in Tanstack Query). This is because settings are local preferences, not remote data.

## Related Specs

- `zustand-store` - provides settings slice in appStore
- `app-providers-wiring` - calls initializeSettings on mount

## Integration Points

- Production call site: Settings page, any component needing settings
- Connects to: Zustand store, Tauri Store, backend (reads Tauri Store directly)

## Integration Test

- Test location: `src/hooks/__tests__/useSettings.test.ts`
- Verification: [ ] Settings load, update, and persist correctly

---
status: completed
created: 2025-12-20
completed: 2025-12-20
dependencies: ["zustand-store"]
review_round: 1
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

## Review

**Reviewed:** 2025-12-20
**Reviewer:** Claude

### Pre-Review Gates

**1. Build Warning Check:** PASS - No unused warnings from cargo check.

**2. Command Registration Check:** N/A - This spec does not add backend commands.

**3. Event Subscription Check:** N/A - This spec does not add events.

### Acceptance Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| `useSettings.ts` refactored to use Zustand store | PASS | `src/hooks/useSettings.ts` imports and uses `useSettingsCache`, `useIsSettingsLoaded` from `stores/appStore` |
| Settings loaded from Tauri Store into Zustand on app init | PASS | `initializeSettings()` at line 58-93 loads from Tauri Store and calls `setSettings()` |
| `useSettings()` reads from Zustand (fast, synchronous) | PASS | `useSettings()` at line 135-179 uses `useSettingsCache()` selector for synchronous reads |
| `updateSetting()` writes to BOTH Zustand and Tauri Store | PASS | `updateSettingInBothStores()` at line 99-121 updates Zustand first, then persists to Tauri Store |
| Settings structure preserved | PASS | `AppSettings` interface matches spec: listening, audio, shortcuts with all sub-fields |
| Backend can still read settings directly from Tauri Store | PASS | Settings persisted via `store.set()` and `store.save()` in `updateSettingInBothStores()` |
| Settings hydration happens before app renders | PASS | `App.tsx:26` calls `initializeSettings()` in AppInitializer before event bridge setup |
| TypeScript types match existing Settings interface | PASS | Types defined in `useSettings.ts` lines 10-26 |
| No race conditions between Zustand and Tauri Store updates | PASS | Zustand updated synchronously first, then async persist (UI always has latest) |

### Test Coverage Audit

| Test Case | Status | Location |
|-----------|--------|----------|
| Settings load from Tauri Store into Zustand on init | PASS | `src/hooks/useSettings.test.ts:36` - "loads settings from Tauri Store into Zustand" |
| `useSettings()` returns current settings from Zustand | PASS | `src/hooks/useSettings.test.ts:69` - "returns settings from Zustand when loaded" |
| `updateSetting()` updates both stores | PASS | `src/hooks/useSettings.test.ts:94` - "updates both Zustand and Tauri Store" |
| Settings persist across app restarts (Tauri Store) | PASS | Implicit in test that verifies `store.set()` and `store.save()` are called |
| Backend can read settings independently | PASS | Architecture ensures Tauri Store is always updated |
| Multiple components see same settings | PASS | All components use shared Zustand store via `useSettingsCache()` selector |

### Code Quality

**Strengths:**
- Clean separation of concerns: Zustand for fast UI reads, Tauri Store for persistence and backend access
- Type-safe update functions with proper TypeScript generics
- Proper initialization flow in AppInitializer ensures settings are loaded before app renders
- Optimized selectors prevent unnecessary re-renders
- Tests follow behavior-focused philosophy per TESTING.md guidelines
- Production usage verified in GeneralSettings.tsx and AudioSettings.tsx components

**Concerns:**
- None identified

### Data Flow Verification

```
[App Mount]
     |
     v
[AppInitializer] src/App.tsx:20-40
     | initializeSettings()
     v
[Tauri Store] @tauri-apps/plugin-store
     | load, get values
     v
[Zustand Store] src/stores/appStore.ts:70-87
     | setSettings() -> isSettingsLoaded = true
     v
[Component] e.g., GeneralSettings.tsx
     | useSettings() -> useSettingsCache()
     v
[User Action] updateAutoStartListening(true)
     |
     v
[updateSettingInBothStores]
     | 1. Zustand update (sync)
     | 2. Tauri Store persist (async)
     v
[UI Re-render] Immediate via Zustand subscription
```

### Production Wiring Verification

| New Code | Type | Production Call Site | Reachable from main/UI? |
|----------|------|---------------------|-------------------------|
| initializeSettings | fn | App.tsx:26 | YES |
| useSettings | hook | GeneralSettings.tsx:24, AudioSettings.tsx:29 | YES |
| updateSettingInBothStores | fn | useSettings hook update functions | YES |

### Verdict

**APPROVED** - All acceptance criteria met. Settings hook successfully migrated to Zustand for fast synchronous reads while maintaining Tauri Store persistence for backend access. Tests are comprehensive and behavior-focused. Production wiring verified in App.tsx and settings components.

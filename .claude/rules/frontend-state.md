---
paths: "src/**/*.ts, src/**/*.tsx"
---

# Frontend State Management

## State Layer Separation

| Layer | Storage | Purpose | Examples |
|-------|---------|---------|----------|
| **Client State** | Zustand (`appStore.ts`) | UI state, settings cache | overlayMode, transcription status |
| **Server State** | Tanstack Query | Cached backend data | recordings, model status |
| **Persistent** | Tauri Store (`settings.json`) | Settings backend needs | audio.selectedDevice |

## Zustand for Client State Only

```typescript
// appStore.ts
/**
 * IMPORTANT: This store holds CLIENT state only. Server state (recordings,
 * recording state, model status) belongs in Tanstack Query, not here.
 */
export interface AppState {
  // Client state only - NO server state here
  overlayMode: string | null;
  settingsCache: AppSettings | null;
  isSettingsLoaded: boolean;
  transcription: TranscriptionState;

  // Actions
  setOverlayMode: (mode: string | null) => void;
  setSettings: (settings: AppSettings) => void;
  // ...
}
```

Use optimized selectors to prevent unnecessary re-renders:

```typescript
// Components using these only re-render when their slice changes
export const useOverlayMode = () => useAppStore((s) => s.overlayMode);
export const useSettingsCache = () => useAppStore((s) => s.settingsCache);
export const useTranscriptionState = () => useAppStore((s) => s.transcription);
```

## Tanstack Query for Server State

```typescript
// Server state via query hooks
const { isRecording, isLoading, error } = useRecordingState();
const { devices } = useAudioDevices();
const { entries } = useDictionaryList();
```

Query cache is invalidated via Event Bridge when backend emits events.

## Dual-Write Settings Pattern

Settings require both:
- **Zustand**: Fast synchronous reads for UI
- **Tauri Store**: Persistence + backend access

```typescript
/**
 * Update a specific setting in both Zustand (immediate) and Tauri Store (persistence).
 * The dual-write ensures UI updates instantly while settings persist for backend access.
 */
async function updateSettingInBothStores<K extends keyof AppSettings>(
  key: K,
  nestedKey: keyof AppSettings[K],
  value: AppSettings[K][keyof AppSettings[K]]
): Promise<void> {
  // Get current settings from Zustand
  const currentSettings = useAppStore.getState().settingsCache;
  if (!currentSettings) return;

  // Update Zustand immediately for fast UI response
  const updatedCategory = {
    ...currentSettings[key],
    [nestedKey]: value,
  };
  useAppStore.getState().updateSetting(key, updatedCategory);

  // Persist to Tauri Store for backend access and restart persistence
  const store = await load(settingsFile);
  await store.set(`${key}.${String(nestedKey)}`, value);
  await store.save();
}
```

## Settings Hook Usage

```typescript
export function useSettings(): UseSettingsReturn {
  const settingsCache = useSettingsCache();
  const isSettingsLoaded = useIsSettingsLoaded();

  const settings = settingsCache ?? DEFAULT_SETTINGS;
  const isLoading = !isSettingsLoaded;

  const updateAudioDevice = async (deviceName: string | null) => {
    await updateSettingInBothStores("audio", "selectedDevice", deviceName);
  };

  return {
    settings,
    isLoading,
    updateAudioDevice,
    // ...
  };
}
```

## Anti-Patterns

### Server data in Zustand

```typescript
// BAD: Recording state is server state, belongs in Tanstack Query
const useAppStore = create((set) => ({
  isRecording: false,
  recordings: [],
  setIsRecording: (value) => set({ isRecording: value }),
}));

// GOOD: Use Tanstack Query for server state
const { isRecording } = useRecordingState();  // Query hook
const { recordings } = useRecordings();        // Query hook
```

### Direct useState for global state

```typescript
// BAD: Local state for what should be global
function Settings() {
  const [overlayMode, setOverlayMode] = useState(null);
}

// GOOD: Use Zustand selector
function Settings() {
  const overlayMode = useOverlayMode();
  const setOverlayMode = useAppStore((s) => s.setOverlayMode);
}
```

### Writing settings to only one store

```typescript
// BAD: Missing Tauri Store persistence
const updateDevice = (device) => {
  useAppStore.getState().updateSetting("audio", { selectedDevice: device });
  // Forgot to persist to Tauri Store!
};

// GOOD: Dual-write pattern
const updateDevice = async (device) => {
  await updateSettingInBothStores("audio", "selectedDevice", device);
};
```

### Not using selectors

```typescript
// BAD: Component re-renders on ANY store change
function MyComponent() {
  const { overlayMode } = useAppStore();  // Subscribes to entire store
}

// GOOD: Component only re-renders when overlayMode changes
function MyComponent() {
  const overlayMode = useOverlayMode();  // Subscribes to slice only
}
```

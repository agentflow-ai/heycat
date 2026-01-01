import { load } from "@tauri-apps/plugin-store";
import { AudioSettings, DEFAULT_AUDIO_SETTINGS } from "../types/audio";
import {
  useAppStore,
  useSettingsCache,
  useIsSettingsLoaded,
} from "../stores/appStore";
import { getSettingsFile } from "../lib/settingsFile";

/** Recording mode determines how the hotkey triggers recording */
export type RecordingMode = "toggle" | "push-to-talk";

/** Settings related to keyboard shortcuts */
export interface ShortcutSettings {
  distinguishLeftRight: boolean;
  recordingMode: RecordingMode;
}

/** All application settings */
export interface AppSettings {
  audio: AudioSettings;
  shortcuts: ShortcutSettings;
}

/** Default settings for fresh installations */
export const DEFAULT_SETTINGS: AppSettings = {
  audio: DEFAULT_AUDIO_SETTINGS,
  shortcuts: {
    distinguishLeftRight: false,
    recordingMode: "toggle",
  },
};

/** Return type of the useSettings hook */
export interface UseSettingsReturn {
  settings: AppSettings;
  isLoading: boolean;
  updateAudioDevice: (deviceName: string | null) => Promise<void>;
  updateDistinguishLeftRight: (enabled: boolean) => Promise<void>;
  updateRecordingMode: (mode: RecordingMode) => Promise<void>;
}

/**
 * Initialize settings from Tauri Store into Zustand on app startup.
 * Called once from AppInitializer before the app renders.
 *
 * This loads all settings from persistent storage into the Zustand store,
 * allowing synchronous access throughout the app.
 */
export async function initializeSettings(): Promise<void> {
  /* v8 ignore start -- @preserve */
  const settingsFile = await getSettingsFile();
  const store = await load(settingsFile);
  const setSettings = useAppStore.getState().setSettings;

  // Load existing settings or use defaults
  const audioSelectedDevice = await store.get<string | null>(
    "audio.selectedDevice"
  );
  const distinguishLeftRight = await store.get<boolean>(
    "shortcuts.distinguishLeftRight"
  );
  const recordingMode = await store.get<RecordingMode>(
    "shortcuts.recordingMode"
  );

  const settings: AppSettings = {
    audio: {
      selectedDevice:
        audioSelectedDevice ?? DEFAULT_SETTINGS.audio.selectedDevice,
    },
    shortcuts: {
      distinguishLeftRight:
        distinguishLeftRight ?? DEFAULT_SETTINGS.shortcuts.distinguishLeftRight,
      recordingMode:
        recordingMode ?? DEFAULT_SETTINGS.shortcuts.recordingMode,
    },
  };

  setSettings(settings);
  /* v8 ignore stop */
}

/**
 * Update a specific setting in both Zustand (immediate) and Tauri Store (persistence).
 * The dual-write ensures UI updates instantly while settings persist for backend access.
 */
async function updateSettingInBothStores<K extends keyof AppSettings>(
  key: K,
  nestedKey: keyof AppSettings[K],
  value: AppSettings[K][keyof AppSettings[K]]
): Promise<void> {
  /* v8 ignore start -- @preserve */
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
  const settingsFile = await getSettingsFile();
  const store = await load(settingsFile);
  await store.set(`${key}.${String(nestedKey)}`, value);
  await store.save();
  /* v8 ignore stop */
}

/**
 * Custom hook for accessing and updating application settings.
 *
 * Settings are stored in:
 * - Zustand (in-memory): For fast synchronous reads from React components
 * - Tauri Store (persistent): For backend access and restart persistence
 *
 * The hook reads from Zustand and writes to both stores, ensuring:
 * - Immediate UI updates (Zustand)
 * - Persistence across restarts (Tauri Store)
 * - Backend can read settings directly from Tauri Store
 */
export function useSettings(): UseSettingsReturn {
  const settingsCache = useSettingsCache();
  const isSettingsLoaded = useIsSettingsLoaded();

  // Use cached settings or defaults if not yet loaded
  const settings = settingsCache ?? DEFAULT_SETTINGS;
  const isLoading = !isSettingsLoaded;

  const updateAudioDevice = async (
    deviceName: string | null
  ): Promise<void> => {
    await updateSettingInBothStores("audio", "selectedDevice", deviceName);
  };

  const updateDistinguishLeftRight = async (
    enabled: boolean
  ): Promise<void> => {
    await updateSettingInBothStores(
      "shortcuts",
      "distinguishLeftRight",
      enabled
    );
  };

  const updateRecordingMode = async (mode: RecordingMode): Promise<void> => {
    await updateSettingInBothStores("shortcuts", "recordingMode", mode);
  };

  return {
    settings,
    isLoading,
    updateAudioDevice,
    updateDistinguishLeftRight,
    updateRecordingMode,
  };
}

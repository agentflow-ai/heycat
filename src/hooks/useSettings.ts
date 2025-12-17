import { useState, useEffect, useCallback } from "react";
import { load, Store } from "@tauri-apps/plugin-store";
import { AudioSettings, DEFAULT_AUDIO_SETTINGS } from "../types/audio";

/** Settings related to listening mode */
export interface ListeningSettings {
  enabled: boolean;
  autoStartOnLaunch: boolean;
}

/** All application settings */
export interface AppSettings {
  listening: ListeningSettings;
  audio: AudioSettings;
}

/** Default settings for fresh installations */
const DEFAULT_SETTINGS: AppSettings = {
  listening: {
    enabled: false,
    autoStartOnLaunch: false,
  },
  audio: DEFAULT_AUDIO_SETTINGS,
};

/** Return type of the useSettings hook */
export interface UseSettingsReturn {
  settings: AppSettings;
  isLoading: boolean;
  error: string | null;
  updateListeningEnabled: (enabled: boolean) => Promise<void>;
  updateAutoStartListening: (enabled: boolean) => Promise<void>;
  updateAudioDevice: (deviceName: string | null) => Promise<void>;
}

const STORE_FILE = "settings.json";

/**
 * Custom hook for managing persistent application settings
 * Uses Tauri's store plugin to persist settings across sessions
 */
export function useSettings(): UseSettingsReturn {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [store, setStore] = useState<Store | null>(null);

  // Initialize store and load settings on mount
  useEffect(() => {
    let mounted = true;

    /* v8 ignore start -- @preserve */
    const initStore = async () => {
      try {
        const storeInstance = await load(STORE_FILE, { autoSave: true });
        if (!mounted) return;
        setStore(storeInstance);

        // Load existing settings or use defaults
        const listeningEnabled = await storeInstance.get<boolean>(
          "listening.enabled"
        );
        const autoStartOnLaunch = await storeInstance.get<boolean>(
          "listening.autoStartOnLaunch"
        );
        const audioSelectedDevice = await storeInstance.get<string | null>(
          "audio.selectedDevice"
        );

        setSettings({
          listening: {
            enabled: listeningEnabled ?? DEFAULT_SETTINGS.listening.enabled,
            autoStartOnLaunch:
              autoStartOnLaunch ?? DEFAULT_SETTINGS.listening.autoStartOnLaunch,
          },
          audio: {
            selectedDevice:
              audioSelectedDevice ?? DEFAULT_SETTINGS.audio.selectedDevice,
          },
        });
        setIsLoading(false);
      } catch (e) {
        if (!mounted) return;
        setError(e instanceof Error ? e.message : String(e));
        setIsLoading(false);
      }
    };

    initStore();
    /* v8 ignore stop */

    return () => {
      mounted = false;
    };
  }, []);

  const updateListeningEnabled = useCallback(
    async (enabled: boolean) => {
      /* v8 ignore start -- @preserve */
      if (!store) return;
      try {
        await store.set("listening.enabled", enabled);
        setSettings((prev) => ({
          ...prev,
          listening: { ...prev.listening, enabled },
        }));
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
      /* v8 ignore stop */
    },
    [store]
  );

  const updateAutoStartListening = useCallback(
    async (enabled: boolean) => {
      /* v8 ignore start -- @preserve */
      if (!store) return;
      try {
        await store.set("listening.autoStartOnLaunch", enabled);
        setSettings((prev) => ({
          ...prev,
          listening: { ...prev.listening, autoStartOnLaunch: enabled },
        }));
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
      /* v8 ignore stop */
    },
    [store]
  );

  const updateAudioDevice = useCallback(
    async (deviceName: string | null) => {
      /* v8 ignore start -- @preserve */
      if (!store) return;
      try {
        await store.set("audio.selectedDevice", deviceName);
        setSettings((prev) => ({
          ...prev,
          audio: { ...prev.audio, selectedDevice: deviceName },
        }));
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
      /* v8 ignore stop */
    },
    [store]
  );

  return {
    settings,
    isLoading,
    error,
    updateListeningEnabled,
    updateAutoStartListening,
    updateAudioDevice,
  };
}

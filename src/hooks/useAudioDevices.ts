import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AudioInputDevice } from "../types/audio";

export interface UseAudioDevicesResult {
  devices: AudioInputDevice[];
  isLoading: boolean;
  error: Error | null;
  refresh: () => void;
}

/**
 * Hook for fetching available audio input devices from the backend.
 * Automatically loads devices on mount and provides a refresh function.
 */
export function useAudioDevices(): UseAudioDevicesResult {
  const [devices, setDevices] = useState<AudioInputDevice[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchDevices = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<AudioInputDevice[]>("list_audio_devices");
      setDevices(result);
    } catch (e) {
      setError(e instanceof Error ? e : new Error(String(e)));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDevices();
  }, [fetchDevices]);

  return { devices, isLoading, error, refresh: fetchDevices };
}

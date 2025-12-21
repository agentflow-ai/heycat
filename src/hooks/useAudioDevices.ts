import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { AudioInputDevice } from "../types/audio";
import { queryKeys } from "../lib/queryKeys";

const DEFAULT_REFRESH_INTERVAL_MS = 5000;

export interface UseAudioDevicesOptions {
  /** Enable periodic refresh while hook is active (default: true) */
  autoRefresh?: boolean;
  /** Refresh interval in milliseconds (default: 5000) */
  refreshInterval?: number;
}

export interface UseAudioDevicesResult {
  devices: AudioInputDevice[];
  isLoading: boolean;
  error: Error | null;
  refetch: () => void;
}

/**
 * Hook for fetching available audio input devices from the backend.
 * Uses Tanstack Query for caching and automatic refetching.
 * Re-fetches on window focus and periodically when autoRefresh is enabled.
 */
export function useAudioDevices(
  options: UseAudioDevicesOptions = {}
): UseAudioDevicesResult {
  const { autoRefresh = true, refreshInterval = DEFAULT_REFRESH_INTERVAL_MS } =
    options;

  const queryClient = useQueryClient();

  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.tauri.listAudioDevices,
    queryFn: async () => {
      const result = await invoke<AudioInputDevice[]>("list_audio_devices");
      return result;
    },
    refetchInterval: autoRefresh ? refreshInterval : false,
    refetchOnWindowFocus: true,
  });

  return {
    devices: data ?? [],
    isLoading,
    error: error instanceof Error ? error : error ? new Error(String(error)) : null,
    refetch: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.tauri.listAudioDevices });
    },
  };
}

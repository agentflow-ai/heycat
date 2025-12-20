import { useQuery, useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { queryKeys } from "../lib/queryKeys";
import { useListeningUIState } from "../stores/appStore";

/** Response from get_listening_status command */
interface ListeningStatusResponse {
  enabled: boolean;
  active: boolean;
  micAvailable: boolean;
}

/** Options for the useListening hook */
export interface UseListeningOptions {
  /** Device name to listen from (null = system default) */
  deviceName?: string | null;
}

/** Return type of the useListeningStatus hook */
export interface UseListeningStatusResult {
  isListening: boolean;
  isMicAvailable: boolean;
  isLoading: boolean;
  error: Error | null;
}

/** Return type of the useListening hook (backward compatible) */
export interface UseListeningReturn {
  isListening: boolean;
  isWakeWordDetected: boolean;
  isMicAvailable: boolean;
  error: string | null;
  enableListening: () => Promise<void>;
  disableListening: () => Promise<void>;
}

/**
 * Hook for reading listening status using Tanstack Query.
 * Uses Event Bridge for cache invalidation on listening_started/listening_stopped events.
 */
export function useListeningStatus(): UseListeningStatusResult {
  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.tauri.getListeningStatus,
    queryFn: () => invoke<ListeningStatusResponse>("get_listening_status"),
  });

  return {
    isListening: data?.enabled ?? false,
    isMicAvailable: data?.micAvailable ?? true,
    isLoading,
    error: error instanceof Error ? error : error ? new Error(String(error)) : null,
  };
}

/**
 * Mutation hook for enabling listening mode.
 * Event Bridge handles cache invalidation on listening_started event.
 */
export function useEnableListening() {
  return useMutation({
    mutationFn: (deviceName?: string) =>
      invoke("enable_listening", { deviceName }),
    // No onSuccess invalidation - Event Bridge handles this via listening_started
  });
}

/**
 * Mutation hook for disabling listening mode.
 * Event Bridge handles cache invalidation on listening_stopped event.
 */
export function useDisableListening() {
  return useMutation({
    mutationFn: () => invoke("disable_listening"),
    // No onSuccess invalidation - Event Bridge handles this via listening_stopped
  });
}

/**
 * Custom hook for managing listening mode state (backward compatible).
 * Combines query and mutation hooks for convenience.
 *
 * State updates happen via Event Bridge, not command responses.
 * This ensures hotkey-triggered listening changes update the UI correctly.
 *
 * @param options Configuration options including device selection
 */
export function useListening(
  options: UseListeningOptions = {}
): UseListeningReturn {
  const { deviceName } = options;
  const { isListening, isMicAvailable, error } = useListeningStatus();
  const { isWakeWordDetected } = useListeningUIState();
  const enableMutation = useEnableListening();
  const disableMutation = useDisableListening();

  const enableListening = async () => {
    try {
      await enableMutation.mutateAsync(deviceName ?? undefined);
      // State will be updated by Event Bridge on listening_started event
    } catch {
      // Error is captured in mutation state
    }
  };

  const disableListening = async () => {
    try {
      await disableMutation.mutateAsync();
      // State will be updated by Event Bridge on listening_stopped event
    } catch {
      // Error is captured in mutation state
    }
  };

  // Combine errors: query error, or mutation errors
  const combinedError = error?.message
    ?? (enableMutation.error instanceof Error ? enableMutation.error.message : null)
    ?? (disableMutation.error instanceof Error ? disableMutation.error.message : null)
    ?? null;

  return {
    isListening,
    isWakeWordDetected,
    isMicAvailable,
    error: combinedError,
    enableListening,
    disableListening,
  };
}

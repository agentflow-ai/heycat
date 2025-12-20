import { useQuery, useMutation } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { queryKeys } from "../lib/queryKeys";

/** Metadata returned when recording stops */
export interface RecordingMetadata {
  duration_secs: number;
  file_path: string;
  sample_count: number;
}

/** Response from get_recording_state command */
interface RecordingStateResponse {
  state: "Idle" | "Recording" | "Processing" | "Listening";
}

/** Options for the useRecording hook */
export interface UseRecordingOptions {
  /** Device name to record from (null = system default) */
  deviceName?: string | null;
}

/** Return type of the useRecordingState hook */
export interface UseRecordingStateResult {
  isRecording: boolean;
  isProcessing: boolean;
  isLoading: boolean;
  error: Error | null;
}

/** Return type of the useRecording hook (backward compatible) */
export interface UseRecordingResult {
  isRecording: boolean;
  isProcessing: boolean;
  error: string | null;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  isStarting: boolean;
  isStopping: boolean;
}

/**
 * Hook for reading recording state using Tanstack Query.
 * Uses Event Bridge for cache invalidation on recording_started/recording_stopped events.
 */
export function useRecordingState(): UseRecordingStateResult {
  const { data, isLoading, error } = useQuery({
    queryKey: queryKeys.tauri.getRecordingState,
    queryFn: () => invoke<RecordingStateResponse>("get_recording_state"),
  });

  return {
    isRecording: data?.state === "Recording",
    isProcessing: data?.state === "Processing",
    isLoading,
    error: error instanceof Error ? error : error ? new Error(String(error)) : null,
  };
}

/**
 * Mutation hook for starting recording.
 * Event Bridge handles cache invalidation on recording_started event.
 */
export function useStartRecording() {
  return useMutation({
    mutationFn: (deviceName?: string) =>
      invoke("start_recording", { deviceName }),
    // No onSuccess invalidation - Event Bridge handles this via recording_started
  });
}

/**
 * Mutation hook for stopping recording.
 * Event Bridge handles cache invalidation on recording_stopped event.
 */
export function useStopRecording() {
  return useMutation({
    mutationFn: () => invoke("stop_recording"),
    // No onSuccess invalidation - Event Bridge handles this via recording_stopped
  });
}

/**
 * Custom hook for managing recording state (backward compatible).
 * Combines query and mutation hooks for convenience.
 *
 * State updates happen via Event Bridge, not command responses.
 * This ensures hotkey-triggered recordings update the UI correctly.
 *
 * @param options Configuration options including device selection
 */
export function useRecording(
  options: UseRecordingOptions = {}
): UseRecordingResult {
  const { deviceName } = options;
  const { isRecording, isProcessing, error } = useRecordingState();
  const startMutation = useStartRecording();
  const stopMutation = useStopRecording();

  const startRecording = async () => {
    try {
      await startMutation.mutateAsync(deviceName ?? undefined);
      // State will be updated by Event Bridge on recording_started event
    } catch {
      // Error is captured in mutation state
    }
  };

  const stopRecording = async () => {
    try {
      await stopMutation.mutateAsync();
      // State will be updated by Event Bridge on recording_stopped event
    } catch {
      // Error is captured in mutation state
    }
  };

  // Combine errors: query error, or mutation errors
  const combinedError = error?.message
    ?? (startMutation.error instanceof Error ? startMutation.error.message : null)
    ?? (stopMutation.error instanceof Error ? stopMutation.error.message : null)
    ?? null;

  return {
    isRecording,
    isProcessing,
    error: combinedError,
    startRecording,
    stopRecording,
    isStarting: startMutation.isPending,
    isStopping: stopMutation.isPending,
  };
}

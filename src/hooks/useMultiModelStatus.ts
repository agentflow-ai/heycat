import { useState, useEffect, useCallback } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { queryKeys } from "../lib/queryKeys";

/** Types of models that can be downloaded */
export type ModelType = "tdt";

/** Download state for a model */
export type DownloadState = "idle" | "downloading" | "completed" | "error";

/** Status for a single model */
export interface ModelStatus {
  isAvailable: boolean;
  downloadState: DownloadState;
  progress: number; // 0-100
  error: string | null;
}

/** Payload for model_file_download_progress event */
export interface ModelFileDownloadProgressPayload {
  modelType: string;
  fileName: string;
  percent: number;
  bytesDownloaded: number;
  totalBytes: number;
}

/** Return type of the useMultiModelStatus hook */
export interface UseMultiModelStatusResult {
  /** Status for the TDT model */
  models: ModelStatus;
  /** Function to start downloading the model */
  downloadModel: (modelType: ModelType) => Promise<void>;
  /** Function to refresh model status */
  refreshStatus: () => void;
}

/**
 * Custom hook for managing TDT model status
 * Uses Tanstack Query for model availability, local state for download progress
 */
export function useMultiModelStatus(): UseMultiModelStatusResult {
  const queryClient = useQueryClient();

  // Download progress is transient UI state (updated at high frequency)
  const [downloadState, setDownloadState] = useState<DownloadState>("idle");
  const [progress, setProgress] = useState(0);
  const [downloadError, setDownloadError] = useState<string | null>(null);

  // Query for model availability (server state)
  const { data: isAvailable = false } = useQuery({
    queryKey: queryKeys.tauri.checkModelStatus("tdt"),
    queryFn: () => invoke<boolean>("check_parakeet_model_status", { modelType: "tdt" }),
  });

  // Mutation for triggering download
  const downloadMutation = useMutation({
    mutationFn: (modelType: ModelType) => invoke("download_model", { modelType }),
    onMutate: () => {
      setDownloadState("downloading");
      setProgress(0);
      setDownloadError(null);
    },
    onError: (error) => {
      setDownloadState("error");
      setDownloadError(error instanceof Error ? error.message : String(error));
    },
    // Success is handled by the model_download_completed event via Event Bridge
  });

  // Listen for download progress events (high-frequency UI updates)
  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    const setupListeners = async () => {
      // Listen for download progress (transient state, not in Query)
      const unlistenProgress = await listen<ModelFileDownloadProgressPayload>(
        "model_file_download_progress",
        (event) => {
          if (event.payload.modelType === "tdt") {
            setProgress(event.payload.percent);
          }
        }
      );
      unlistenFns.push(unlistenProgress);

      // Listen for download completion to update local state
      // (Query invalidation is handled by Event Bridge)
      const unlistenCompleted = await listen<{ modelType: string }>(
        "model_download_completed",
        (event) => {
          if (event.payload.modelType === "tdt") {
            setDownloadState("completed");
            setProgress(100);
            setDownloadError(null);
          }
        }
      );
      unlistenFns.push(unlistenCompleted);
    };

    setupListeners();

    return () => {
      unlistenFns.forEach((unlisten) => unlisten());
    };
  }, []);

  // Derive download state from query data when model is already available
  const effectiveDownloadState: DownloadState = isAvailable && downloadState === "idle"
    ? "completed"
    : downloadState;

  const downloadModel = useCallback(
    async (modelType: ModelType) => {
      await downloadMutation.mutateAsync(modelType);
    },
    [downloadMutation]
  );

  const refreshStatus = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: queryKeys.tauri.checkModelStatus("tdt") });
  }, [queryClient]);

  return {
    models: {
      isAvailable,
      downloadState: effectiveDownloadState,
      progress,
      error: downloadError,
    },
    downloadModel,
    refreshStatus,
  };
}

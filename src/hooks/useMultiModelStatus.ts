import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/** Types of models that can be downloaded */
export type ModelType = "tdt" | "eou";

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
  model_type: string;
  file_name: string;
  percent: number;
  bytes_downloaded: number;
  total_bytes: number;
}

/** Payload for model_download_completed event */
interface ModelDownloadCompletedPayload {
  model_type: string;
  model_path: string;
}

/** Return type of the useMultiModelStatus hook */
export interface UseMultiModelStatusResult {
  /** Status for each model type */
  models: Record<ModelType, ModelStatus>;
  /** Function to start downloading a specific model */
  downloadModel: (modelType: ModelType) => Promise<void>;
  /** Function to refresh status for all models */
  refreshStatus: () => Promise<void>;
}

const initialModelStatus: ModelStatus = {
  isAvailable: false,
  downloadState: "idle",
  progress: 0,
  error: null,
};

/**
 * Custom hook for managing multiple model statuses (TDT and EOU)
 * Provides methods to check availability and trigger downloads for each model type
 */
export function useMultiModelStatus(): UseMultiModelStatusResult {
  const [models, setModels] = useState<Record<ModelType, ModelStatus>>({
    tdt: { ...initialModelStatus },
    eou: { ...initialModelStatus },
  });

  const updateModelStatus = useCallback(
    (modelType: ModelType, updates: Partial<ModelStatus>) => {
      setModels((prev) => ({
        ...prev,
        [modelType]: { ...prev[modelType], ...updates },
      }));
    },
    []
  );

  const refreshStatus = useCallback(async () => {
    /* v8 ignore start -- @preserve */
    try {
      const [tdtAvailable, eouAvailable] = await Promise.all([
        invoke<boolean>("check_parakeet_model_status", { modelType: "ParakeetTDT" }),
        invoke<boolean>("check_parakeet_model_status", { modelType: "ParakeetEOU" }),
      ]);

      setModels((prev) => ({
        tdt: {
          ...prev.tdt,
          isAvailable: tdtAvailable,
          downloadState: tdtAvailable ? "completed" : prev.tdt.downloadState,
        },
        eou: {
          ...prev.eou,
          isAvailable: eouAvailable,
          downloadState: eouAvailable ? "completed" : prev.eou.downloadState,
        },
      }));
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setModels((prev) => ({
        tdt: { ...prev.tdt, error: errorMsg },
        eou: { ...prev.eou, error: errorMsg },
      }));
    }
    /* v8 ignore stop */
  }, []);

  const downloadModel = useCallback(
    async (modelType: ModelType) => {
      updateModelStatus(modelType, {
        error: null,
        downloadState: "downloading",
        progress: 0,
      });
      /* v8 ignore start -- @preserve */
      try {
        await invoke("download_model", { modelType });
        // State will be updated by model_download_completed event
      } catch (e) {
        updateModelStatus(modelType, {
          downloadState: "error",
          error: e instanceof Error ? e.message : String(e),
        });
      }
      /* v8 ignore stop */
    },
    [updateModelStatus]
  );

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListeners = async () => {
      // Check initial model status
      await refreshStatus();

      // Listen for download progress
      const unlistenProgress = await listen<ModelFileDownloadProgressPayload>(
        "model_file_download_progress",
        (event) => {
          const modelType = event.payload.model_type as ModelType;
          if (modelType === "tdt" || modelType === "eou") {
            updateModelStatus(modelType, {
              progress: event.payload.percent,
            });
          }
        }
      );
      unlistenFns.push(unlistenProgress);

      // Listen for download completion
      const unlistenCompleted = await listen<ModelDownloadCompletedPayload>(
        "model_download_completed",
        (event) => {
          const modelType = event.payload.model_type as ModelType;
          if (modelType === "tdt" || modelType === "eou") {
            updateModelStatus(modelType, {
              isAvailable: true,
              downloadState: "completed",
              progress: 100,
              error: null,
            });
          }
        }
      );
      unlistenFns.push(unlistenCompleted);
    };

    setupListeners();
    /* v8 ignore stop */

    return () => {
      /* v8 ignore start -- @preserve */
      unlistenFns.forEach((unlisten) => unlisten());
      /* v8 ignore stop */
    };
  }, [refreshStatus, updateModelStatus]);

  return {
    models,
    downloadModel,
    refreshStatus,
  };
}

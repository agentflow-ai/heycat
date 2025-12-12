import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/** Payload for model_download_completed event */
interface ModelDownloadCompletedPayload {
  model_path: string;
}

/** Download state for the model */
export type DownloadState = "idle" | "downloading" | "completed" | "error";

/** Return type of the useModelStatus hook */
export interface UseModelStatusResult {
  /** Whether the model is available on disk */
  isModelAvailable: boolean;
  /** Current download state */
  downloadState: DownloadState;
  /** Error message if download failed */
  error: string | null;
  /** Function to start downloading the model */
  downloadModel: () => Promise<void>;
  /** Function to refresh model status from backend */
  refreshStatus: () => Promise<void>;
}

/**
 * Custom hook for managing whisper model status
 * Provides methods to check availability and trigger downloads
 */
export function useModelStatus(): UseModelStatusResult {
  const [isModelAvailable, setIsModelAvailable] = useState(false);
  const [downloadState, setDownloadState] = useState<DownloadState>("idle");
  const [error, setError] = useState<string | null>(null);

  const refreshStatus = useCallback(async () => {
    /* v8 ignore start -- @preserve */
    try {
      const available = await invoke<boolean>("check_model_status");
      setIsModelAvailable(available);
      if (available) {
        setDownloadState("completed");
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
    /* v8 ignore stop */
  }, []);

  const downloadModel = useCallback(async () => {
    setError(null);
    setDownloadState("downloading");
    /* v8 ignore start -- @preserve */
    try {
      await invoke("download_model");
      // State will be updated by model_download_completed event
    } catch (e) {
      setDownloadState("error");
      setError(e instanceof Error ? e.message : String(e));
    }
    /* v8 ignore stop */
  }, []);

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListeners = async () => {
      // Check initial model status
      await refreshStatus();

      // Listen for download completion
      const unlistenCompleted = await listen<ModelDownloadCompletedPayload>(
        "model_download_completed",
        () => {
          setIsModelAvailable(true);
          setDownloadState("completed");
          setError(null);
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
  }, [refreshStatus]);

  return {
    isModelAvailable,
    downloadState,
    error,
    downloadModel,
    refreshStatus,
  };
}

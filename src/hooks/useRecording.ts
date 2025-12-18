import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/** Metadata returned when recording stops */
export interface RecordingMetadata {
  duration_secs: number;
  file_path: string;
  sample_count: number;
}

/** Payload for recording_started event */
interface RecordingStartedPayload {
  timestamp: string;
}

/** Payload for recording_stopped event */
interface RecordingStoppedPayload {
  metadata: RecordingMetadata;
}

/** Payload for recording_error event */
interface RecordingErrorPayload {
  message: string;
}

/** Payload for recording_cancelled event */
interface RecordingCancelledPayload {
  reason: string;
  timestamp: string;
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

/** Return type of the useRecording hook */
export interface UseRecordingResult {
  isRecording: boolean;
  error: string | null;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  lastRecording: RecordingMetadata | null;
  /** True if the last recording was cancelled (not stopped normally) */
  wasCancelled: boolean;
  /** Reason for cancellation (e.g., "double-tap-escape"), null if not cancelled */
  cancelReason: string | null;
}

/**
 * Custom hook for managing recording state
 * Provides methods to start/stop recording and listens to backend events
 *
 * @param options Configuration options including device selection
 */
export function useRecording(
  options: UseRecordingOptions = {}
): UseRecordingResult {
  const { deviceName } = options;
  const [isRecording, setIsRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lastRecording, setLastRecording] = useState<RecordingMetadata | null>(
    null
  );
  const [wasCancelled, setWasCancelled] = useState(false);
  const [cancelReason, setCancelReason] = useState<string | null>(null);

  // Fetch initial recording state from backend on mount
  useEffect(() => {
    /* v8 ignore start -- @preserve */
    async function fetchInitialState() {
      try {
        const status = await invoke<RecordingStateResponse>("get_recording_state");
        setIsRecording(status.state === "Recording");
      } catch {
        // Silently handle error - state will be updated via events
      }
    }
    fetchInitialState();
    /* v8 ignore stop */
  }, []);

  // Note: State updates happen via events, not command responses.
  // This ensures hotkey-triggered recordings update the UI correctly.
  const startRecording = useCallback(async () => {
    setError(null);
    /* v8 ignore start -- @preserve */
    try {
      await invoke("start_recording", {
        deviceName: deviceName ?? undefined,
      });
      // State will be updated by recording_started event
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
    /* v8 ignore stop */
  }, [deviceName]);

  const stopRecording = useCallback(async () => {
    setError(null);
    /* v8 ignore start -- @preserve */
    try {
      await invoke<RecordingMetadata>("stop_recording");
      // State will be updated by recording_stopped event
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
    /* v8 ignore stop */
  }, []);

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListeners = async () => {
      const unlistenStarted = await listen<RecordingStartedPayload>(
        "recording_started",
        () => {
          setIsRecording(true);
          setError(null);
          // Reset cancelled state when new recording starts
          setWasCancelled(false);
          setCancelReason(null);
        }
      );
      unlistenFns.push(unlistenStarted);

      const unlistenStopped = await listen<RecordingStoppedPayload>(
        "recording_stopped",
        (event) => {
          setIsRecording(false);
          setLastRecording(event.payload.metadata);
          setError(null);
        }
      );
      unlistenFns.push(unlistenStopped);

      const unlistenError = await listen<RecordingErrorPayload>(
        "recording_error",
        (event) => {
          setError(event.payload.message);
        }
      );
      unlistenFns.push(unlistenError);

      const unlistenCancelled = await listen<RecordingCancelledPayload>(
        "recording_cancelled",
        (event) => {
          setIsRecording(false);
          setWasCancelled(true);
          setCancelReason(event.payload.reason);
          setError(null);
        }
      );
      unlistenFns.push(unlistenCancelled);
    };

    setupListeners();
    /* v8 ignore stop */

    return () => {
      /* v8 ignore start -- @preserve */
      unlistenFns.forEach((unlisten) => unlisten());
      /* v8 ignore stop */
    };
  }, []);

  return {
    isRecording,
    error,
    startRecording,
    stopRecording,
    lastRecording,
    wasCancelled,
    cancelReason,
  };
}

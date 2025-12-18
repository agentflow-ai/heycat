import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/** Payload for listening_started event */
interface ListeningStartedPayload {
  timestamp: string;
}

/** Payload for listening_stopped event */
interface ListeningStoppedPayload {
  timestamp: string;
}

/** Payload for wake_word_detected event */
interface WakeWordDetectedPayload {
  confidence: number;
  transcription: string;
  timestamp: string;
}

/** Payload for listening_unavailable event */
interface ListeningUnavailablePayload {
  reason: string;
  timestamp: string;
}

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

/** Return type of the useListening hook */
export interface UseListeningReturn {
  isListening: boolean;
  isWakeWordDetected: boolean;
  isMicAvailable: boolean;
  error: string | null;
  enableListening: () => Promise<void>;
  disableListening: () => Promise<void>;
}

/**
 * Custom hook for managing listening mode state
 * Provides methods to enable/disable listening and listens to backend events
 *
 * @param options Configuration options including device selection
 */
export function useListening(
  options: UseListeningOptions = {}
): UseListeningReturn {
  const { deviceName } = options;
  const [isListening, setIsListening] = useState(false);
  const [isWakeWordDetected, setIsWakeWordDetected] = useState(false);
  const [isMicAvailable, setIsMicAvailable] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Note: State updates happen via events, not command responses.
  // This ensures hotkey-triggered listening changes update the UI correctly.
  const enableListening = useCallback(async () => {
    setError(null);
    /* v8 ignore start -- @preserve */
    try {
      await invoke("enable_listening", {
        deviceName: deviceName ?? undefined,
      });
      // State will be updated by listening_started event
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
    /* v8 ignore stop */
  }, [deviceName]);

  const disableListening = useCallback(async () => {
    setError(null);
    /* v8 ignore start -- @preserve */
    try {
      await invoke("disable_listening");
      // State will be updated by listening_stopped event
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
    /* v8 ignore stop */
  }, []);

  // Fetch initial listening state from backend on mount
  useEffect(() => {
    /* v8 ignore start -- @preserve */
    async function fetchInitialState() {
      try {
        const status = await invoke<ListeningStatusResponse>("get_listening_status");
        setIsListening(status.enabled);
        setIsMicAvailable(status.micAvailable);
      } catch {
        // Silently handle error - state will be updated via events
      }
    }
    fetchInitialState();
    /* v8 ignore stop */
  }, []);

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListeners = async () => {
      const unlistenStarted = await listen<ListeningStartedPayload>(
        "listening_started",
        () => {
          setIsListening(true);
          setError(null);
          setIsMicAvailable(true);
        }
      );
      unlistenFns.push(unlistenStarted);

      const unlistenStopped = await listen<ListeningStoppedPayload>(
        "listening_stopped",
        () => {
          setIsListening(false);
          setError(null);
        }
      );
      unlistenFns.push(unlistenStopped);

      const unlistenWakeWord = await listen<WakeWordDetectedPayload>(
        "wake_word_detected",
        () => {
          setIsWakeWordDetected(true);
          // Reset after a short delay to make it transient
          setTimeout(() => {
            setIsWakeWordDetected(false);
          }, 500);
        }
      );
      unlistenFns.push(unlistenWakeWord);

      const unlistenUnavailable = await listen<ListeningUnavailablePayload>(
        "listening_unavailable",
        (event) => {
          setIsMicAvailable(false);
          setIsListening(false);
          setError(event.payload.reason);
        }
      );
      unlistenFns.push(unlistenUnavailable);
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
    isListening,
    isWakeWordDetected,
    isMicAvailable,
    error,
    enableListening,
    disableListening,
  };
}

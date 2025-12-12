import { useState, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/** Payload for transcription_started event */
interface TranscriptionStartedPayload {
  timestamp: string;
}

/** Payload for transcription_completed event */
interface TranscriptionCompletedPayload {
  text: string;
  duration_ms: number;
}

/** Payload for transcription_error event */
interface TranscriptionErrorPayload {
  error: string;
}

/** Return type of the useTranscription hook */
export interface UseTranscriptionResult {
  isTranscribing: boolean;
  transcribedText: string | null;
  error: string | null;
  durationMs: number | null;
}

/**
 * Custom hook for managing transcription state
 * Listens to backend transcription events and updates state accordingly
 */
export function useTranscription(): UseTranscriptionResult {
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [transcribedText, setTranscribedText] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [durationMs, setDurationMs] = useState<number | null>(null);

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];

    /* v8 ignore start -- @preserve */
    const setupListeners = async () => {
      const unlistenStarted = await listen<TranscriptionStartedPayload>(
        "transcription_started",
        () => {
          setIsTranscribing(true);
          setError(null);
          setTranscribedText(null);
          setDurationMs(null);
        }
      );
      unlistenFns.push(unlistenStarted);

      const unlistenCompleted = await listen<TranscriptionCompletedPayload>(
        "transcription_completed",
        (event) => {
          setIsTranscribing(false);
          setTranscribedText(event.payload.text);
          setDurationMs(event.payload.duration_ms);
          setError(null);
        }
      );
      unlistenFns.push(unlistenCompleted);

      const unlistenError = await listen<TranscriptionErrorPayload>(
        "transcription_error",
        (event) => {
          setIsTranscribing(false);
          setError(event.payload.error);
        }
      );
      unlistenFns.push(unlistenError);
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
    isTranscribing,
    transcribedText,
    error,
    durationMs,
  };
}

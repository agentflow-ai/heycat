import { useTranscriptionState } from "../stores/appStore";

/** Return type of the useTranscription hook */
export interface UseTranscriptionResult {
  isTranscribing: boolean;
  transcribedText: string | null;
  error: string | null;
  durationMs: number | null;
}

/**
 * Custom hook for managing transcription state.
 * Reads from Zustand store which is updated via Event Bridge.
 *
 * The transcription events are handled centrally by the Event Bridge,
 * which updates the Zustand store. This hook provides a convenient
 * interface for components to access the transcription state.
 */
export function useTranscription(): UseTranscriptionResult {
  const transcription = useTranscriptionState();

  return {
    isTranscribing: transcription.isTranscribing,
    transcribedText: transcription.transcribedText,
    error: transcription.error,
    durationMs: transcription.durationMs,
  };
}

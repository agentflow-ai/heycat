import { useMemo } from "react";
import { useRecording } from "./useRecording";
import { useTranscription } from "./useTranscription";
import type { StatusPillStatus } from "../components/ui/StatusPill";

export interface UseAppStatusResult {
  /** Current app status derived from all state hooks */
  status: StatusPillStatus;
  /** Whether recording is in progress */
  isRecording: boolean;
  /** Whether transcription is in progress */
  isTranscribing: boolean;
  /** Any error from the hooks */
  error: string | null;
}

/**
 * Derives the combined app status from recording and transcription hooks.
 * Priority order: recording > processing > idle
 */
export function useAppStatus(): UseAppStatusResult {
  const { isRecording, error: recordingError } = useRecording();
  const { isTranscribing, error: transcriptionError } = useTranscription();

  const status = useMemo<StatusPillStatus>(() => {
    if (isRecording) return "recording";
    if (isTranscribing) return "processing";
    return "idle";
  }, [isRecording, isTranscribing]);

  const error = recordingError ?? transcriptionError ?? null;

  return {
    status,
    isRecording,
    isTranscribing,
    error,
  };
}

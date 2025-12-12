/* v8 ignore file -- @preserve */
import { useRecording } from "../hooks/useRecording";
import "./RecordingIndicator.css";

export interface RecordingIndicatorProps {
  className?: string;
  /** When true, recording is blocked (e.g., during transcription) */
  isBlocked?: boolean;
}

export function RecordingIndicator({
  className = "",
  isBlocked = false,
}: RecordingIndicatorProps) {
  const { isRecording, error } = useRecording();

  const stateClass = isBlocked
    ? "recording-indicator--blocked"
    : isRecording
      ? "recording-indicator--recording"
      : "recording-indicator--idle";
  const statusText = isBlocked ? "Recording blocked" : isRecording ? "Recording" : "Idle";

  return (
    <div
      className={`recording-indicator ${stateClass} ${className}`.trim()}
      role="status"
      aria-live="polite"
      aria-label={`Recording status: ${statusText}`}
    >
      <span className="recording-indicator__dot" aria-hidden="true" />
      <span className="recording-indicator__label">{statusText}</span>
      {error && (
        <span className="recording-indicator__error" role="alert">
          {error}
        </span>
      )}
    </div>
  );
}

/* v8 ignore file -- @preserve */
import { useRecording } from "../hooks/useRecording";
import "./RecordingIndicator.css";

export interface RecordingIndicatorProps {
  className?: string;
}

export function RecordingIndicator({
  className = "",
}: RecordingIndicatorProps) {
  const { isRecording, error } = useRecording();

  const stateClass = isRecording
    ? "recording-indicator--recording"
    : "recording-indicator--idle";
  const statusText = isRecording ? "Recording" : "Idle";

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

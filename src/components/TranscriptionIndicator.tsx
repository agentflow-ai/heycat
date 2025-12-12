/* v8 ignore file -- @preserve */
import { useTranscription } from "../hooks/useTranscription";
import "./TranscriptionIndicator.css";

export interface TranscriptionIndicatorProps {
  className?: string;
}

export function TranscriptionIndicator({
  className = "",
}: TranscriptionIndicatorProps) {
  const { isTranscribing } = useTranscription();

  if (!isTranscribing) {
    return null;
  }

  return (
    <div
      className={`transcription-indicator ${className}`.trim()}
      role="status"
      aria-live="polite"
      aria-busy="true"
      aria-label="Transcribing audio"
    >
      <span className="transcription-indicator__spinner" aria-hidden="true" />
      <span className="transcription-indicator__label">Transcribing...</span>
    </div>
  );
}

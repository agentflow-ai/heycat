/* v8 ignore file -- @preserve */
import { useState, useEffect } from "react";
import { useTranscription } from "../hooks/useTranscription";
import "./TranscriptionNotification.css";

export interface TranscriptionNotificationProps {
  className?: string;
  /** Auto-dismiss delay for success notifications in ms (default: 5000) */
  autoDismissDelay?: number;
}

function truncateText(text: string, maxLength: number = 50): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength) + "...";
}

export function TranscriptionNotification({
  className = "",
  autoDismissDelay = 5000,
}: TranscriptionNotificationProps) {
  const { transcribedText, error } = useTranscription();
  const [dismissedSuccess, setDismissedSuccess] = useState(false);
  const [dismissedError, setDismissedError] = useState(false);
  const [lastText, setLastText] = useState<string | null>(null);
  const [lastError, setLastError] = useState<string | null>(null);

  // Track when transcribedText changes to reset dismiss state
  useEffect(() => {
    if (transcribedText && transcribedText !== lastText) {
      setDismissedSuccess(false);
      setLastText(transcribedText);
    }
  }, [transcribedText, lastText]);

  // Track when error changes to reset dismiss state
  useEffect(() => {
    if (error && error !== lastError) {
      setDismissedError(false);
      setLastError(error);
    }
  }, [error, lastError]);

  // Auto-dismiss success notification
  useEffect(() => {
    if (transcribedText && !dismissedSuccess) {
      const timer = setTimeout(() => {
        setDismissedSuccess(true);
      }, autoDismissDelay);
      return () => clearTimeout(timer);
    }
  }, [transcribedText, dismissedSuccess, autoDismissDelay]);

  const handleDismissError = () => {
    setDismissedError(true);
  };

  const showSuccess = transcribedText && !dismissedSuccess && !error;
  const showError = error && !dismissedError;

  if (!showSuccess && !showError) {
    return null;
  }

  return (
    <div className={`transcription-notification ${className}`.trim()}>
      {showSuccess && (
        <div
          className="transcription-notification__success"
          role="status"
          aria-live="polite"
        >
          <span className="transcription-notification__icon" aria-hidden="true">
            ✓
          </span>
          <span className="transcription-notification__text">
            {truncateText(transcribedText)} — Copied to clipboard
          </span>
        </div>
      )}
      {showError && (
        <div
          className="transcription-notification__error"
          role="alert"
          aria-live="assertive"
        >
          <span className="transcription-notification__icon" aria-hidden="true">
            ✕
          </span>
          <span className="transcription-notification__text">{error}</span>
          <button
            className="transcription-notification__dismiss"
            onClick={handleDismissError}
            aria-label="Dismiss error"
          >
            ✕
          </button>
        </div>
      )}
    </div>
  );
}

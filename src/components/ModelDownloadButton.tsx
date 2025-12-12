/* v8 ignore file -- @preserve */
import { useModelStatus, DownloadState } from "../hooks/useModelStatus";
import "./ModelDownloadButton.css";

export interface ModelDownloadButtonProps {
  className?: string;
}

function getButtonText(state: DownloadState, isAvailable: boolean): string {
  if (isAvailable || state === "completed") {
    return "Model Ready";
  }
  if (state === "downloading") {
    return "Downloading...";
  }
  if (state === "error") {
    return "Retry Download";
  }
  return "Download Model";
}

function getAriaLabel(state: DownloadState, isAvailable: boolean): string {
  if (isAvailable || state === "completed") {
    return "Whisper model is ready";
  }
  if (state === "downloading") {
    return "Downloading whisper model, please wait";
  }
  if (state === "error") {
    return "Download failed, click to retry";
  }
  return "Click to download whisper model";
}

export function ModelDownloadButton({
  className = "",
}: ModelDownloadButtonProps) {
  const { isModelAvailable, downloadState, error, downloadModel } =
    useModelStatus();

  const isDisabled =
    downloadState === "downloading" ||
    downloadState === "completed" ||
    isModelAvailable;
  const buttonText = getButtonText(downloadState, isModelAvailable);
  const ariaLabel = getAriaLabel(downloadState, isModelAvailable);

  const stateClass =
    downloadState === "completed" || isModelAvailable
      ? "model-download-button--ready"
      : downloadState === "downloading"
        ? "model-download-button--downloading"
        : downloadState === "error"
          ? "model-download-button--error"
          : "model-download-button--idle";

  return (
    <div
      className={`model-download-button ${className}`.trim()}
      role="region"
      aria-label="Model download controls"
    >
      <button
        className={`model-download-button__button ${stateClass}`}
        onClick={downloadModel}
        disabled={isDisabled}
        aria-label={ariaLabel}
        aria-busy={downloadState === "downloading"}
      >
        {downloadState === "downloading" && (
          <span
            className="model-download-button__spinner"
            aria-hidden="true"
          />
        )}
        {buttonText}
      </button>
      {error && (
        <span className="model-download-button__error" role="alert">
          {error}
        </span>
      )}
    </div>
  );
}

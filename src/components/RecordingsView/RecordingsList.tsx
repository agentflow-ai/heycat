import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { EmptyState } from "./EmptyState";
import "./RecordingsList.css";

export interface RecordingInfo {
  filename: string;
  file_path: string;
  duration_secs: number;
  created_at: string;
  file_size_bytes: number;
  /** Error message if the recording has issues (missing file, corrupt metadata) */
  error?: string;
}

export interface RecordingsListProps {
  className?: string;
}

export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export function formatDate(isoString: string): string {
  const date = new Date(isoString);
  return date.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const value = bytes / Math.pow(k, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function RecordingsList({ className = "" }: RecordingsListProps) {
  const [recordings, setRecordings] = useState<RecordingInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedPath, setExpandedPath] = useState<string | null>(null);
  const [openError, setOpenError] = useState<string | null>(null);
  const [transcribingPath, setTranscribingPath] = useState<string | null>(null);
  const [isTdtAvailable, setIsTdtAvailable] = useState(false);

  const toggleExpanded = (filePath: string) => {
    setExpandedPath((current) => (current === filePath ? null : filePath));
  };

  const handleOpenRecording = async (filePath: string, event: React.MouseEvent) => {
    event.stopPropagation();
    setOpenError(null);
    try {
      await openPath(filePath);
    } catch (err) {
      setOpenError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleTranscribe = async (filePath: string, event: React.MouseEvent) => {
    event.stopPropagation();
    setTranscribingPath(filePath);
    try {
      await invoke<string>("transcribe_file", { filePath });
      // Success - text is copied to clipboard, notification shown via events
    } catch (err) {
      console.error("Transcription failed:", err);
    } finally {
      setTranscribingPath(null);
    }
  };

  useEffect(() => {
    async function fetchRecordings() {
      try {
        setIsLoading(true);
        setError(null);
        const result = await invoke<RecordingInfo[]>("list_recordings");
        setRecordings(result);

        // Log errors for any recordings with issues
        result.forEach((recording) => {
          if (recording.error) {
            console.error(
              `Recording error for ${recording.filename}: ${recording.error}`,
              { file_path: recording.file_path }
            );
          }
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setIsLoading(false);
      }
    }

    fetchRecordings();
  }, []);

  // Check TDT model availability on mount
  useEffect(() => {
    async function checkModel() {
      try {
        const available = await invoke<boolean>("check_parakeet_model_status", { modelType: "tdt" });
        setIsTdtAvailable(available);
      } catch (err) {
        console.error("Failed to check model status:", err);
        setIsTdtAvailable(false);
      }
    }
    checkModel();
  }, []);

  if (isLoading) {
    return (
      <div
        className={`recordings-list recordings-list--loading ${className}`.trim()}
        role="status"
        aria-busy="true"
        aria-label="Loading recordings"
      >
        <span className="recordings-list__loading-text">
          Loading recordings...
        </span>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className={`recordings-list recordings-list--error ${className}`.trim()}
        role="alert"
      >
        <span className="recordings-list__error-text">
          Failed to load recordings: {error}
        </span>
      </div>
    );
  }

  if (recordings.length === 0) {
    return <EmptyState hasFiltersActive={false} className={className} />;
  }

  return (
    <div className={`recordings-list ${className}`.trim()}>
      <ul className="recordings-list__items" role="list">
        {recordings.map((recording) => {
          const isExpanded = expandedPath === recording.file_path;
          const hasError = Boolean(recording.error);
          const itemClasses = [
            "recordings-list__item",
            isExpanded && "recordings-list__item--expanded",
            hasError && "recordings-list__item--has-error",
          ].filter(Boolean).join(" ");
          return (
            <li key={recording.file_path} className="recordings-list__item-container">
              <button
                type="button"
                className={itemClasses}
                onClick={() => toggleExpanded(recording.file_path)}
                aria-expanded={isExpanded}
              >
                <span className="recordings-list__filename-wrapper">
                  {hasError && (
                    <svg
                      className="recordings-list__error-indicator"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                      aria-hidden="true"
                    >
                      <path
                        fillRule="evenodd"
                        d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                        clipRule="evenodd"
                      />
                    </svg>
                  )}
                  <span className="recordings-list__filename">{recording.filename}</span>
                </span>
                <span className="recordings-list__duration">
                  {recording.error ? "--:--" : formatDuration(recording.duration_secs)}
                </span>
                <span className="recordings-list__date">
                  {recording.created_at ? formatDate(recording.created_at) : "--"}
                </span>
              </button>
              <div
                className={`recordings-list__details ${isExpanded ? "recordings-list__details--visible" : ""}`}
                aria-hidden={!isExpanded}
              >
                {hasError && (
                  <div className="recordings-list__error-detail" role="alert">
                    {recording.error}
                  </div>
                )}
                <dl className="recordings-list__metadata">
                  <div className="recordings-list__metadata-row">
                    <dt>File size</dt>
                    <dd>{recording.file_size_bytes > 0 ? formatFileSize(recording.file_size_bytes) : "--"}</dd>
                  </div>
                  <div className="recordings-list__metadata-row">
                    <dt>Location</dt>
                    <dd className="recordings-list__path">{recording.file_path}</dd>
                  </div>
                </dl>
                <div className="recordings-list__actions">
                  <button
                    type="button"
                    className="recordings-list__open-button"
                    onClick={(e) => handleOpenRecording(recording.file_path, e)}
                  >
                    Open
                  </button>
                  <button
                    type="button"
                    className="recordings-list__transcribe-button"
                    onClick={(e) => handleTranscribe(recording.file_path, e)}
                    disabled={!isTdtAvailable || transcribingPath === recording.file_path || hasError}
                    title={!isTdtAvailable ? "Download Batch model first" : undefined}
                  >
                    {transcribingPath === recording.file_path ? "Transcribing..." : "Transcribe"}
                  </button>
                </div>
                {openError && expandedPath === recording.file_path && (
                  <div className="recordings-list__open-error" role="alert">
                    Failed to open recording: {openError}
                  </div>
                )}
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

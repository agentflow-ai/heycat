import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EmptyState } from "./EmptyState";
import "./RecordingsList.css";

export interface RecordingInfo {
  filename: string;
  file_path: string;
  duration_secs: number;
  created_at: string;
  file_size_bytes: number;
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

  const toggleExpanded = (filePath: string) => {
    setExpandedPath((current) => (current === filePath ? null : filePath));
  };

  useEffect(() => {
    async function fetchRecordings() {
      try {
        setIsLoading(true);
        setError(null);
        const result = await invoke<RecordingInfo[]>("list_recordings");
        setRecordings(result);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setIsLoading(false);
      }
    }

    fetchRecordings();
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
          return (
            <li key={recording.file_path} className="recordings-list__item-container">
              <button
                type="button"
                className={`recordings-list__item ${isExpanded ? "recordings-list__item--expanded" : ""}`}
                onClick={() => toggleExpanded(recording.file_path)}
                aria-expanded={isExpanded}
              >
                <span className="recordings-list__filename">{recording.filename}</span>
                <span className="recordings-list__duration">
                  {formatDuration(recording.duration_secs)}
                </span>
                <span className="recordings-list__date">
                  {formatDate(recording.created_at)}
                </span>
              </button>
              <div
                className={`recordings-list__details ${isExpanded ? "recordings-list__details--visible" : ""}`}
                aria-hidden={!isExpanded}
              >
                <dl className="recordings-list__metadata">
                  <div className="recordings-list__metadata-row">
                    <dt>File size</dt>
                    <dd>{formatFileSize(recording.file_size_bytes)}</dd>
                  </div>
                  <div className="recordings-list__metadata-row">
                    <dt>Location</dt>
                    <dd className="recordings-list__path">{recording.file_path}</dd>
                  </div>
                </dl>
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

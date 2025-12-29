import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQuery } from "@tanstack/react-query";
import { queryKeys } from "../lib/queryKeys";
import { Play, ArrowRight } from "lucide-react";
import {
  Card,
  CardHeader,
  CardContent,
  Button,
} from "../components/ui";
import { useRecording } from "../hooks/useRecording";
import { useMultiModelStatus } from "../hooks/useMultiModelStatus";
import { useSettings } from "../hooks/useSettings";
import { useRouteContext } from "../routes";
import { formatDuration, formatDate, type RecordingInfo } from "./components/RecordingItem";

export interface DashboardProps {
  /** Navigate to another page (deprecated: use useRouteContext instead) */
  onNavigate?: (page: string) => void;
}

export function Dashboard({ onNavigate: onNavigateProp }: DashboardProps) {
  // Use route context for navigation, fall back to prop for backward compatibility
  const routeContext = useRouteContext();
  const onNavigate = onNavigateProp ?? routeContext?.onNavigate;
  const { settings } = useSettings();
  const { isRecording, startRecording, stopRecording } = useRecording({
    deviceName: settings.audio.selectedDevice,
  });
  const { models, downloadModel } = useMultiModelStatus();

  // Recordings data via React Query - auto-updates via event bridge
  const { data: recordings = [], isLoading: recordingsLoading } = useQuery({
    queryKey: queryKeys.tauri.listRecordings,
    queryFn: () => invoke<RecordingInfo[]>("list_recordings"),
  });

  // Commands count (placeholder until commands system exists)
  const [commandsCount] = useState(0);

  const handleRecordingToggle = async () => {
    if (isRecording) {
      await stopRecording();
    } else {
      await startRecording();
    }
  };

  const handleDownloadModel = async () => {
    await downloadModel("tdt");
  };

  const recentRecordings = recordings.slice(0, 5);
  const isModelDownloading = models.downloadState === "downloading";

  // Double-escape detection for stopping recordings started via button
  // Time window for double-tap detection (300ms, matching backend)
  const DOUBLE_TAP_WINDOW_MS = 300;
  const lastEscapeTime = useRef<number | null>(null);

  const handleEscapeKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !isRecording) return;

      const now = Date.now();
      if (
        lastEscapeTime.current !== null &&
        now - lastEscapeTime.current < DOUBLE_TAP_WINDOW_MS
      ) {
        // Double-tap detected - stop recording
        stopRecording();
        lastEscapeTime.current = null;
      } else {
        // First tap - record time
        lastEscapeTime.current = now;
      }
    },
    [isRecording, stopRecording]
  );

  // Set up escape key listener when recording
  useEffect(() => {
    if (isRecording) {
      window.addEventListener("keydown", handleEscapeKeyDown);
      return () => {
        window.removeEventListener("keydown", handleEscapeKeyDown);
        lastEscapeTime.current = null;
      };
    }
  }, [isRecording, handleEscapeKeyDown]);

  return (
    <div className="p-6 space-y-6">
      {/* Page Header */}
      <header>
        <h1 className="text-2xl font-semibold text-text-primary">Dashboard</h1>
        <p className="text-text-secondary mt-1">
          Welcome back! Here's your HeyCat status.
        </p>
      </header>

      {/* Status Cards Row */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Recordings Card */}
        <Card
          variant="interactive"
          onClick={() => onNavigate?.("recordings")}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              onNavigate?.("recordings");
            }
          }}
        >
          <CardHeader>
            <span className="text-xs font-semibold text-text-secondary uppercase tracking-wider">
              Recordings
            </span>
          </CardHeader>
          <CardContent className="flex items-center justify-between">
            <span className="text-lg font-medium text-text-primary">
              {recordingsLoading ? "..." : `${recordings.length} recordings`}
            </span>
            <ArrowRight className="h-5 w-5 text-text-secondary" />
          </CardContent>
        </Card>

        {/* Commands Card */}
        <Card
          variant="interactive"
          onClick={() => onNavigate?.("commands")}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              onNavigate?.("commands");
            }
          }}
        >
          <CardHeader>
            <span className="text-xs font-semibold text-text-secondary uppercase tracking-wider">
              Commands
            </span>
          </CardHeader>
          <CardContent className="flex items-center justify-between">
            <span className="text-lg font-medium text-text-primary">
              {commandsCount} active
            </span>
            <ArrowRight className="h-5 w-5 text-text-secondary" />
          </CardContent>
        </Card>
      </div>

      {/* Quick Action Buttons */}
      <div className="flex flex-wrap gap-3">
        <Button onClick={handleRecordingToggle}>
          {isRecording ? "Stop Recording" : "Start Recording"}
        </Button>
        <Button variant="secondary" onClick={() => onNavigate?.("commands")}>
          Train Command
        </Button>
        {!models.isAvailable && (
          <Button
            variant="secondary"
            onClick={handleDownloadModel}
            loading={isModelDownloading}
          >
            {isModelDownloading
              ? `Downloading... ${models.progress}%`
              : "Download Model"}
          </Button>
        )}
      </div>

      {/* Recent Activity Section */}
      <section>
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wider">
            Recent Activity
          </h2>
          {recordings.length > 0 && (
            <button
              type="button"
              className="text-sm text-heycat-orange hover:text-heycat-orange-light bg-transparent border-none cursor-pointer transition-colors"
              onClick={() => onNavigate?.("recordings")}
            >
              View all
            </button>
          )}
        </div>

        {recordingsLoading ? (
          <div className="text-text-secondary text-sm">
            Loading recordings...
          </div>
        ) : recentRecordings.length === 0 ? (
          <Card className="text-center py-8">
            <CardContent>
              <p className="text-text-secondary">
                No recordings yet. Click "Start Recording" to get started!
              </p>
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {recentRecordings.map((recording) => (
              <div
                key={recording.file_path}
                className="flex items-center gap-3 p-3 bg-surface rounded-lg border border-border hover:border-heycat-orange/50 transition-colors"
              >
                <button
                  type="button"
                  className="flex items-center justify-center w-8 h-8 rounded-full bg-heycat-orange/20 text-heycat-orange hover:bg-heycat-orange hover:text-white transition-colors"
                  aria-label={`Play ${recording.filename}`}
                >
                  <Play className="h-4 w-4" />
                </button>
                <div className="flex-1 min-w-0">
                  <span className="text-sm font-medium text-text-primary truncate block">
                    {recording.filename}
                  </span>
                  <span className="text-xs text-text-secondary">
                    {formatDate(recording.created_at)}
                  </span>
                </div>
                <span className="text-sm text-text-secondary">
                  {formatDuration(recording.duration_secs)}
                </span>
                <TranscriptionBadge recording={recording} />
              </div>
            ))}
          </div>
        )}

        {/* Model not downloaded prompt */}
        {!models.isAvailable && !recordingsLoading && (
          <Card className="mt-4 border-heycat-orange/30 bg-heycat-cream/50">
            <CardContent className="flex items-center gap-4 py-4">
              <div className="flex-1">
                <p className="text-sm font-medium text-text-primary">
                  Download the transcription model to enable voice commands
                </p>
                <p className="text-xs text-text-secondary mt-1">
                  Required for transcribing recordings and voice activation
                </p>
              </div>
              <Button onClick={handleDownloadModel} loading={isModelDownloading}>
                {isModelDownloading
                  ? `${models.progress}%`
                  : "Download"}
              </Button>
            </CardContent>
          </Card>
        )}
      </section>
    </div>
  );
}

// Helper component for transcription status badge
function TranscriptionBadge({ recording }: { recording: RecordingInfo }) {
  // For now, we don't have transcription status in RecordingInfo
  // This is a placeholder that can be enhanced when transcription tracking is added
  const hasError = Boolean(recording.error);

  if (hasError) {
    return (
      <span className="px-2 py-1 text-xs font-medium rounded-full bg-error/10 text-error">
        Error
      </span>
    );
  }

  // Default to pending since we don't track transcription status yet
  return (
    <span className="px-2 py-1 text-xs font-medium rounded-full bg-text-secondary/10 text-text-secondary">
      Pending
    </span>
  );
}

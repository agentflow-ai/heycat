import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "../lib/queryKeys";
import { openPath } from "@tauri-apps/plugin-opener";
import { Search, ChevronLeft, ChevronRight } from "lucide-react";
import {
  Button,
  Card,
  CardContent,
  Input,
  Select,
  SelectItem,
} from "../components/ui";
import { useToast } from "../components/overlays";
import { useRecording } from "../hooks/useRecording";
import { useSettings } from "../hooks/useSettings";
import { useAudioPlayback } from "../hooks/useAudioPlayback";
import { RecordingItem, type RecordingInfo, type PaginatedRecordingsResponse } from "./components/RecordingItem";
import { RecordingsEmptyState } from "./components/RecordingsEmptyState";

const PAGE_SIZE = 20;

export type FilterOption = "all" | "transcribed" | "pending";
export type SortOption = "newest" | "oldest" | "longest" | "shortest";

export interface RecordingsProps {
  onNavigate?: (page: string) => void;
}

export function Recordings(_props: RecordingsProps) {
  const { toast } = useToast();
  const { settings } = useSettings();
  const { startRecording, isRecording } = useRecording({
    deviceName: settings.audio.selectedDevice,
  });

  const queryClient = useQueryClient();

  // Audio playback hook
  const { toggle: togglePlayback, stop: stopPlayback, isPlaying: isAudioPlaying, currentFilePath: playingFilePath, error: playbackError } = useAudioPlayback();

  // Stop audio playback when a new recording starts
  useEffect(() => {
    if (isRecording && isAudioPlaying) {
      stopPlayback();
    }
  }, [isRecording, isAudioPlaying, stopPlayback]);

  // Display playback errors
  useEffect(() => {
    if (playbackError) {
      console.error("Audio playback error:", playbackError);
      toast({
        type: "error",
        title: "Playback Error",
        description: playbackError,
      });
    }
  }, [playbackError, toast]);

  // Pagination state
  const [currentPage, setCurrentPage] = useState(0);
  const offset = currentPage * PAGE_SIZE;

  // Fetch recordings via React Query - auto-updates via event bridge
  const {
    data: paginatedResponse,
    isLoading: loading,
    error: queryError,
    refetch,
  } = useQuery({
    queryKey: queryKeys.tauri.listRecordings(PAGE_SIZE, offset),
    queryFn: () => invoke<PaginatedRecordingsResponse>("list_recordings", { limit: PAGE_SIZE, offset }),
  });

  const recordings = paginatedResponse?.recordings ?? [];
  const totalCount = paginatedResponse?.total_count ?? 0;
  const hasMore = paginatedResponse?.has_more ?? false;
  const totalPages = Math.ceil(totalCount / PAGE_SIZE);

  const error = queryError ? (queryError instanceof Error ? queryError.message : String(queryError)) : null;

  const [searchQuery, setSearchQuery] = useState("");
  const [filterOption, setFilterOption] = useState<FilterOption>("all");
  const [sortOption, setSortOption] = useState<SortOption>("newest");

  // Expanded item state
  const [expandedPath, setExpandedPath] = useState<string | null>(null);

  // Delete confirmation state
  const [deleteConfirmPath, setDeleteConfirmPath] = useState<string | null>(null);

  // Transcribing state
  const [transcribingPath, setTranscribingPath] = useState<string | null>(null);

  
  // Filter and sort recordings
  const filteredRecordings = useMemo(() => {
    let result = [...recordings];

    // Apply search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      result = result.filter(
        (rec) =>
          rec.filename.toLowerCase().includes(query) ||
          rec.transcription?.toLowerCase().includes(query)
      );
    }

    // Apply status filter
    if (filterOption === "transcribed") {
      result = result.filter((rec) => Boolean(rec.transcription));
    } else if (filterOption === "pending") {
      result = result.filter((rec) => !rec.transcription);
    }

    // Apply sort
    result.sort((a, b) => {
      switch (sortOption) {
        case "newest":
          return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
        case "oldest":
          return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
        case "longest":
          return b.duration_secs - a.duration_secs;
        case "shortest":
          return a.duration_secs - b.duration_secs;
        default:
          return 0;
      }
    });

    return result;
  }, [recordings, searchQuery, filterOption, sortOption]);

  const handleToggleExpand = (filePath: string) => {
    setExpandedPath((current) => (current === filePath ? null : filePath));
  };

  const handlePlay = async (filePath: string) => {
    await togglePlayback(filePath);
  };

  const handleTranscribe = async (filePath: string) => {
    setTranscribingPath(filePath);
    try {
      await invoke<string>("transcribe_file", { filePath });
      // Invalidate all pages to refetch with updated transcription
      await queryClient.invalidateQueries({ queryKey: queryKeys.tauri.listRecordings() });
      toast({
        type: "success",
        title: "Transcription complete",
        description: "Text has been copied to clipboard.",
      });
    } catch (e) {
      toast({
        type: "error",
        title: "Transcription failed",
        description: e instanceof Error ? e.message : String(e),
      });
    } finally {
      setTranscribingPath(null);
    }
  };

  const handleCopyText = async (recording: RecordingInfo) => {
    if (!recording.transcription) return;

    try {
      await navigator.clipboard.writeText(recording.transcription);
      toast({
        type: "success",
        title: "Copied to clipboard",
        description: "Transcription text has been copied.",
      });
    } catch (e) {
      toast({
        type: "error",
        title: "Copy failed",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  };

  const handleOpenFile = async (filePath: string) => {
    try {
      await openPath(filePath);
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to open file",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  };

  const handleDelete = async (filePath: string) => {
    const recording = recordings.find((r) => r.file_path === filePath);
    try {
      await invoke("delete_recording", { filePath });
      // Invalidate all pages to refetch without the deleted recording
      await queryClient.invalidateQueries({ queryKey: queryKeys.tauri.listRecordings() });
      setDeleteConfirmPath(null);
      if (expandedPath === filePath) {
        setExpandedPath(null);
      }
      toast({
        type: "success",
        title: "Recording deleted",
        description: recording ? `"${recording.filename}" has been removed.` : "Recording removed.",
      });
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to delete recording",
        description: e instanceof Error ? e.message : String(e),
      });
    }
  };

  const handleStartRecording = async () => {
    await startRecording();
  };

  if (loading) {
    return (
      <div className="p-6 space-y-6" role="status" aria-label="Loading recordings">
        {/* Header skeleton */}
        <header>
          <div className="h-8 w-40 bg-surface-hover rounded animate-pulse" />
          <div className="h-4 w-80 bg-surface-hover rounded animate-pulse mt-2" />
        </header>

        {/* Search bar skeleton */}
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="flex-1 h-11 bg-surface-hover rounded animate-pulse" />
          <div className="w-full sm:w-40 h-11 bg-surface-hover rounded animate-pulse" />
          <div className="w-full sm:w-40 h-11 bg-surface-hover rounded animate-pulse" />
        </div>

        {/* Recording items skeleton */}
        <div className="space-y-2">
          {[1, 2, 3, 4, 5].map((i) => (
            <Card key={i}>
              <CardContent className="flex items-center gap-3 p-4">
                <div className="w-10 h-10 rounded-full bg-surface-hover animate-pulse flex-shrink-0" />
                <div className="flex-1 min-w-0">
                  <div className="h-4 w-48 bg-surface-hover rounded animate-pulse" />
                  <div className="h-3 w-32 bg-surface-hover rounded animate-pulse mt-1" />
                </div>
                <div className="h-6 w-20 bg-surface-hover rounded-full animate-pulse" />
                <div className="h-4 w-4 bg-surface-hover rounded animate-pulse" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <Card className="border-error">
          <CardContent>
            <div className="text-error" role="alert">
              {error}
            </div>
            <button
              type="button"
              onClick={() => refetch()}
              className="mt-4 text-heycat-orange hover:text-heycat-orange-light"
            >
              Retry
            </button>
          </CardContent>
        </Card>
      </div>
    );
  }

  const hasFiltersActive = searchQuery.trim() !== "" || filterOption !== "all";

  return (
    <div className="p-6 space-y-6">
      {/* Page Header */}
      <header>
        <h1 className="text-2xl font-semibold text-text-primary">
          Recordings
        </h1>
        <p className="text-text-secondary mt-1">
          Manage your voice recordings and transcriptions.
        </p>
      </header>

      {/* Search & Filter Bar */}
      <div className="flex flex-col sm:flex-row gap-3">
        {/* Search Input */}
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-text-secondary" />
          <Input
            type="text"
            placeholder="Search recordings..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
            aria-label="Search recordings"
          />
        </div>

        {/* Filter Dropdown */}
        <div className="w-full sm:w-40">
          <Select
            value={filterOption}
            onValueChange={(value) => setFilterOption(value as FilterOption)}
            placeholder="Filter"
          >
            <SelectItem value="all">All</SelectItem>
            <SelectItem value="transcribed">Transcribed</SelectItem>
            <SelectItem value="pending">Pending</SelectItem>
          </Select>
        </div>

        {/* Sort Dropdown */}
        <div className="w-full sm:w-40">
          <Select
            value={sortOption}
            onValueChange={(value) => setSortOption(value as SortOption)}
            placeholder="Sort by"
          >
            <SelectItem value="newest">Newest</SelectItem>
            <SelectItem value="oldest">Oldest</SelectItem>
            <SelectItem value="longest">Longest</SelectItem>
            <SelectItem value="shortest">Shortest</SelectItem>
          </Select>
        </div>
      </div>

      {/* Recording List */}
      {totalCount === 0 ? (
        <RecordingsEmptyState onStartRecording={handleStartRecording} />
      ) : filteredRecordings.length === 0 ? (
        <Card className="text-center py-8">
          <CardContent>
            <p className="text-text-secondary">
              No recordings match your search
            </p>
            {hasFiltersActive && (
              <button
                type="button"
                onClick={() => {
                  setSearchQuery("");
                  setFilterOption("all");
                }}
                className="mt-2 text-sm text-heycat-orange hover:text-heycat-orange-light"
              >
                Clear filters
              </button>
            )}
          </CardContent>
        </Card>
      ) : (
        <>
          <div className="space-y-2" role="list" aria-label="Recordings list">
            {filteredRecordings.map((recording) => (
              <RecordingItem
                key={recording.file_path}
                recording={recording}
                isExpanded={expandedPath === recording.file_path}
                onToggleExpand={() => handleToggleExpand(recording.file_path)}
                onPlay={() => handlePlay(recording.file_path)}
                onTranscribe={() => handleTranscribe(recording.file_path)}
                onCopyText={() => handleCopyText(recording)}
                onOpenFile={() => handleOpenFile(recording.file_path)}
                onDelete={() => setDeleteConfirmPath(recording.file_path)}
                isPlaying={isAudioPlaying && playingFilePath === recording.file_path}
                isTranscribing={transcribingPath === recording.file_path}
                isDeleting={deleteConfirmPath === recording.file_path}
                onConfirmDelete={() => handleDelete(recording.file_path)}
                onCancelDelete={() => setDeleteConfirmPath(null)}
              />
            ))}
          </div>

          {/* Pagination Controls */}
          {totalPages > 1 && (
            <div className="flex items-center justify-between pt-4">
              <p className="text-sm text-text-secondary">
                Showing {offset + 1}-{Math.min(offset + recordings.length, totalCount)} of {totalCount} recordings
              </p>
              <div className="flex items-center gap-2">
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => setCurrentPage((p) => Math.max(0, p - 1))}
                  disabled={currentPage === 0}
                  aria-label="Previous page"
                >
                  <ChevronLeft className="h-4 w-4" />
                  Previous
                </Button>
                <span className="text-sm text-text-secondary px-2">
                  Page {currentPage + 1} of {totalPages}
                </span>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => setCurrentPage((p) => p + 1)}
                  disabled={!hasMore}
                  aria-label="Next page"
                >
                  Next
                  <ChevronRight className="h-4 w-4" />
                </Button>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}

// Re-export for use by other components
export { type RecordingInfo, type PaginatedRecordingsResponse, formatDuration, formatDate, formatFileSize } from "./components/RecordingItem";

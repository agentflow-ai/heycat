import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "../../lib/queryKeys";
import { openPath } from "@tauri-apps/plugin-opener";
import { Button, Card, CardContent } from "../../components/ui";
import { useToast } from "../../components/overlays";
import { useRecording } from "../../hooks/useRecording";
import { useSettings } from "../../hooks/useSettings";
import { useAudioPlayback } from "../../hooks/useAudioPlayback";
import { useRecordingsFilter } from "../../hooks/useRecordingsFilter";
import { RecordingItem, type RecordingInfo, type PaginatedRecordingsResponse } from "../components/RecordingItem";
import { RecordingsEmptyState } from "../components/RecordingsEmptyState";
import { RecordingsFilters } from "./RecordingsFilters";
import { RecordingsPagination } from "./RecordingsPagination";

const PAGE_SIZE = 20;

export interface RecordingsProps {
  onNavigate?: (page: string) => void;
}

/**
 * Recordings page component.
 * Lists recordings with search, filter, sort, and pagination.
 */
export function Recordings(_props: RecordingsProps) {
  const { toast } = useToast();
  const { isRecording } = useRecording();

  const queryClient = useQueryClient();

  // Audio playback hook
  const {
    toggle: togglePlayback,
    stop: stopPlayback,
    isPlaying: isAudioPlaying,
    currentFilePath: playingFilePath,
    error: playbackError,
  } = useAudioPlayback();

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

  // Fetch recordings via React Query
  const {
    data: paginatedResponse,
    isLoading: loading,
    error: queryError,
    refetch,
  } = useQuery({
    queryKey: queryKeys.tauri.listRecordings(PAGE_SIZE, offset),
    queryFn: () =>
      invoke<PaginatedRecordingsResponse>("list_recordings", {
        limit: PAGE_SIZE,
        offset,
      }),
  });

  const recordings = paginatedResponse?.recordings ?? [];
  const totalCount = paginatedResponse?.total_count ?? 0;
  const hasMore = paginatedResponse?.has_more ?? false;
  const totalPages = Math.ceil(totalCount / PAGE_SIZE);

  const error = queryError
    ? queryError instanceof Error
      ? queryError.message
      : String(queryError)
    : null;

  // Use the recordings filter hook for search, filter, and sort
  const {
    searchQuery,
    setSearchQuery,
    filterOption,
    setFilterOption,
    sortOption,
    setSortOption,
    filteredRecordings,
    hasActiveFilters,
    clearFilters,
  } = useRecordingsFilter({ recordings });

  // Expanded item state
  const [expandedPath, setExpandedPath] = useState<string | null>(null);

  // Delete confirmation state
  const [deleteConfirmPath, setDeleteConfirmPath] = useState<string | null>(null);

  // Transcribing state
  const [transcribingPath, setTranscribingPath] = useState<string | null>(null);

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
      await queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.listRecordings(),
      });
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
      await queryClient.invalidateQueries({
        queryKey: queryKeys.tauri.listRecordings(),
      });
      setDeleteConfirmPath(null);
      if (expandedPath === filePath) {
        setExpandedPath(null);
      }
      toast({
        type: "success",
        title: "Recording deleted",
        description: recording
          ? `"${recording.filename}" has been removed.`
          : "Recording removed.",
      });
    } catch (e) {
      toast({
        type: "error",
        title: "Failed to delete recording",
        description: e instanceof Error ? e.message : String(e),
      });
    }
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

  return (
    <div className="p-6 space-y-6">
      {/* Page Header */}
      <header>
        <h1 className="text-2xl font-semibold text-text-primary">Recordings</h1>
        <p className="text-text-secondary mt-1">
          Manage your voice recordings and transcriptions.
        </p>
      </header>

      {/* Search & Filter Bar */}
      <RecordingsFilters
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        filterOption={filterOption}
        onFilterChange={setFilterOption}
        sortOption={sortOption}
        onSortChange={setSortOption}
      />

      {/* Recording List */}
      {totalCount === 0 ? (
        <RecordingsEmptyState />
      ) : filteredRecordings.length === 0 ? (
        <Card className="text-center py-8">
          <CardContent>
            <p className="text-text-secondary">No recordings match your search</p>
            {hasActiveFilters && (
              <button
                type="button"
                onClick={clearFilters}
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
            <RecordingsPagination
              currentPage={currentPage}
              totalPages={totalPages}
              totalCount={totalCount}
              pageSize={PAGE_SIZE}
              recordingsOnPage={recordings.length}
              hasMore={hasMore}
              onPreviousPage={() => setCurrentPage((p) => Math.max(0, p - 1))}
              onNextPage={() => setCurrentPage((p) => p + 1)}
            />
          )}
        </>
      )}
    </div>
  );
}

// Re-export for use by other components
export { type RecordingInfo, type PaginatedRecordingsResponse } from "../components/RecordingItem";
export { formatDuration, formatDate, formatFileSize } from "../../lib/formatting";
